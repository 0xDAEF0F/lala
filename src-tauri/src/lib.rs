use tauri_plugin_clipboard_manager::ClipboardExt as _;
use tauri_plugin_notification::NotificationExt as _;

#[tauri::command]
fn greet(name: &str) -> String {
	format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_clipboard_manager::init())
		.plugin(tauri_plugin_notification::init())
		.plugin(tauri_plugin_opener::init())
		.setup(|app| {
			app.clipboard().write_text("gotchuuu")?;

			app.notification()
				.builder()
				.title("Tauri")
				.body("Tauri is awesome")
				.show()
				.unwrap();

			Ok(())
		})
		.invoke_handler(tauri::generate_handler![greet])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
