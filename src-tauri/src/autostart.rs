use tauri_plugin_autostart::MacosLauncher;

pub fn init(app: &tauri::App) {
	match try {
		app.handle().plugin(tauri_plugin_autostart::init(
			MacosLauncher::LaunchAgent,
			None,
		))?;
	} {
		Ok(()) => log::info!("autostart plugin initialized"),
		Err::<_, anyhow::Error>(e) => {
			log::error!("autostart plugin initialization failed: {e}")
		}
	}
}
