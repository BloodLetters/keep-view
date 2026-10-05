// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod hotkey;
mod icon;
mod injected_assets;
mod paths;
mod tray;

use config::AppConfig;
use hotkey::AppHotKey;
use icon::create_window_icon;
use injected_assets::{OFFLINE_FALLBACK_HTML, USER_AGENT, WIDGET_SCRIPT};
use paths::AppPaths;
use tray::AppTray;

use global_hotkey::GlobalHotKeyEvent;
use serde::Deserialize;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tao::{
    dpi::{LogicalSize, PhysicalPosition},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
#[cfg(target_os = "windows")]
use tao::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};
use tray_icon::{menu::MenuEvent, MouseButtonState, TrayIconEvent};
use wry::{NewWindowResponse, WebContext, WebViewBuilder};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    #[serde(rename = "toggle_pin")]
    TogglePin,
    #[serde(rename = "toggle_compact")]
    ToggleCompact { value: bool },
    #[serde(rename = "set_zoom")]
    SetZoom { value: f64 },
    #[serde(rename = "status_changed")]
    StatusChanged { status: String, text: String },
    #[serde(rename = "drag_window")]
    DragWindow,
    #[serde(rename = "close_window")]
    CloseWindow,
}

#[derive(Debug)]
pub enum AppCustomEvent {
    Ipc(IpcMessage),
}

fn is_network_available() -> bool {
    let test_targets = [
        SocketAddr::from(([8, 8, 8, 8], 53)),
        SocketAddr::from(([1, 1, 1, 1], 53)),
    ];

    for target in &test_targets {
        if TcpStream::connect_timeout(target, Duration::from_millis(800)).is_ok() {
            return true;
        }
    }
    false
}

fn has_local_profile_cache(profile_dir: &Path) -> bool {
    let ebwebview = profile_dir.join("EBWebView");
    if ebwebview.exists() {
        if let Ok(entries) = std::fs::read_dir(&ebwebview) {
            return entries.count() > 0;
        }
    }
    false
}

fn is_allowed_internal_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    if lower.starts_with("data:")
        || lower.starts_with("about:")
        || lower.starts_with("blob:")
        || lower.starts_with("javascript:")
    {
        return true;
    }

    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            let host_lower = host.to_lowercase();
            if host_lower == "google.com"
                || host_lower.ends_with(".google.com")
                || host_lower.contains(".google.")
                || host_lower.ends_with(".googleusercontent.com")
                || host_lower.ends_with(".gstatic.com")
                || host_lower.ends_with(".youtube.com")
                || host_lower.ends_with(".googleapis.com")
                || host_lower.ends_with(".1e100.net")
            {
                return true;
            }
        }
    } else if lower.contains("google.com")
        || lower.contains("gstatic.com")
        || lower.contains("googleusercontent.com")
        || lower.contains("youtube.com")
    {
        return true;
    }

    false
}

#[cfg(windows)]
fn get_windows_work_area() -> Option<(i32, i32, i32, i32)> {
    #[repr(C)]
    struct RECT {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    extern "system" {
        fn SystemParametersInfoW(
            ui_action: u32,
            ui_param: u32,
            pv_param: *mut std::ffi::c_void,
            f_win_ini: u32,
        ) -> i32;
    }

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    const SPI_GETWORKAREA: u32 = 0x0030;
    let res = unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            &mut rect as *mut _ as *mut std::ffi::c_void,
            0,
        )
    };

    if res != 0 && (rect.right > rect.left) && (rect.bottom > rect.top) {
        Some((rect.left, rect.top, rect.right, rect.bottom))
    } else {
        None
    }
}

