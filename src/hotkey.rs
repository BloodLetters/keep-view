use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};

pub struct AppHotKey {
    _manager: GlobalHotKeyManager,
    toggle_hotkey: HotKey,
}

impl AppHotKey {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = GlobalHotKeyManager::new()?;
        let toggle_hotkey = HotKey::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyK);
        manager.register(toggle_hotkey)?;

        Ok(Self {
            _manager: manager,
            toggle_hotkey,
        })
    }

    pub fn is_toggle_event(&self, event: &GlobalHotKeyEvent) -> bool {
        event.id == self.toggle_hotkey.id() && event.state == HotKeyState::Released
    }
}
