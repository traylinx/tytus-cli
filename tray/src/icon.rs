//! Tray icon generation. Creates a simple "T" glyph as a template image.
//! On macOS, template images auto-adapt to light/dark mode.

use tray_icon::Icon;

/// Create the tray icon — a bold "T" on transparent background.
/// 22x22 pixels (macOS menu bar standard).
pub fn create_tray_icon() -> Icon {
    let size = 22u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    // Draw a bold "T" shape (white on transparent)
    // Horizontal bar: y=3..6, x=3..19
    for y in 3..7 {
        for x in 3..19 {
            set_pixel(&mut rgba, size, x, y, [255, 255, 255, 255]);
        }
    }
    // Vertical bar: y=6..19, x=9..13
    for y in 6..19 {
        for x in 9..13 {
            set_pixel(&mut rgba, size, x, y, [255, 255, 255, 255]);
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
