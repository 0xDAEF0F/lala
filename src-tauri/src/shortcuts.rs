#[cfg(desktop)]
pub fn setup_shortcuts(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
	use crate::{
		async_task,
		notifs::{self, Notif},
		start_recording_process, IS_RECORDING,
	};
	use std::sync::atomic::Ordering;
	use tauri_plugin_global_shortcut::{
		Code, GlobalShortcutExt, Shortcut, ShortcutState,
	};

	app.handle().plugin(
		tauri_plugin_global_shortcut::Builder::new()
			.with_handler({
				let app_handle = app.handle().clone();
				move |_app, shortcut, event| {
					if *shortcut == Shortcut::new(None, Code::F2)
						&& event.state() == ShortcutState::Pressed
					{
						if IS_RECORDING.load(Ordering::SeqCst) {
							log::debug!("Stopping recording...");
							async_task(app_handle.clone());
						} else {
							log::debug!("Starting recording...");
							tauri::async_runtime::spawn({
								let app_handle = app_handle.clone();
								async move {
									if let Err(e) =
										start_recording_process(app_handle.clone()).await
									{
										log::error!("Failed to start recording: {:#}", e);
										notifs::notify(
											app_handle,
											Notif::FailedToStartRecording,
										);
									}
								}
							});
						}
					}
				}
			})
			.build(),
	)?;

	app.global_shortcut()
		.register(Shortcut::new(None, Code::F2))?;

	Ok(())
}
