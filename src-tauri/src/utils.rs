use crate::process_transcription;
use anyhow::{ensure, Context, Result};
use std::{path::PathBuf, time::UNIX_EPOCH};
use tokio::process::Command;
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

/// The directory where this application stores its recordings relative to the home
/// directory.
const RECORDINGS_DIR: &str =
	"Library/Application Support/com.lala.app/tauri-plugin-mic-recorder";

/// The directory where whisper-cli is installed relative to the home directory.
const WHISPER_DIR: &str = "external-libraries/whisper.cpp";

/// Retrieves the path to the most recently modified WAV file
/// from the application's recording directory.
///
/// Returns an error if the home directory cannot be found,
/// or if no WAV files are present in the recordings directory.
pub async fn get_latest_wav_file() -> Result<PathBuf> {
	let recordings_dir = dirs::home_dir()
		.context("Could not find home directory")?
		.join(RECORDINGS_DIR);

	let mut entries = Vec::new();

	let entries_stream = tokio::fs::read_dir(&recordings_dir).await?;
	let mut entries_stream = ReadDirStream::new(entries_stream);
	while let Some(entry_result) = entries_stream.next().await {
		if let Ok(entry) = entry_result
			&& let Some(ext) = entry.path().extension()
			&& ext == "wav"
			&& let Ok(metadata) = tokio::fs::metadata(&entry.path()).await
		{
			entries.push((entry.path(), metadata));
		}
	}

	ensure!(
		!entries.is_empty(),
		"No WAV files found when looking for wav extension"
	);

	let latest = entries
		.into_iter()
		.max_by_key(|(_, metadata)| metadata.modified().unwrap_or(UNIX_EPOCH))
		.map(|(path, _)| path)
		.context("No WAV files found when looking for latest")?;

	Ok(latest)
}

/// Runs whisper-cli on the given WAV file and returns the transcription.
///
/// Returns an error if the home directory cannot be found,
/// or if the whisper-cli command fails.
pub async fn transcribe_audio(wav_path: PathBuf) -> Result<String> {
	let whisper_dir = dirs::home_dir()
		.context("Could not find home directory")?
		.join(WHISPER_DIR);

	let binary = "./build/bin/whisper-cli";
	let output = Command::new(binary)
		.current_dir(&whisper_dir)
		.arg("--output-txt")
		.arg("--no-prints")
		.arg(&wav_path)
		.output()
		.await?;

	log::debug!("Command exit status: {}", output.status);

	ensure!(
		output.status.success(),
		"Whisper CLI failed: {:?}",
		output.stderr
	);

	let stdout = String::from_utf8_lossy(&output.stdout);
	let stdout = stdout.trim();

	log::debug!("Raw stdout: {stdout}");

	Ok(process_transcription(stdout))
}
