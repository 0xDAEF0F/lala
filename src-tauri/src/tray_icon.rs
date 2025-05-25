use tauri::{
	menu::{Menu, MenuItem},
	tray::TrayIconBuilder,
	App,
};

pub fn setup_tray_icon(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
	let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
	let menu = Menu::with_items(app, &[&quit_i])?;

	let _tray = TrayIconBuilder::new()
		.icon(app.default_window_icon().unwrap().clone())
		.menu(&menu)
		.show_menu_on_left_click(true)
		.on_menu_event(|app, event| match event.id.as_ref() {
			"quit" => {
				log::info!("quit menu item was clicked");
				app.exit(0);
			}
			_ => {
				log::info!("menu item {:?} not handled", event.id);
			}
		})
		.build(app)?;

	Ok(())
}
