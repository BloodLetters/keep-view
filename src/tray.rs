use crate::icon::create_keep_icon;
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder,
};

pub struct AppTray {
    pub tray_icon: TrayIcon,
    pub id_toggle: MenuId,
    pub id_pin: MenuId,
    pub id_compact: MenuId,
    pub id_sync: MenuId,
    pub id_cache: MenuId,
    pub id_quit: MenuId,
    pub item_pin: CheckMenuItem,
    pub item_compact: CheckMenuItem,
}

impl AppTray {
    pub fn new(always_on_top: bool, compact_mode: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let menu = Menu::new();

        let item_toggle = MenuItem::new("Buka / Sembunyikan Keep (Alt+Shift+K)", true, None);
        let item_pin = CheckMenuItem::new("📌 Always on Top", true, always_on_top, None);
        let item_compact = CheckMenuItem::new("📱 Mode Widget Kompak", true, compact_mode, None);
        let sep1 = PredefinedMenuItem::separator();
        let item_sync = MenuItem::new("🔄 Sinkronkan / Muat Ulang", true, None);
        let item_cache = MenuItem::new("📁 Buka Folder Data Offline", true, None);
        let sep2 = PredefinedMenuItem::separator();
        let item_quit = MenuItem::new("❌ Keluar", true, None);

        let id_toggle = item_toggle.id().clone();
        let id_pin = item_pin.id().clone();
        let id_compact = item_compact.id().clone();
        let id_sync = item_sync.id().clone();
        let id_cache = item_cache.id().clone();
        let id_quit = item_quit.id().clone();

        menu.append(&item_toggle)?;
        menu.append(&item_pin)?;
        menu.append(&item_compact)?;
        menu.append(&sep1)?;
        menu.append(&item_sync)?;
        menu.append(&item_cache)?;
        menu.append(&sep2)?;
        menu.append(&item_quit)?;

        let icon = create_keep_icon();

        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Google Keep Widget (Offline & Auto-Sync)")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()?;

        Ok(Self {
            tray_icon,
            id_toggle,
            id_pin,
            id_compact,
            id_sync,
            id_cache,
            id_quit,
            item_pin,
            item_compact,
        })
    }

    pub fn set_pin_checked(&self, checked: bool) {
        self.item_pin.set_checked(checked);
    }

    pub fn set_compact_checked(&self, checked: bool) {
        self.item_compact.set_checked(checked);
    }

    pub fn set_tooltip(&self, text: &str) {
        let _ = self.tray_icon.set_tooltip(Some(text));
    }
}
