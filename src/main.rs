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
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use tray_icon::{menu::MenuEvent, MouseButton, MouseButtonState, TrayIconEvent};
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Paths & Config
    let paths = AppPaths::init();
    let mut config = AppConfig::load(&paths.config_file);

    // 2. Initialize Tao Event Loop with Custom Events
    let event_loop = EventLoopBuilder::<AppCustomEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // 3. Build Window
    let mut window_builder = WindowBuilder::new()
        .with_title("Google Keep Widget")
        .with_inner_size(LogicalSize::new(config.width, config.height))
        .with_min_inner_size(LogicalSize::new(320.0, 420.0))
        .with_always_on_top(config.always_on_top)
        .with_decorations(true)
        .with_visible(true);

    if let (Some(x), Some(y)) = (config.x, config.y) {
        window_builder = window_builder.with_position(LogicalPosition::new(x, y));
    }

    if let Some(w_icon) = create_window_icon() {
        window_builder = window_builder.with_window_icon(Some(w_icon));
    }

    let window = Arc::new(window_builder.build(&event_loop)?);

    // 4. Initialize System Tray & Hotkey
    let tray = AppTray::new(config.always_on_top, config.compact_mode)?;
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

        // Process Tray Menu Events
        while let Ok(menu_event) = menu_channel.try_recv() {
            if menu_event.id == tray.id_toggle {
                let visible = w.is_visible();
                w.set_visible(!visible);
                if !visible {
                    w.set_focus();
                }
            } else if menu_event.id == tray.id_pin {
                config.always_on_top = !config.always_on_top;
                w.set_always_on_top(config.always_on_top);
                tray.set_pin_checked(config.always_on_top);
                let _ = webview.evaluate_script(&format!(
                    "if (window.__KEEP_SET_PIN__) window.__KEEP_SET_PIN__({});",
                    config.always_on_top
                ));
                let _ = config.save(&paths.config_file);
            } else if menu_event.id == tray.id_compact {
                config.compact_mode = !config.compact_mode;
                tray.set_compact_checked(config.compact_mode);
                let _ = webview.evaluate_script(&format!(
                    "if (window.__KEEP_SET_COMPACT__) window.__KEEP_SET_COMPACT__({});",
                    config.compact_mode
                ));
                let _ = config.save(&paths.config_file);
            } else if menu_event.id == tray.id_sync {
                let _ = webview.evaluate_script("window.location.reload();");
            } else if menu_event.id == tray.id_cache {
                let _ = open::that(&paths.webview_data_dir);
            } else if menu_event.id == tray.id_quit {
                let _ = config.save(&paths.config_file);
                *control_flow = ControlFlow::Exit;
            }
        }

        // Process Tray Icon Left-Click Events
        while let Ok(tray_event) = tray_channel.try_recv() {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = tray_event
            {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    let visible = w.is_visible();
                    w.set_visible(!visible);
                    if !visible {
                        w.set_focus();
                    }
                }
            }
        }

        // Process Global Hotkey Events (Alt+Shift+K)
        while let Ok(hotkey_event) = hotkey_channel.try_recv() {
            if let Some(ref hk) = hotkey_manager {
                if hk.is_toggle_event(&hotkey_event) {
                    let visible = w.is_visible();
                    w.set_visible(!visible);
                    if !visible {
                        w.set_focus();
                    }
                }
            }
        }

        match event {
            // Custom IPC events from JavaScript inside WebView
            Event::UserEvent(AppCustomEvent::Ipc(ipc_msg)) => match ipc_msg {
                IpcMessage::TogglePin => {
                    config.always_on_top = !config.always_on_top;
                    w.set_always_on_top(config.always_on_top);
                    tray.set_pin_checked(config.always_on_top);
                    let _ = webview.evaluate_script(&format!(
                        "if (window.__KEEP_SET_PIN__) window.__KEEP_SET_PIN__({});",
                        config.always_on_top
                    ));
                    let _ = config.save(&paths.config_file);
                }
                IpcMessage::ToggleCompact { value } => {
                    config.compact_mode = value;
                    tray.set_compact_checked(value);
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
            },

            // Window Events
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    if config.minimize_to_tray {
                        w.set_visible(false);
                    } else {
                        let _ = config.save(&paths.config_file);
                        *control_flow = ControlFlow::Exit;
                    }
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
