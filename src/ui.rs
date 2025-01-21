use raylib::prelude::*;

use crate::audio::gain_to_db;

pub fn volume_meter(d: &mut RaylibDrawHandle, bounds: Rectangle, gain: f32) {
    let volume = gain_to_db(gain);
    let value = (60.0 + volume).max(0.0) / 60.0;
    let h = bounds.height * value;
    d.draw_rectangle_rec(
        Rectangle {
            x: bounds.x,
            y: bounds.y + bounds.height - h,
            width: bounds.width,
            height: h,
        },
        Color::GREEN,
    );
    d.draw_rectangle_lines_ex(bounds, 1.0, Color::BLACK);
}
