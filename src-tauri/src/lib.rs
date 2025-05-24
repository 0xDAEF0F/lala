#![allow(unused)]

use colored::Colorize as _;
use env_logger::fmt::Formatter;
use log::LevelFilter;
use tap::Tap;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt as _;
use tauri_plugin_log::{LogLevel, TimezoneStrategy};
use tauri_plugin_mic_recorder::{start_recording, stop_recording};
use tauri_plugin_notification::NotificationExt as _;

#[tauri::command]
fn greet(name: &str) -> String {
	format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(
			tauri_plugin_log::Builder::new()
				.level(LevelFilter::Warn)
				.level_for("lala_lib", LevelFilter::Info)
				.format(|cb, args, record| {
					use env_logger::fmt::style;
					let style = match record.level() {
						log::Level::Trace => style::AnsiColor::Cyan.on_default(),
						log::Level::Debug => style::AnsiColor::Blue.on_default(),
						log::Level::Info => style::AnsiColor::Green.on_default(),
						log::Level::Warn => style::AnsiColor::Yellow.on_default(),
						log::Level::Error => style::AnsiColor::Red
							.on_default()
							.effects(style::Effects::BOLD),
					};
					cb.finish(format_args!(
						"[{}] [{style}{}{style:#}] [{}]: {}",
						chrono::Local::now()
							.format("%I:%M:%S%p")
							.to_string()
							.yellow()
							.dimmed(),
						record.level(),
						record.target(),
						record.args()
					));
				})
				.build(),
		)
		.plugin(tauri_plugin_clipboard_manager::init())
		.plugin(tauri_plugin_mic_recorder::init())
		.plugin(tauri_plugin_notification::init())
		.plugin(tauri_plugin_opener::init())
		.setup(|app| {
			#[cfg(desktop)]
			{
				use tauri_plugin_global_shortcut::{
					Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
				};

				let f2_shortcut = Shortcut::new(None, Code::F2);
				app.handle().plugin(
					tauri_plugin_global_shortcut::Builder::new()
						.with_handler(move |_app, shortcut, event| {
							if shortcut == &f2_shortcut {
								match event.state() {
									ShortcutState::Pressed => {
										log::info!("Ctrl-N Pressed!");
									}
									ShortcutState::Released => {
										log::info!("Ctrl-N Released!");
									}
								}
							}
						})
						.build(),
				)?;

				app.global_shortcut().register(f2_shortcut)?;
			}

			app.clipboard().write_text("gotchuuu")?;

			app.notification()
				.builder()
				.title("Tauri")
				.body("Tauri is awesome")
				.show()
				.unwrap();

			// todo: fix this
			// let app_handle = app.handle().clone();
			// tauri::async_runtime::spawn(async move {
			// 	dbg!(start_recording(app_handle).await);
			// 	tokio::time::sleep(std::time::Duration::from_secs(5)).await;
			// 	let res = stop_recording().await;
			// 	println!("res: {:?}", res);
			// });

			_ = stop_recording();

			Ok(())
		})
		.invoke_handler(tauri::generate_handler![greet])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
