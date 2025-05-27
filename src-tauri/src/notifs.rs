use anyhow::Result;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

#[derive(Debug, Clone)]
pub enum Notif {
	FailedToStartRecording,
	FailedToStopRecording,
	TranscriptionReady(String),
	TranscriptionFailed,
}

#[derive(Debug)]
pub struct Payload {
	pub title: &'static str,
	pub body: String,
}

impl From<Notif> for Payload {
	fn from(notif: Notif) -> Self {
		match notif {
			Notif::FailedToStartRecording => Payload {
				title: "❌",
				body: "Failed to start recording. Please try again.".to_string(),
			},
			Notif::FailedToStopRecording => Payload {
				title: "❌",
				body: "Failed to stop recording. Please try again.".to_string(),
			},
			Notif::TranscriptionReady(text) => Payload {
				title: "Transcription ready ✅",
				body: text.get(..50).unwrap_or(&text).to_string(),
			},
			Notif::TranscriptionFailed => Payload {
				title: "❌",
				body: "Transcription failed. Please try again.".to_string(),
			},
		}
	}
}

pub fn notify(app_handle: AppHandle, notification_type: Notif) -> Result<()> {
	let payload: Payload = notification_type.clone().into();
	app_handle
		.notification()
		.builder()
		.title(payload.title)
		.body(payload.body)
		.show()?;
	Ok(())
}
