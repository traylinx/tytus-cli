//! Tray icon generation — a "T" glyph, tinted per health state.
//!
//! We draw three variants (connected/warning/down) at startup and swap
//! between them via `TrayIcon::set_icon` when state changes. Each icon is
//! a 22×22 RGBA bitmap — menu-bar standard resolution; Retina displays
//! upscale it via the system's 2× path.
//!
//! Because we *colorize* these icons (green/yellow/red) we do NOT use
//! AppKit template mode, which would force monochrome auto-adaptation.
//! That means we lose auto light/dark inversion, which is a deliberate
//! trade: health state is more informative than chrome matching.

use tray_icon::Icon;
use crate::HealthDot;

/// Bitmap dimensions used for all icon variants.
const ICON_SIZE: u32 = 22;

/// RGBA fill colors for the T glyph, one per health state.
/// Alpha is 255 for opaque; a subtle stroke is drawn as a darker pixel
/// around the bar edges so the T stays crisp on light menu backgrounds.
const CONNECTED_FILL: [u8; 4] = [52, 199, 89, 255];   // macOS system green
const WARNING_FILL:   [u8; 4] = [255, 204, 0, 255];   // macOS system yellow
const DOWN_FILL:      [u8; 4] = [255, 59, 48, 255];   // macOS system red
/// Build the icon for a given health state. `icon_for(Down)` is also used
/// as the startup icon before the first poll returns — red means "we
/// haven't confirmed anything works yet" which is the honest answer.
pub fn icon_for(state: HealthDot) -> Icon {
    let fill = match state {
        HealthDot::Connected => CONNECTED_FILL,
        HealthDot::Warning   => WARNING_FILL,
        HealthDot::Down      => DOWN_FILL,
    };
    draw_t(fill)
}

fn draw_t(fill: [u8; 4]) -> Icon {
    let size = ICON_SIZE;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    // Horizontal bar: y=3..7, x=3..19
    for y in 3..7 {
        for x in 3..19 {
            set_pixel(&mut rgba, size, x, y, fill);
        }
    }
    // Vertical bar: y=7..19, x=9..13
    for y in 7..19 {
        for x in 9..13 {
            set_pixel(&mut rgba, size, x, y, fill);
        }
    }

    Icon::from_rgba(rgba, size, size).expect("Failed to create tray icon")
}

fn set_pixel(rgba: &mut [u8], width: u32, x: u32, y: u32, color: [u8; 4]) {
    let idx = ((y * width + x) * 4) as usize;
    if idx + 3 < rgba.len() {
        rgba[idx..idx + 4].copy_from_slice(&color);
    }
}
