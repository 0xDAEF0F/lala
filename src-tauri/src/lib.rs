#![feature(let_chains, try_blocks)]

mod autostart;
mod logger;
mod notifs;
mod shortcuts;
mod tray_icon;
mod utils;

use anyhow::{anyhow, Result};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use log::error;
use notifs::Notif;
use std::sync::{
	atomic::{AtomicBool, Ordering},
	mpsc,
};
use tap::TapFallible;
use tauri::{tray::TrayIconId, ActivationPolicy, AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt as _;
use tauri_plugin_mic_recorder::{start_recording, stop_recording};
use tray_icon::{update_tray_icon, AppState};
use utils::transcribe_audio;

pub static IS_RECORDING: AtomicBool = AtomicBool::new(false);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	dotenvy::dotenv().ok();

	tauri::Builder::default()
		.plugin(logger::init())
		.plugin(tauri_plugin_clipboard_manager::init())
		.plugin(tauri_plugin_mic_recorder::init())
		.plugin(tauri_plugin_notification::init())
		.plugin(tauri_plugin_opener::init())
		.setup(|app| {
			#[cfg(target_os = "macos")]
			app.set_activation_policy(ActivationPolicy::Accessory);

			#[cfg(desktop)]
			autostart::init(app);

			shortcuts::setup_shortcuts(app)?;

			let tray_id = tray_icon::setup_tray_icon(app)?;
			app.manage(tray_id);

			// channel for sending paste actions to the main thread
			let (tx, rx) = mpsc::channel::<String>();
			app.manage(tx);

			// Set up receiver on the main thread
			std::thread::spawn({
				let app_handle = app.app_handle().clone();
				move || {
					while let Ok(original_clipboard) = rx.recv() {
						let app_handle_clone = app_handle.clone();
						_ = app_handle.clone().run_on_main_thread(move || {
							simulate_paste(&app_handle_clone, original_clipboard);
						});
					}
				}
			});

			Ok(())
		})
		.on_window_event(|window, event| {
			if let tauri::WindowEvent::CloseRequested { api, .. } = event {
				log::trace!("window close requested. hiding window");
				window.hide().ok();
				api.prevent_close();
			}
		})
		.invoke_handler(tauri::generate_handler![])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

// todo: make sure we only instantiate enigo once on the main thread
// we are currently instantiating it on every paste action
fn simulate_paste(app_handle: &AppHandle, original_clipboard: String) {
	match try {
		let mut enigo = Enigo::new(&Settings::default())?;
		enigo.key(Key::Meta, Direction::Press)?;
		enigo.key(Key::Unicode('v'), Direction::Click)?;
		enigo.key(Key::Meta, Direction::Release)?;

		// Wait a bit for the paste to complete before restoring clipboard
		std::thread::sleep(std::time::Duration::from_millis(100));
	} {
		Ok(()) => {
			log::trace!("Simulated paste keystroke");
			// Restore original clipboard content after paste
			if let Err(e) = restore_clipboard(app_handle, original_clipboard) {
				error!("Failed to restore original clipboard: {e}");
			}
		}
		Err::<_, anyhow::Error>(e) => error!("Failed to simulate paste: {e}"),
	}
}

fn restore_clipboard(app_handle: &AppHandle, content: String) -> Result<()> {
	app_handle.clipboard().write_text(&content)?;
	log::trace!("Restored original clipboard content");
	Ok(())
}

fn start_async_task(app_handle: AppHandle) {
	IS_RECORDING.store(true, Ordering::SeqCst);
	log::trace!("Recording status false => {IS_RECORDING:?}");
	log::trace!("Updating tray icon to recording");
	let tray_id = app_handle.state::<TrayIconId>();
	let tray = app_handle.tray_by_id(tray_id.inner()).unwrap();
	update_tray_icon(&tray, AppState::Recording).unwrap();
	tauri::async_runtime::spawn(async move {
		if let Err(e) = start_recording(app_handle.clone()).await {
			error!("Failed to start recording: {e:#}");
			_ = notifs::notify(app_handle, Notif::FailedToStartRecording);
		}
	});
}

// todo: refactor
fn stop_async_task(app_handle: AppHandle, auto_paste: bool) {
	IS_RECORDING.store(false, Ordering::SeqCst);
	log::trace!("Recording status true => {IS_RECORDING:?}");
	log::trace!("Updating tray icon to transcribing");
	let tray_id = app_handle.state::<TrayIconId>();
	let tray = app_handle.tray_by_id(tray_id.inner()).unwrap();
	update_tray_icon(&tray, AppState::Transcribing).unwrap();
	tauri::async_runtime::spawn(async move {
		match stop_and_process_recording(app_handle.clone(), auto_paste).await {
			Ok(maybe_past_clipboard) => {
				if auto_paste {
					let tx = app_handle.state::<mpsc::Sender<String>>();
					match maybe_past_clipboard {
						Some(clipboard) => {
							tx.send(clipboard).map_err(|e| error!("{e}")).ok();
						}
						None => {
							tx.send("".to_string()).map_err(|e| error!("{e}")).ok();
						}
					}
				}
				update_tray_icon(&tray, AppState::Idle).unwrap();
			}
			Err(e) => {
				error!("Recording processing failed: {e:#}");
				// todo: improve error handling
				if e.to_string().contains("stop recording") {
					notifs::notify(app_handle, Notif::FailedToStopRecording).ok();
					update_tray_icon(&tray, AppState::Idle).unwrap();
				} else {
					notifs::notify(app_handle, Notif::TranscriptionFailed).ok();
					update_tray_icon(&tray, AppState::Idle).unwrap();
				}
			}
		}
	});
}

async fn stop_and_process_recording(
	app_handle: AppHandle,
	silent_notification: bool,
) -> Result<Option<String>> {
	// Capture original clipboard content before overwriting
	let original_clipboard = app_handle.clipboard().read_text().ok();

	let wav_path = stop_recording().await.map_err(|s| anyhow!(s))?;
	log::trace!("Recording stopped successfully");

	log::debug!("Processing WAV file: {wav_path:?}");
	let transcript = transcribe_audio(&wav_path).await?;

	_ = tokio::fs::remove_file(&wav_path).await.tap_err(|err| {
		log::error!("Failed to delete the WAV file: {err}");
	});

	app_handle.clipboard().write_text(&transcript)?;
	log::trace!("Transcript copied to clipboard");

	if !silent_notification {
		notifs::notify(app_handle, Notif::TranscriptionReady(transcript))?;
	}

	Ok(original_clipboard)
}

pub fn cancel_async_task(app_handle: AppHandle) {
	IS_RECORDING.store(false, Ordering::SeqCst);
	log::trace!("Recording cancelled by user");
	let tray_id = app_handle.state::<TrayIconId>();
	let tray = app_handle.tray_by_id(tray_id.inner()).unwrap();
	update_tray_icon(&tray, AppState::Idle).unwrap();
	tauri::async_runtime::spawn(async move {
		match cancel_and_discard_recording().await {
			Ok(()) => {
				log::trace!("Recording cancelled and audio discarded");
				notifs::notify(app_handle, Notif::UserCancelledRecording).ok();
			}
			Err(e) => {
				error!("Failed to cancel recording: {e:#}");
				notifs::notify(app_handle, Notif::FailedToStopRecording).ok();
			}
		}
	});
}

async fn cancel_and_discard_recording() -> Result<()> {
	let wav_path = stop_recording().await.map_err(|s| anyhow!(s))?;
	log::trace!("Recording stopped for cancellation");

	_ = tokio::fs::remove_file(&wav_path).await.tap_err(|err| {
		log::error!("Failed to delete the cancelled WAV file: {err}");
	});

	Ok(())
}
