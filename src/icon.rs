use tao::window::Icon as TaoIcon;
use tray_icon::Icon as TrayIcon;

pub fn get_icon_rgba() -> (Vec<u8>, u32, u32) {
    let width: u32 = 32;
    let height: u32 = 32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let px = x as f32;
            let py = y as f32;

            // Rounded rectangle radius = 6.0, box from (2, 2) to (29, 29)
            let in_box = is_in_rounded_rect(px, py, 2.0, 2.0, 28.0, 28.0, 6.0);

            if !in_box {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
                continue;
            }

            // Note sheet symbol in center: from x: 9..22, y: 8..23
            let in_sheet = px >= 9.0 && px <= 22.0 && py >= 8.0 && py <= 23.0;
            let in_fold = px > 18.0 && py < 13.0 && (px - 18.0) + (13.0 - py) < 5.0;

            if in_sheet && !in_fold {
                let on_line1 = py >= 12.0 && py <= 13.0 && px >= 11.0 && px <= 20.0;
                let on_line2 = py >= 15.0 && py <= 16.0 && px >= 11.0 && px <= 18.0;
                let on_line3 = py >= 18.0 && py <= 19.0 && px >= 11.0 && px <= 16.0;

                if on_line1 || on_line2 || on_line3 {
                    rgba.extend_from_slice(&[210, 150, 0, 255]);
                } else {
                    rgba.extend_from_slice(&[255, 255, 255, 255]);
                }
            } else if in_sheet && in_fold {
                rgba.extend_from_slice(&[230, 230, 230, 255]);
            } else {
                let shadow_factor = 1.0 - (py / 64.0);
                let r = (244.0 * shadow_factor) as u8;
                let g = (180.0 * shadow_factor) as u8;
                let b = (0.0 * shadow_factor) as u8;
                rgba.extend_from_slice(&[r, g, b, 255]);
            }
        }
    }

    (rgba, width, height)
}

pub fn create_keep_icon() -> TrayIcon {
    let (rgba, width, height) = get_icon_rgba();
    TrayIcon::from_rgba(rgba, width, height).expect("Failed to create tray icon")
}

pub fn create_window_icon() -> Option<TaoIcon> {
    let (rgba, width, height) = get_icon_rgba();
    TaoIcon::from_rgba(rgba, width, height).ok()
}

fn is_in_rounded_rect(px: f32, py: f32, x: f32, y: f32, w: f32, h: f32, r: f32) -> bool {
    let right = x + w;
    let bottom = y + h;

    if px < x || px > right || py < y || py > bottom {
        return false;
    }

    if px < x + r && py < y + r {
        let dx = px - (x + r);
        let dy = py - (y + r);
        return dx * dx + dy * dy <= r * r;
    }
    if px > right - r && py < y + r {
        let dx = px - (right - r);
        let dy = py - (y + r);
        return dx * dx + dy * dy <= r * r;
    }
    if px < x + r && py > bottom - r {
        let dx = px - (x + r);
        let dy = py - (bottom - r);
        return dx * dx + dy * dy <= r * r;
    }
    if px > right - r && py > bottom - r {
        let dx = px - (right - r);
        let dy = py - (bottom - r);
        return dx * dx + dy * dy <= r * r;
    }

    true
}
