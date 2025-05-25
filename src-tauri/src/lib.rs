#![feature(let_chains)]

mod logger;
mod notifs;
mod shortcuts;
mod utils;

use anyhow::{Context, Result};
use notifs::Notif;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri_plugin_clipboard_manager::ClipboardExt as _;
use tauri_plugin_mic_recorder::{start_recording, stop_recording};
use utils::{get_latest_wav_file, transcribe_audio};

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

/// Processes the recorded audio after stopping recording.
///
/// Transcribes the audio file, copies it to clipboard, and shows a notification.
async fn process_recording(app_handle: tauri::AppHandle) -> Result<()> {
	if let Err(e) = stop_recording().await {
		anyhow::bail!("Failed to stop recording: {}", e);
	}
	log::debug!("Recording stopped successfully");

	let wav_path = get_latest_wav_file().await?;
	log::debug!("Processing WAV file: {:?}", wav_path);

	// Transcribe the audio
	let transcript = transcribe_audio(wav_path).await?;

	// Copy to clipboard
	if let Err(e) = app_handle.clipboard().write_text(&transcript) {
		anyhow::bail!("Failed to copy to clipboard: {}", e);
	}

	// Create notification with truncated text
	let display_text = if transcript.len() > 100 {
		format!("{}...", &transcript[..97])
	} else {
		transcript.clone()
	};

	// Show notification
	notifs::notify(app_handle, Notif::TranscriptionReady(display_text));
	Ok(())
}

/// Starts the recording process.
///
/// Shows a notification on successful start.
async fn start_recording_process(app_handle: tauri::AppHandle) -> Result<()> {
	if let Err(e) = start_recording(app_handle.clone()).await {
		anyhow::bail!("Failed to start recording: {}", e);
	}

	IS_RECORDING.store(true, Ordering::SeqCst);
	notifs::notify(app_handle, Notif::RecordingStart);
	Ok(())
}

/// Handles the recording task asynchronously.
///
/// This function is called when stopping a recording and handles the entire
/// post-processing workflow with proper error handling.
fn async_task(app_handle: tauri::AppHandle) {
	tauri::async_runtime::spawn(async move {
		IS_RECORDING.store(false, Ordering::SeqCst);

		if let Err(e) = process_recording(app_handle.clone()).await {
			log::error!("Recording processing failed: {:#}", e);

			// Determine the appropriate notification based on error context
			if e.to_string().contains("stop recording") {
				notifs::notify(app_handle, Notif::FailedToStopRecording);
			} else {
				notifs::notify(app_handle, Notif::TranscriptionFailed);
			}
		}
	});
}
