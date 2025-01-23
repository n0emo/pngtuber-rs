use std::ffi::{c_uint, CStr, CString};

use anyhow::Context as _;
use audio::{gain_to_db, AudioInterface};
use raylib::prelude::*;
use ui::volume_meter;

mod audio;
mod ui;

enum View {
    Main,
    Overlay,
    Settings {
        audio_dropdown: bool
    },
}

#[allow(unused)]
struct Application {
    mouth_closed: Texture2D,
    mouth_open: Texture2D,
    rl: RaylibHandle,
    thread: RaylibThread,
    device_list: Vec<String>,
    current_device: usize,
    device_text: CString,
    audio: AudioInterface,
    decorated: bool,
    talking: bool,
    time_release: f32,
    width: f32,
    height: f32,
    view: View,
    error_message: Option<String>,
}

impl Application {
    pub fn new() -> anyhow::Result<Self> {
        let (mut rl, thread) = raylib::init().transparent().title("PNG-rs").build();

        rl.set_target_fps(60);

        let mouth_open = rl
            .load_texture(&thread, "static/open_mouth.png")
            .map_err(|e| anyhow::anyhow!(e))?;
        let mouth_closed = rl
            .load_texture(&thread, "static/close_mouth.png")
            .map_err(|e| anyhow::anyhow!(e))?;

        let width = rl.get_screen_width() as f32;
        let height = rl.get_screen_height() as f32;

        let audio = AudioInterface::new(None)?;

        let device_list = audio.available_inputs()?;
        let mut device_text = device_list.join(";").into_bytes();
        device_text.push(0);
        let device_text = CString::from_vec_with_nul(device_text).unwrap();
        let current_name = audio.current_name()?;
        let current_device = device_list.iter().position(|s| *s == current_name)
            .context("Something really wrong happened when searching for default device index")?;

        Ok(Self {
            rl,
            thread,
            audio,
            mouth_open,
            mouth_closed,
            decorated: true,
            talking: false,
            time_release: 0.0,
            width,
            height,
            device_list,
            current_device,
            device_text,
            error_message: None,
            view: View::Main,
        })
    }

    fn run(&mut self) {
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
                    self.view = View::Overlay;
                } else {
                    ffi::ClearWindowState(flags);
                    self.view = View::Main;
                }
            }

            self.decorated = !self.decorated;
        }

        let volume = gain_to_db(self.audio.rms());
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
            true => Color::new(40, 40, 40, 50),
            false => Color::BLANK,
        };
        d.clear_background(clear_color);

        match self.view {
            View::Main => {
                let image_pos = Vector2::new(100.0, 20.0);
                let image = match self.talking {
                    true => &self.mouth_open,
                    false => &self.mouth_closed,
                };
                d.draw_texture_ex(image, image_pos, 0.0, 0.5, Color::WHITE);

                if icon_button(&mut d, (20.0, 20.0), c"#142#") {
                    self.view = View::Settings { audio_dropdown: false };
                }

                let bounds = Rectangle {
                    x: 20.0,
                    y: 50.0,
                    width: 20.0,
                    height: self.height - 100.0,
                };
                volume_meter(&mut d, bounds, self.audio.rms());

                d.draw_text(
                    &format!("Talking: {}, time: {}", self.talking, self.time_release),
                    20,
                    self.height as i32 - 30,
                    18,
                    Color::WHITE,
                );
            },
            View::Overlay => {
                let image_pos = Vector2::new(100.0, 20.0);
                let image = match self.talking {
                    true => &self.mouth_open,
                    false => &self.mouth_closed,
                };
                d.draw_texture_ex(image, image_pos, 0.0, 0.5, Color::WHITE);
            }
            View::Settings { ref mut audio_dropdown } => 'settings: {
                if icon_button(&mut d, (20.0, 20.0), c"#72#") {
                    self.view = View::Main;
                    break 'settings;
                }

                let bounds = Rectangle {
                    x: 20.0,
                    y: 50.0,
                    width: 200.0,
                    height: 30.0,
                };

                let mut active = self.current_device as i32;
                if d.gui_dropdown_box(bounds, Some(self.device_text.as_c_str()), &mut active, *audio_dropdown) {
                    *audio_dropdown = !*audio_dropdown;
                }
                if active != self.current_device as i32 {
                    match AudioInterface::new(Some(&self.device_list[active as usize])) {
                        Ok(new_audio) => {
                            self.audio = new_audio;
                            self.current_device = active as usize;
                        },
                        Err(e) => self.error_message = Some(e.to_string()),
                    }
                }
            }
        }

        if let Some(e) = self.error_message.as_ref() {
            let mut msg = textwrap::wrap(e, 100).join("\n");
            msg.push('\0');
            let w = 500.0;
            let h = 130.0;
            let res = d.gui_message_box(
                Rectangle {
                    x: (self.width - w) * 0.5,
                    y: (self.height - h) * 0.5,
                    width: w,
                    height: h,
                },
                Some(c"Error"),
                Some(CStr::from_bytes_with_nul(&msg.as_bytes()).unwrap()),
                Some(c"OK"),
            );

            if res >= 0 {
                self.error_message = None;
            }
        }

        if self.decorated {
            d.draw_rectangle_lines(0, 0, self.width as i32, self.height as i32, Color::BLACK);
        }
    }
}

fn icon_button(d: &mut RaylibDrawHandle, pos: impl Into<Vector2>, icon: &CStr) -> bool {
    let pos: Vector2 = pos.into();
    let bounds = Rectangle {
        x: pos.x,
        y: pos.y,
        width: 20.0,
        height: 20.0,
    };
    d.gui_button(bounds, Some(icon))
}

fn main() {
    let mut app = Application::new().expect("Error creating application");
    app.run();
}
