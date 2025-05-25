use crate::{start_async_task, stop_async_task, IS_RECORDING};
use std::sync::atomic::Ordering;
use tauri::{
	menu::{Menu, MenuItem},
	tray::{MouseButton, MouseButtonState, TrayIconBuilder},
	App,
};

pub fn setup_tray_icon(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
	let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
	let menu = Menu::with_items(app, &[&quit_i])?;

	let _tray = TrayIconBuilder::new()
		.icon(app.default_window_icon().unwrap().clone())
		.menu(&menu)
		.show_menu_on_left_click(false)
		.on_menu_event(|app, event| match event.id.as_ref() {
			"quit" => {
				log::info!("quit menu item was clicked");
				app.exit(0);
			}
			_ => {
				log::info!("menu item {:?} not handled", event.id);
			}
		})
		.on_tray_icon_event(|tray_icon, event| {
			if let tauri::tray::TrayIconEvent::Click {
				button,
				button_state: MouseButtonState::Down,
				..
			} = event
			{
				let app_handle = tray_icon.app_handle().clone();
				if button == MouseButton::Left {
					match IS_RECORDING.load(Ordering::SeqCst) {
						false => start_async_task(app_handle),
						true => stop_async_task(app_handle),
					}
				} else if button == MouseButton::Right {
					if let Err(e) = tray_icon.set_visible(true) {
						log::warn!("Failed to show tray icon menu: {e}");
					}
				}
			}
		})
		.build(app)?;

	Ok(())
}
