#[cfg(desktop)]
pub fn setup_shortcuts(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
	use crate::{start_async_task, stop_async_task, IS_RECORDING};
	use std::sync::atomic::Ordering;
	use tauri::Manager;
	use tauri_plugin_global_shortcut::{
		Code, GlobalShortcutExt, Shortcut, ShortcutState,
	};

	app.handle().plugin(
		tauri_plugin_global_shortcut::Builder::new()
			.with_handler({
				move |app, &shortcut, event| {
					if shortcut == Shortcut::new(None, Code::F2)
						&& event.state() == ShortcutState::Pressed
					{
						match IS_RECORDING.load(Ordering::SeqCst) {
							true => stop_async_task(app.app_handle().clone(), true),
							false => start_async_task(app.app_handle().clone()),
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
