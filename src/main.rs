use audio::{create_stream, gain_to_db};
use cpal::traits::StreamTrait as _;
use raylib::prelude::*;

mod audio;

fn main() {
    let (mut rl, thread) = raylib::init()
        .transparent()
        .title("PNG-rs")
        .build();

    let (stream, buf) = create_stream();
    stream.play().unwrap();

    let mut decorated = true;

    while !rl.window_should_close() {
        let (w, h) = (rl.get_screen_width(), rl.get_screen_height());

        if rl.is_key_pressed(KeyboardKey::KEY_D) {
            if decorated {
                rl.set_window_state(WindowState::default().set_window_undecorated(true))
            } else {
                rl.clear_window_state(WindowState::default().set_window_undecorated(true))
            }
            decorated = !decorated;
        }

        let mut d = rl.begin_drawing(&thread);

        let clear_color = match decorated {
            true => Color::new(40, 40, 40, 255),
            false => Color::BLANK,
        };
        d.clear_background(clear_color);

        let bounds = Rectangle {
            x: 20.0,
            y: 20.0,
            width: 20.0,
            height: h as f32 - 40.0,
        };
        let volume = gain_to_db(buf.read().unwrap().rms());
        let value = (60.0 + volume).max(0.0) / 60.0;
        volume_meter(&mut d, bounds, value);

        if decorated {
            d.draw_rectangle_lines(0, 0, w, h, Color::BLACK);
        }
    }
}

fn volume_meter(d: &mut RaylibDrawHandle, bounds: Rectangle, value: f32) {
    let h = bounds.height * value;
    d.draw_rectangle_rec(Rectangle {
        x: bounds.x,
        y: bounds.y + bounds.height - h,
        width: bounds.width,
        height: h,
    }, Color::GREEN);
    d.draw_rectangle_lines_ex(bounds, 1.0, Color::BLACK);
}

