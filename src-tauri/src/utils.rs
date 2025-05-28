use anyhow::{bail, ensure, Context, Result};
use std::{path::Path, time::Instant};
use tokio::process::Command;

/// The directory where whisper-cli is installed relative to the home directory.
const WHISPER_DIR: &str = "external-libraries/whisper.cpp";

/// Runs whisper-cli on the given WAV file and returns the transcription.
///
/// Returns an error if the home directory cannot be found,
/// or if the whisper-cli command fails.
pub async fn transcribe_audio(wav_path: &Path) -> Result<String> {
	let whisper_dir = dirs::home_dir()
		.context("Could not find home directory")?
		.join(WHISPER_DIR);

	let binary = "./build/bin/whisper-cli";

	log::trace!("Starting transcription for: {}", wav_path.display());
	let start_time = Instant::now();

	let output = Command::new(binary)
		.current_dir(&whisper_dir)
		.args(["-m", "models/ggml-small.bin"])
		.args(["-l", "auto"]) // auto language detection
		.args(["--no-prints"]) // no verbose
		.arg(wav_path)
		.output()
		.await?;

	let transcription_duration = start_time.elapsed();

	log::debug!(
		"Transcription completed in: {:.2}s",
		transcription_duration.as_secs_f64()
	);

	log::debug!("Command exit status: {}", output.status);

	ensure!(
		output.status.success(),
		"Whisper CLI failed: {:?}",
		output.stderr
	);

	let stdout = String::from_utf8_lossy(&output.stdout);
	let stdout = stdout.trim();

	log::debug!("Raw stdout: {stdout}");

	cleanse_transcription(stdout)
}

fn cleanse_transcription(raw_text: &str) -> Result<String> {
	Ok(raw_text
		.lines()
		.map(|line| {
			// line must contain timestamp pattern [HH:MM:SS.mmm --> HH:MM:SS.mmm]
			if let Some(idx) = line.find(']') {
				Ok(line[idx + 1..].trim())
			} else {
				bail!(
					"Corrupted transcription line, missing ']' timestamp delimiter: {}",
					line
				)
			}
		})
		.collect::<Result<Vec<_>>>()?
		.join(" ")
		.to_owned())
}
