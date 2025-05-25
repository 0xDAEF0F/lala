#![feature(let_chains)]

mod logger;
mod notifs;
mod shortcuts;
mod utils;

use anyhow::{anyhow, Result};
use notifs::Notif;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt as _;
use tauri_plugin_mic_recorder::{start_recording, stop_recording};
use utils::transcribe_audio;

// Global state to track if recording is in progress
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
		.setup(|app| shortcuts::setup_shortcuts(app))
		.invoke_handler(tauri::generate_handler![])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

fn start_async_task(app_handle: AppHandle) {
	IS_RECORDING.store(true, Ordering::SeqCst);
	log::trace!("Recording status false => {IS_RECORDING:?}");
	tauri::async_runtime::spawn(async move {
		if let Err(e) = start_recording(app_handle.clone()).await {
			log::error!("Failed to start recording: {e:#}");
			notifs::notify(app_handle, Notif::FailedToStartRecording);
		}
	});
}

fn stop_async_task(app_handle: AppHandle) {
	IS_RECORDING.store(false, Ordering::SeqCst);
	log::trace!("Recording status true => {IS_RECORDING:?}");
	tauri::async_runtime::spawn(async move {
		if let Err(e) = stop_and_process_recording(app_handle.clone()).await {
			log::error!("Recording processing failed: {e:#}");
			// todo: improve error handling
			if e.to_string().contains("stop recording") {
				notifs::notify(app_handle, Notif::FailedToStopRecording);
			} else {
				notifs::notify(app_handle, Notif::TranscriptionFailed);
			}
		}
	});
}

async fn stop_and_process_recording(app_handle: AppHandle) -> Result<()> {
	let wav_path = stop_recording().await.map_err(|s| anyhow!(s))?;

	log::trace!("Recording stopped successfully");
	log::debug!("Processing WAV file: {wav_path:?}");

	let transcript = transcribe_audio(wav_path).await?;

	app_handle.clipboard().write_text(&transcript)?;

	notifs::notify(app_handle, Notif::TranscriptionReady(transcript));

	Ok(())
}