fn calculate_default_spawn_position(
    monitor: Option<tao::monitor::MonitorHandle>,
    width: f64,
    height: f64,
) -> (i32, i32) {
    let margin_right = 24.0;
    let margin_bottom = 20.0;

    #[cfg(windows)]
    if let Some((left, top, right, bottom)) = get_windows_work_area() {
        let scale = monitor.as_ref().map(|m| m.scale_factor()).unwrap_or(1.0);

        let win_w = (width * scale) as i32;
        let win_h = (height * scale) as i32;
        let pad_r = (margin_right * scale) as i32;
        let pad_b = (margin_bottom * scale) as i32;

        let spawn_x = (right - win_w - pad_r).max(left);
        let spawn_y = (bottom - win_h - pad_b).max(top);

        return (spawn_x, spawn_y);
    }

    if let Some(mon) = monitor {
        let size = mon.size();
        let scale = mon.scale_factor();
        let mon_w = size.width as i32;
        let mon_h = size.height as i32;
        let taskbar_h = (48.0 * scale) as i32;

        let win_w = (width * scale) as i32;
        let win_h = (height * scale) as i32;
        let pad_r = (margin_right * scale) as i32;
        let pad_b = (margin_bottom * scale) as i32;

        let spawn_x = mon_w - win_w - pad_r;
        let spawn_y = mon_h - win_h - taskbar_h - pad_b;

        return (spawn_x, spawn_y);
    }

    (100, 100)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Paths & Config
    let paths = AppPaths::init();
    let mut config = AppConfig::load(&paths.config_file);

    // 2. Initialize Tao Event Loop with Custom Events
    let event_loop = EventLoopBuilder::<AppCustomEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // 3. Build Window
    let (default_x, default_y) = calculate_default_spawn_position(event_loop.primary_monitor(), config.width, config.height);
    let (initial_x, initial_y) = match (config.x, config.y) {
        (Some(x), Some(y)) => (x, y),
        _ => (default_x, default_y),
    };

    let mut window_builder = WindowBuilder::new()
        .with_title("Google Keep Widget")
        .with_inner_size(LogicalSize::new(config.width, config.height))
        .with_min_inner_size(LogicalSize::new(config.width, config.height))
        .with_max_inner_size(LogicalSize::new(config.width, config.height))
        .with_position(PhysicalPosition::new(initial_x, initial_y))
        .with_always_on_top(config.always_on_top)
        .with_decorations(false)
        .with_resizable(false)
        .with_visible(true);

    #[cfg(target_os = "windows")]
    {
        window_builder = window_builder.with_skip_taskbar(true);
    }

    if let Some(w_icon) = create_window_icon() {
        window_builder = window_builder.with_window_icon(Some(w_icon));
    }

    let window = Arc::new(window_builder.build(&event_loop)?);

    #[cfg(target_os = "windows")]
    {
        let _ = window.set_skip_taskbar(true);
    }

    // 4. Initialize System Tray & Hotkey
    let tray = AppTray::new()?;
    let hotkey_manager = AppHotKey::new().ok();

    // 5. Configure Persistent WebContext
    // This stores all IndexedDB notes, Service Workers, cookies, and local session
    let mut web_context = WebContext::new(Some(paths.webview_data_dir.clone()));

    // 6. IPC Proxy Handler
    let ipc_proxy = proxy.clone();

    // 7. Build WebView
    let initial_compact = config.compact_mode;
    let initial_zoom = config.zoom_level;
    let initial_pin = config.always_on_top;

    let init_script = format!(
        "{}\nwindow.addEventListener('DOMContentLoaded', () => {{\n    if (window.__KEEP_SET_COMPACT__) window.__KEEP_SET_COMPACT__({});\n    if (window.__KEEP_SET_PIN__) window.__KEEP_SET_PIN__({});\n    if (window.__KEEP_SET_ZOOM__) window.__KEEP_SET_ZOOM__({});\n}});\n",
        WIDGET_SCRIPT, initial_compact, initial_pin, initial_zoom
    );

    let mut builder = WebViewBuilder::new_with_web_context(&mut web_context)
        .with_user_agent(USER_AGENT)
        .with_devtools(true)
        .with_initialization_script(&init_script)
        .with_ipc_handler(move |req| {
            let body = req.body();
            if let Ok(msg) = serde_json::from_str::<IpcMessage>(body) {
                let _ = ipc_proxy.send_event(AppCustomEvent::Ipc(msg));
            }
        })
        .with_new_window_req_handler(|url, _| {
            // Keep all Google/Keep/OAuth internal windows inside the app
            if is_allowed_internal_url(&url) {
                NewWindowResponse::Allow
            } else {
                // External links clicked inside notes open in default browser
                let _ = open::that_detached(&url);
                NewWindowResponse::Deny
            }
        })
        .with_navigation_handler(|url| {
            if is_allowed_internal_url(&url) {
                true
            } else {
                let _ = open::that_detached(&url);
                false
            }
        });

    let has_cache = has_local_profile_cache(&paths.webview_data_dir);
    let online = is_network_available();

    let webview = if !has_cache && !online {
        builder = builder.with_html(OFFLINE_FALLBACK_HTML);
        match builder.build(&*window) {
            Ok(wv) => wv,
            Err(_) => {
                eprintln!("⚠️ Aplikasi Google Keep Widget sudah sedang berjalan di layar/taskbar Anda! Silakan cek jendela yang terbuka atau tekan Alt+Shift+K.");
                return Ok(());
            }
        }
    } else {
        builder = builder.with_url("https://keep.google.com/u/0/");
        match builder.build(&*window) {
            Ok(wv) => wv,
            Err(_) => {
                eprintln!("⚠️ Aplikasi Google Keep Widget sudah sedang berjalan di layar/taskbar Anda! Silakan cek jendela yang terbuka atau tekan Alt+Shift+K.");
                return Ok(());
            }
        }
    };

    // Channels for Tray and Hotkeys
    let tray_channel = TrayIconEvent::receiver();
    let menu_channel = MenuEvent::receiver();
    let hotkey_channel = GlobalHotKeyEvent::receiver();

    // Clone window arc for the event loop
    let w = Arc::clone(&window);

    // 8. Run Event Loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Process Tray Menu Events (Hanya pilihan "Keluar")
        while let Ok(menu_event) = menu_channel.try_recv() {
            if menu_event.id == tray.id_quit {
                let _ = config.save(&paths.config_file);
                *control_flow = ControlFlow::Exit;
            }
        }

        // Process Tray Icon Events (Klik pada tray tidak menutup window)
        while let Ok(tray_event) = tray_channel.try_recv() {
            if let TrayIconEvent::Click {
                button_state,
                ..
            } = tray_event
            {
                if button_state == MouseButtonState::Up {
                    if !w.is_visible() {
                        w.set_visible(true);
                        #[cfg(target_os = "windows")]
                        let _ = w.set_skip_taskbar(true);
                    }
                    w.set_focus();
                }
            }
        }

        // Process Global Hotkey Events (Alt+Shift+K) - Fokuskan widget selalu terbuka
        while let Ok(hotkey_event) = hotkey_channel.try_recv() {
            if let Some(ref hk) = hotkey_manager {
                if hk.is_toggle_event(&hotkey_event) {
                    if !w.is_visible() {
                        w.set_visible(true);
                        #[cfg(target_os = "windows")]
                        let _ = w.set_skip_taskbar(true);
                    }
                    w.set_focus();
                }
            }
        }

        match event {
            // Custom IPC events from JavaScript inside WebView
            Event::UserEvent(AppCustomEvent::Ipc(ipc_msg)) => match ipc_msg {
                IpcMessage::TogglePin => {
                    config.always_on_top = !config.always_on_top;
                    w.set_always_on_top(config.always_on_top);
                    let _ = webview.evaluate_script(&format!(
                        "if (window.__KEEP_SET_PIN__) window.__KEEP_SET_PIN__({});",
                        config.always_on_top
                    ));
                    let _ = config.save(&paths.config_file);
                }
                IpcMessage::ToggleCompact { value } => {
                    config.compact_mode = value;
                    let _ = config.save(&paths.config_file);
                }
                IpcMessage::SetZoom { value } => {
                    config.zoom_level = value;
                    let _ = config.save(&paths.config_file);
                }
                IpcMessage::StatusChanged { status, text } => {
                    let pin_text = if config.always_on_top { "📌 Pinned" } else { "" };
                    tray.set_tooltip(&format!(
                        "Google Keep Widget - {} ({}) {}",
                        status.to_uppercase(),
                        text,
                        pin_text
                    ));
                }
                IpcMessage::DragWindow => {
                    let _ = w.drag_window();
                }
                IpcMessage::CloseWindow => {
                    // Widget dikonfigurasi selalu terbuka dan tidak bisa ditutup
                }
            },

            // Window Events
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    // Widget selalu terbuka dan tidak bisa ditutup kecuali lewat Tray -> Keluar
                }
                WindowEvent::Resized(size) => {
                    config.width = size.width as f64;
                    config.height = size.height as f64;
                    let _ = config.save(&paths.config_file);
                }
                WindowEvent::Moved(pos) => {
                    config.x = Some(pos.x);
                    config.y = Some(pos.y);
                    let _ = config.save(&paths.config_file);
                }
                _ => {}
            },

            Event::MainEventsCleared => {
                // Keep event loop active for asynchronous signals
            }

            _ => {}
        }
    });
}
