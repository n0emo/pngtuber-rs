use std::sync::{Arc, RwLock};

use audio::{create_stream, gain_to_db, AudioBuffer};
use cpal::{traits::StreamTrait as _, Stream};
use raylib::prelude::*;

mod audio;

struct Application {
    rl: RaylibHandle,
    thread: RaylibThread,
    buf: Arc<RwLock<AudioBuffer>>,
    stream: Stream,
    mouth_open: Texture2D,
    mouth_closed: Texture2D,
    decorated: bool,
    talking: bool,
    time_release: f32,
    width: f32,
    height: f32,
}

impl Application {
    pub fn new() -> anyhow::Result<Self> {
        let (mut rl, thread) = raylib::init()
            .transparent()
            .title("PNG-rs")
            .build();

        rl.set_target_fps(60);

        let (stream, buf) = create_stream();

        let mouth_open = rl.load_texture(&thread, "static/open_mouth.png").map_err(|e| anyhow::anyhow!(e))?;
        let mouth_closed = rl.load_texture(&thread, "static/close_mouth.png").map_err(|e| anyhow::anyhow!(e))?;

        let width = rl.get_screen_width() as f32;
        let height = rl.get_screen_height() as f32;

        Ok(Self {
            rl,
            thread,
            buf,
            stream,
            mouth_open,
            mouth_closed,
            decorated: true,
            talking: false,
            time_release: 0.0,
            width,
            height,
        })
    }

    fn run(&mut self) {
        self.stream.play().unwrap();
        while !self.rl.window_should_close() {
            self.width = self.rl.get_screen_width() as f32;
            self.height = self.rl.get_screen_height() as f32;

            self.update();
            self.draw();
        }
    }

    fn update(&mut self) {
        let (rl, _thread) = (&mut self.rl, &mut self.thread);
        let dt = rl.get_frame_time();

        if rl.is_key_pressed(KeyboardKey::KEY_D) {
            if self.decorated {
                rl.set_window_state(WindowState::default().set_window_undecorated(true))
            } else {
                rl.clear_window_state(WindowState::default().set_window_undecorated(true))
            }
            self.decorated = !self.decorated;
        }

        let volume = gain_to_db(self.buf.read().unwrap().rms());
        if volume > -40.0 {
            self.time_release = 0.25;
        }

        if !self.talking && volume > -40.0 {
            self.talking = true;
        } else if self.talking {
            self.time_release -= dt;
            if self.time_release <= 0.0 {
                self.talking = false;
            }
        }
    }

    fn draw(&mut self) {
        let (rl, thread) = (&mut self.rl, &mut self.thread);
        let mut d = rl.begin_drawing(thread);

        let clear_color = match self.decorated {
            true => Color::new(40, 40, 40, 255),
            false => Color::BLANK,
        };
        d.clear_background(clear_color);

        let bounds = Rectangle {
            x: 20.0,
            y: 20.0,
            width: 20.0,
            height: self.height as f32 - 70.0,
        };
        let volume = gain_to_db(self.buf.read().unwrap().rms());
        let value = (60.0 + volume).max(0.0) / 60.0;
        volume_meter(&mut d, bounds, value);

        d.draw_text(&format!("Talking: {}, time: {}", self.talking, self.time_release), 20, self.height as i32 - 30, 18, Color::WHITE);
        let image_pos = Vector2::new(100.0, 20.0);
        let image = match self.talking {
            true => &self.mouth_open,
            false => &self.mouth_closed,
        };
        d.draw_texture_ex(image, image_pos, 0.0, 0.5, Color::WHITE);

        if self.decorated {
            d.draw_rectangle_lines(0, 0, self.width as i32, self.height as i32, Color::BLACK);
        }
    }
}

fn main() {
    let mut app = Application::new().expect("Error creating application");
    app.run();
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

