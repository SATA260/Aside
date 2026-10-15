//! 尺寸用的占位画家。浮窗里的嘎嘎用 src/assets/characters/gaga 里的画，测试不会覆盖那些文件。

use image::{Rgba, RgbaImage};

use super::{FRAME_H, FRAME_W};

const BLUE: Rgba<u8> = Rgba([62, 74, 99, 255]);
const CREAM: Rgba<u8> = Rgba([244, 236, 220, 255]);
const ORANGE: Rgba<u8> = Rgba([224, 160, 96, 255]);
const INK: Rgba<u8> = Rgba([28, 36, 48, 255]);

pub fn sheet(state: &str, frames: u32) -> RgbaImage {
    let mut image = RgbaImage::new(FRAME_W * frames, FRAME_H);
    for frame in 0..frames {
        paint(&mut image, frame, frames, state);
    }
    image
}

fn paint(image: &mut RgbaImage, frame: u32, frames: u32, state: &str) {
    let ox = frame * FRAME_W;
    let bob = match state {
        "pause" => 0,
        "done" => (frame as i32 % 2) * 6,
        _ => (frame as i32 % 2) * 2,
    };
    let wing = match state {
        "pause" => -28 + frame as i32 * 8,
        "done" => -20 + (frame as i32 * 14) % 28,
        "last5" => 6,
        _ => 4 + (frame as i32 % 2) * 3,
    };
    ellipse(image, ox, 180, 250 + bob, 92, 120, BLUE);
    ellipse(image, ox, 180, 270 + bob, 58, 78, CREAM);
    ellipse(image, ox, 180, 150 + bob, 78, 64, BLUE);
    ellipse(image, ox, 180, 168 + bob, 46, 36, CREAM);
    ellipse(image, ox, 180, 158 + bob, 10, 7, ORANGE);
    ellipse(image, ox, 150 - wing, 250 + bob, 22, 36, BLUE);
    ellipse(image, ox, 210 + wing, 250 + bob, 22, 36, BLUE);
    ellipse(image, ox, 150, 360, 28, 16, ORANGE);
    ellipse(image, ox, 210, 360, 28, 16, ORANGE);
    let blink = state == "blink" && frame + 1 == frames;
    eye(image, ox + 158, (164 + bob) as u32, blink);
    eye(image, ox + 196, (164 + bob) as u32, blink);
    let _ = frames;
}

fn eye(image: &mut RgbaImage, x: u32, y: u32, shut: bool) {
    if shut {
        hline(image, x.saturating_sub(8), x + 8, y, INK);
        return;
    }
    ellipse(image, 0, x as i32, y as i32, 8, 8, INK);
    ellipse(image, 0, x as i32 - 2, y as i32 - 2, 3, 3, CREAM);
}

fn ellipse(image: &mut RgbaImage, ox: u32, cx: i32, cy: i32, rx: i32, ry: i32, color: Rgba<u8>) {
    if rx <= 0 || ry <= 0 {
        return;
    }
    for y in (cy - ry)..(cy + ry) {
        for x in (cx - rx)..(cx + rx) {
            let dx = (x - cx) as f32 / rx as f32;
            let dy = (y - cy) as f32 / ry as f32;
            if dx * dx + dy * dy <= 1.0 {
                put(image, ox as i32 + x, y, color);
            }
        }
    }
}

fn hline(image: &mut RgbaImage, x0: u32, x1: u32, y: u32, color: Rgba<u8>) {
    for x in x0..=x1 {
        put(image, x as i32, y as i32, color);
    }
}

fn put(image: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x >= 0 && y >= 0 && (x as u32) < image.width() && (y as u32) < image.height() {
        image.put_pixel(x as u32, y as u32, color);
    }
}
