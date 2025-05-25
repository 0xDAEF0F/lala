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
	pub title: String,
	pub body: String,
}

impl From<Notif> for Payload {
	fn from(notif: Notif) -> Self {
		match notif {
			Notif::FailedToStartRecording => Payload {
				title: "Recording error".to_string(),
				body: "Failed to start recording. Please try again.".to_string(),
			},
			Notif::FailedToStopRecording => Payload {
				title: "Recording error".to_string(),
				body: "Failed to stop recording. Please try again.".to_string(),
			},
			Notif::TranscriptionReady(text) => Payload {
				title: "Transcription ready".to_string(),
				body: text.get(..50).unwrap_or(&text).to_string(),
			},
			Notif::TranscriptionFailed => Payload {
				title: "Transcription error".to_string(),
				body: "Transcription failed. Please try again.".to_string(),
			},
		}
	}
}

/// handles logging if there is an error displaying the notification
pub fn notify(app_handle: AppHandle, notification_type: Notif) {
	let payload: Payload = notification_type.clone().into();
	app_handle
		.notification()
		.builder()
		.title(payload.title)
		.body(payload.body)
		.show()
		.map_or_else(
			|e| log::error!("Failed show notif: {e}"),
			|_| log::trace!("Notif {notification_type:?}"),
		);
}
