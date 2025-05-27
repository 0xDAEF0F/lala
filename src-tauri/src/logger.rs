use colored::Colorize as _;
use log::LevelFilter;
use std::env;
use tauri::{plugin::TauriPlugin, Runtime};
use tauri_plugin_log::Builder;

/// logs go to `~/Library/Logs/com.lala.app`
pub fn init<R: Runtime>() -> TauriPlugin<R> {
	Builder::new()
		.level(LevelFilter::Warn)
		.level_for("lala_lib", {
			env::var("RUST_LOG")
				.ok()
				.and_then(|s| {
					// First, try to parse as a simple level (e.g., "debug", "info")
					if let Ok(level) = s.parse::<LevelFilter>() {
						Some(level)
					} else {
						// Otherwise, look for module-specific setting (e.g.,
						// "foo=warn,lala_lib=debug")
						s.split(',')
							.find(|s| s.contains("lala_lib"))
							.and_then(|s| s.split('=').nth(1))
							.and_then(|level_str| level_str.parse::<LevelFilter>().ok())
					}
				})
				.unwrap_or(LevelFilter::Info)
		})
		.format(|cb, _, record| {
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
		.build()
}
