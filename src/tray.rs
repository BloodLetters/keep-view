use crate::icon::create_keep_icon;
use tray_icon::{
    menu::{Menu, MenuId, MenuItem},
    TrayIcon, TrayIconBuilder,
};

pub struct AppTray {
    pub tray_icon: TrayIcon,
    pub id_quit: MenuId,
}

impl AppTray {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let menu = Menu::new();

        let item_quit = MenuItem::new("Keluar", true, None);
        let id_quit = item_quit.id().clone();
        menu.append(&item_quit)?;

        let icon = create_keep_icon();

        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Google Keep Widget")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(true)
            .build()?;

        Ok(Self {
            tray_icon,
            id_quit,
        })
    }

    pub fn set_tooltip(&self, text: &str) {
        let _ = self.tray_icon.set_tooltip(Some(text));
    }
}
