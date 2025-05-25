mod logger;
mod notifs;

use iter_tools::Itertools;
use notifs::Notif;
use std::{
	fs,
	path::PathBuf,
	sync::atomic::{AtomicBool, Ordering},
	time::SystemTime,
};
use tauri_plugin_clipboard_manager::ClipboardExt as _;
use tauri_plugin_mic_recorder::{start_recording, stop_recording};
use tokio::process::Command;

// Global state to track if recording is in progress
static IS_RECORDING: AtomicBool = AtomicBool::new(false);

// Get the path to the most recently created WAV file
fn get_latest_wav_file() -> anyhow::Result<PathBuf> {
	let app_support_dir = dirs::home_dir()
		.ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
		.join("Library/Application Support/com.lala.app/tauri-plugin-mic-recorder");

	let entries = fs::read_dir(&app_support_dir)?
		.filter_map(|entry| entry.ok())
		.filter(|entry| {
			entry
				.path()
				.extension()
				.map(|ext| ext == "wav")
				.unwrap_or(false)
		})
		.collect::<Vec<_>>();

	let latest = entries
		.into_iter()
		.max_by_key(|entry| {
			entry
				.metadata()
				.ok()
				.and_then(|m| m.modified().ok())
				.unwrap_or(SystemTime::UNIX_EPOCH)
		})
		.ok_or_else(|| anyhow::anyhow!("No WAV files found"))?;

	Ok(latest.path())
}

// Run whisper-cli on the given WAV file and return the path to the generated TXT file
async fn transcribe_audio(wav_path: PathBuf) -> anyhow::Result<String> {
	let whisper_dir = dirs::home_dir()
		.ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
		.join("external-libraries/whisper.cpp");

	// Log the command we're about to run
	log::info!("Running whisper-cli from directory: {:?}", whisper_dir);
	log::info!("Processing audio file: {:?}", wav_path);

	// Run whisper-cli
	let mut command = Command::new("./build/bin/whisper-cli");
	command
		.current_dir(&whisper_dir)
		.arg("--output-txt")
		.arg("--no-prints")
		.arg(&wav_path);

	log::info!(
		"Executing whisper-cli with audio file: {}",
		wav_path.display()
	);

	let output = command.output().await?;

	log::info!("Command exit status: {}", output.status);

	if !output.status.success() {
		// Extract error message
		let stderr = String::from_utf8_lossy(&output.stderr);
		log::error!("Command stderr: {}", stderr);
		anyhow::bail!("Whisper CLI failed: {}", stderr);
	}

	// Extract transcription from stdout
	let stdout = String::from_utf8_lossy(&output.stdout);
	if !stdout.is_empty() {
		log::info!("Command stdout: {}", stdout);

		// Process the transcription to remove timestamps
		let text = process_transcription(stdout.trim());
		return Ok(text);
	}

	// If no stdout, try to check for the file in both possible locations
	let possible_locations = [
		// Next to the input file
		wav_path.with_extension("txt"),
		// In the whisper directory
		whisper_dir
			.join(wav_path.file_name().unwrap_or_default())
			.with_extension("txt"),
	];

	for txt_path in &possible_locations {
		log::info!("Checking for output file at: {:?}", txt_path);
		if txt_path.exists() {
			let content = fs::read_to_string(txt_path)?;
			return Ok(content);
		}
	}

	anyhow::bail!("Could not find or extract transcription")
}

// Add a new function to process the transcription
fn process_transcription(raw_text: &str) -> String {
	// Split by lines and process each line
	raw_text
		.lines()
		.map(|line| {
			// Check if line contains timestamp pattern [HH:MM:SS.mmm --> HH:MM:SS.mmm]
			if let Some(idx) = line.find(']') {
				// Extract only the text part after the timestamp
				line[idx + 1..].trim().to_string()
			} else {
				// If no timestamp format found, keep the line as is
				line.trim().to_string()
			}
		})
		.collect_vec()
		.join(" ")
		.trim()
		.to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(logger::init())
		.plugin(tauri_plugin_clipboard_manager::init())
		.plugin(tauri_plugin_mic_recorder::init())
		.plugin(tauri_plugin_notification::init())
		.plugin(tauri_plugin_opener::init())
		.setup(|app| {
			#[cfg(desktop)]
			{
				use tauri_plugin_global_shortcut::{
					Code, GlobalShortcutExt, Shortcut, ShortcutState,
				};

				let f2_shortcut = Shortcut::new(None, Code::F2);
				let app_handle = app.handle().clone();

				app.handle().plugin(
					tauri_plugin_global_shortcut::Builder::new()
						.with_handler({
							let app_handle = app_handle.clone();
							move |_app, shortcut, event| {
								if shortcut == &f2_shortcut
									&& event.state() == ShortcutState::Pressed
								{
									let app_handle = app_handle.clone();

									// Toggle recording state and take appropriate action
									if IS_RECORDING.load(Ordering::SeqCst) {
										log::info!("Stopping recording...");
										IS_RECORDING.store(false, Ordering::SeqCst);

										// Launch the async processing pipeline
										tauri::async_runtime::spawn(async move {
											match stop_recording().await {
												Ok(_) => {
													log::info!(
														"Recording stopped successfully"
													);

													// Process the recording
													match get_latest_wav_file() {
														Ok(wav_path) => {
															log::info!(
																"Processing WAV file: \
																 {:?}",
																wav_path
															);

															// Transcribe the audio
															match transcribe_audio(
																wav_path,
															)
															.await
															{
																Ok(transcript) => {
																	// Copy to clipboard
																	if let Err(e) = app_handle.clipboard().write_text(&transcript) {
																		log::error!("Failed to copy to clipboard: {}", e);
																	}

																	// Create notification
																	// with truncated text
																	let display_text =
																		if transcript
																			.len() > 100
																		{
																			format!("{}...", &transcript[..97])
																		} else {
																			transcript
																				.clone()
																		};

																	// Show notification
																	notifs::notify(
																		app_handle,
																		Notif::TranscriptionReady(
																			display_text,
																		),
																	);
																}
																Err(e) => {
																	log::error!(
																		"Transcription \
																		 failed: {}",
																		e
																	);
																	notifs::notify(
																		app_handle,
																		Notif::TranscriptionFailed,
																	);
																}
															}
														}
														Err(e) => {
															notifs::notify(
																app_handle,
																Notif::TranscriptionFailed,
															);
														}
													}
												}
												Err(e) => {
													log::error!(
														"Failed to stop recording: {}",
														e
													);
													notifs::notify(
														app_handle,
														Notif::FailedToStopRecording,
													);
												}
											}
										});
									} else {
										log::info!("Starting recording...");

										// Start a new recording
										tauri::async_runtime::spawn({
											let app_handle_rec = app_handle.clone();
											let app_handle_notify = app_handle.clone();
											async move {
												match start_recording(app_handle_rec)
													.await
												{
													Ok(_) => {
														IS_RECORDING.store(
															true,
															Ordering::SeqCst,
														);
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
			}

			Ok(())
		})
		.invoke_handler(tauri::generate_handler![])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
