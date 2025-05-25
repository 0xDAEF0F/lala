#![feature(let_chains)]

mod logger;
mod notifs;
mod utils;

use anyhow::Result;
use notifs::Notif;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri_plugin_clipboard_manager::ClipboardExt as _;
use tauri_plugin_mic_recorder::{start_recording, stop_recording};
use utils::{get_latest_wav_file, transcribe_audio};

// Global state to track if recording is in progress
static IS_RECORDING: AtomicBool = AtomicBool::new(false);

fn async_task(app_handle: tauri::AppHandle) {
	tauri::async_runtime::spawn(async move {
		match stop_recording().await {
			Ok(_) => {
				log::debug!("Recording stopped successfully");

				match get_latest_wav_file().await {
					Ok(wav_path) => {
						log::debug!("Processing WAV file: {:?}", wav_path);

						// Transcribe the audio
						match transcribe_audio(wav_path).await {
							Ok(transcript) => {
								// Copy to clipboard
								if let Err(e) =
									app_handle.clipboard().write_text(&transcript)
								{
									log::error!("Failed to copy to clipboard: {}", e);
								}

								// Create notification
								// with truncated text
								let display_text = if transcript.len() > 100 {
									format!("{}...", &transcript[..97])
								} else {
									transcript.clone()
								};

								// Show notification
								notifs::notify(
									app_handle,
									Notif::TranscriptionReady(display_text),
								);
							}
							Err(e) => {
								log::error!("Transcription failed: {}", e);
								notifs::notify(app_handle, Notif::TranscriptionFailed);
							}
						}
					}
					Err(e) => {
						notifs::notify(app_handle, Notif::TranscriptionFailed);
					}
				}
			}
			Err(e) => {
				log::error!("Failed to stop recording: {}", e);
				notifs::notify(app_handle, Notif::FailedToStopRecording);
			}
		}
	});
}

#[cfg(desktop)]
fn setup_shortcuts(app: &mut tauri::App) -> Result<()> {
	use tauri_plugin_global_shortcut::{
		Code, GlobalShortcutExt, Shortcut, ShortcutState,
	};

	let f2_shortcut = Shortcut::new(None, Code::F2);
	let app_handle = app.handle().clone();

	app.handle().plugin(
		tauri_plugin_global_shortcut::Builder::new()
			.with_handler({
				move |_app, shortcut, event| {
					if shortcut == &f2_shortcut && event.state() == ShortcutState::Pressed
					{
						let app_handle = app_handle.clone();
						if IS_RECORDING.load(Ordering::SeqCst) {
							log::debug!("Stopping recording...");
							IS_RECORDING.store(false, Ordering::SeqCst);

							async_task(app_handle);
						} else {
							log::debug!("Starting recording...");
							tauri::async_runtime::spawn({
								let app_handle_rec = app_handle.clone();
								let app_handle_notify = app_handle.clone();
								async move {
									match start_recording(app_handle_rec).await {
										Ok(_) => {
											IS_RECORDING.store(true, Ordering::SeqCst);
											notifs::notify(
												app_handle_notify,
												Notif::RecordingStart,
											);
										}
										Err(e) => {
											notifs::notify(
												app_handle_notify,
												Notif::FailedToStartRecording,
											);
										}
									}
								}
							});
						}
					}
				}
			})
			.build(),
	)?;

	app.global_shortcut().register(f2_shortcut)?;
	Ok(())
}

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
			setup_shortcuts(app)?;
			Ok(())
		})
		.invoke_handler(tauri::generate_handler![])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
