use std::{
    ffi::{c_uint, CString},
    sync::{Arc, RwLock},
};

use audio::{create_stream, gain_to_db, AudioBuffer};
use cpal::{
    traits::{DeviceTrait as _, HostTrait as _, StreamTrait as _},
    Device, Host, Stream,
};
use raylib::prelude::*;
use ui::volume_meter;

mod audio;
mod ui;

#[allow(unused)]
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
    host: Host,
    devices: Vec<Device>,
    device_text: CString,
}

impl Application {
    pub fn new() -> anyhow::Result<Self> {
        let (mut rl, thread) = raylib::init().transparent().title("PNG-rs").build();

        rl.set_target_fps(60);

        let (stream, buf) = create_stream();

        let mouth_open = rl
            .load_texture(&thread, "static/open_mouth.png")
            .map_err(|e| anyhow::anyhow!(e))?;
        let mouth_closed = rl
            .load_texture(&thread, "static/close_mouth.png")
            .map_err(|e| anyhow::anyhow!(e))?;

        let width = rl.get_screen_width() as f32;
        let height = rl.get_screen_height() as f32;

        let host = cpal::default_host();
        let devices = host.input_devices()?.collect::<Vec<_>>();
        let mut device_text = devices
            .iter()
            .map(|d| d.name().unwrap_or_else(|_| "Unknown device".into()))
            .collect::<Vec<_>>()
            .join(";")
            .into_bytes();
        device_text.push(0);
        let device_text = CString::from_vec_with_nul(device_text).unwrap();

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
            host,
            devices,
            device_text,
        })
    }

    fn run(&mut self) {
        self.stream.play().unwrap();
        while !self.rl.window_should_close() {
            self.width = self.rl.get_screen_width() as f32;
            self.height = self.rl.get_screen_height() as f32;

            self.update();
        }
    }

    fn update(&mut self) {
        let (rl, thread) = (&mut self.rl, &mut self.thread);
        let dt = rl.get_frame_time();

        if rl.is_key_pressed(KeyboardKey::KEY_F5) {
            let flags = ConfigFlags::FLAG_WINDOW_MOUSE_PASSTHROUGH as c_uint
                | ConfigFlags::FLAG_WINDOW_TOPMOST as c_uint
                | ConfigFlags::FLAG_WINDOW_UNDECORATED as c_uint;
            unsafe {
                if self.decorated {
                    ffi::SetWindowState(flags);
                } else {
                    ffi::ClearWindowState(flags);
                }
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

        let mut d = rl.begin_drawing(thread);

        let clear_color = match self.decorated {
            true => Color::new(40, 40, 40, 200),
            false => Color::BLANK,
        };
        d.clear_background(clear_color);

        let bounds = Rectangle {
            x: 20.0,
            y: 20.0,
            width: 20.0,
            height: self.height as f32 - 70.0,
        };
        volume_meter(&mut d, bounds, self.buf.read().unwrap().rms());

        d.draw_text(
            &format!("Talking: {}, time: {}", self.talking, self.time_release),
            20,
            self.height as i32 - 30,
            18,
            Color::WHITE,
        );
        let image_pos = Vector2::new(100.0, 20.0);
        let image = match self.talking {
            true => &self.mouth_open,
            false => &self.mouth_closed,
        };
        d.draw_texture_ex(image, image_pos, 0.0, 0.5, Color::WHITE);

        let bounds = Rectangle {
            x: 400.0,
            y: 20.0,
            width: 100.0,
            height: 30.0,
        };
        let mut active = 0;
        d.gui_dropdown_box(bounds, Some(self.device_text.as_c_str()), &mut active, true);

        if self.decorated {
            d.draw_rectangle_lines(0, 0, self.width as i32, self.height as i32, Color::BLACK);
        }
    }
}

fn main() {
    let mut app = Application::new().expect("Error creating application");
    app.run();
}
