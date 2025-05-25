use anyhow::Context as _;
use colored::Colorize as _;
use log::LevelFilter;
use std::env;
use tauri::{plugin::TauriPlugin, Runtime};
use tauri_plugin_log::Builder;

pub fn init<R>() -> TauriPlugin<R>
where
	R: Runtime,
{
	Builder::new()
		.level(LevelFilter::Warn)
		.level_for("lala_lib", {
			env::var("RUST_LOG")
				.context("Could not find RUST_LOG in env vars")
				.and_then(|s| {
					if let Some(substr) = s.split(',').find(|s| s.contains("lala_lib")) {
						Ok(substr.parse::<LevelFilter>()?)
					} else {
						Ok(LevelFilter::Info) // if parsing fails => we get logs `Info`
					}
				}) // if reading `RUST_LOG` fails => we get logs `Info`
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
