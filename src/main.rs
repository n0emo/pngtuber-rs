use std::sync::{Arc, Mutex};

use audio::{gain_to_db, AudioError, AudioInterface};
use eframe::CreationContext;
use egui::{
    include_image, Color32, ComboBox, FontId, Frame, Image, Pos2, Rect, RichText, Ui, Vec2,
    ViewportCommand, ViewportInfo, WindowLevel,
};
use ui::AudioUiExt as _;

mod audio;
mod ui;

fn main() -> eframe::Result {
    eframe::run_native(
        "PNGTuber-rs",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_transparent(true),
            ..Default::default()
        },
        Box::new(|cc| {
            let app = Application::new(cc)?;
            Ok(Box::new(app))
        }),
    )
}

enum View {
    Main,
    Overlay,
    Settings,
}

#[derive(Debug, thiserror::Error)]
enum AppCreationError {
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),
}

#[allow(unused)]
struct Application {
    device_list: Vec<String>,
    current_device: usize,
    audio: AudioInterface,
    talking: bool,
    time_release: f32,
    view: View,
    error_message: Option<String>,
    top_padding: f32,
    second_window: Arc<Mutex<bool>>,
}

impl Application {
    pub fn new(cc: &CreationContext) -> Result<Self, AppCreationError> {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let audio = AudioInterface::new(None)?;

        let device_list = audio.available_inputs()?;
        let mut device_text = device_list.join(";").into_bytes();
        device_text.push(0);
        let current_name = audio.current_name()?;
        let current_device = device_list
            .iter()
            .position(|s| *s == current_name)
            .expect("Something really wrong happened when searching for default device index");

        Ok(Self {
            audio,
            talking: false,
            time_release: 0.0,
            device_list,
            current_device,
            error_message: None,
            view: View::Main,
            top_padding: 0.0,
            second_window: Arc::default(),
        })
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let avatar_viewport_id = egui::ViewportId::from_hash_of("Avatar window");
        let mut dt = 0.0;
        ctx.input(|i| {
            dt = i.stable_dt;

            if i.key_pressed(egui::Key::F5) {
                self.view = match self.view {
                    View::Overlay => View::Main,
                    _ => View::Overlay,
                }
            }

            if i.key_pressed(egui::Key::F1) {
                *self.second_window.lock().unwrap() = true;
            }

            if self.top_padding == 0.0 {
                self.top_padding = get_top_padding(i.viewport()).unwrap_or_default();
            }
        });

        let overlay = matches!(self.view, View::Overlay);

        ctx.send_viewport_cmd(ViewportCommand::Decorations(!overlay));
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(match overlay {
            true => WindowLevel::AlwaysOnTop,
            false => WindowLevel::Normal,
        }));
        ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(overlay));

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

        let mouth_open = Image::new(include_image!("../static/open_mouth.png"));
        let mouth_closed = Image::new(include_image!("../static/close_mouth.png"));

        let second_window = self.second_window.clone();
        if *self.second_window.lock().unwrap() {
            let builder = egui::ViewportBuilder::default().with_transparent(true);
            ctx.show_viewport_deferred(avatar_viewport_id, builder, move |ctx, _class| {
                egui::CentralPanel::default()
                    .frame(
                        Frame::default()
                            .fill(Color32::TRANSPARENT)
                            .shadow(egui::Shadow::NONE),
                    )
                    .show(ctx, |ui| {
                        ctx.input(|i| {
                            if i.viewport().close_requested() {
                                *second_window.lock().unwrap() = false;
                            }
                        });

                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| {
                                ui.add(
                                    Image::new(include_image!("../static/open_mouth.png"))
                                        .fit_to_fraction(Vec2::splat(1.0)),
                                );
                            },
                        )
                    });
            });
        }

        egui::CentralPanel::default()
            .frame(
                Frame::default()
                    .fill(Color32::from_rgb(30, 30, 30))
                    .outer_margin(match overlay {
                        true => egui::Margin {
                            top: self.top_padding,
                            left: 1.0,
                            ..Default::default()
                        },
                        false => egui::Margin::default(),
                    })
                    .inner_margin(egui::Margin::same(15.0)),
            )
            .show(ctx, |ui| {
                let draw_image = |ui| {
                    let image = match self.talking {
                        true => &mouth_open,
                        false => &mouth_closed,
                    };
                    let mut pos = Pos2::new(100.0, 20.0);
                    if matches!(self.view, View::Overlay) {
                        pos.x += 1.0;
                        pos.y += self.top_padding;
                    }
                    let size = Vec2::new(300.0, 300.0);

                    image.paint_at(ui, Rect::from_min_size(pos, size));
                };

                fn top_panel(ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
                    ui.horizontal(|ui| {
                        content(ui);
                    });
                    ui.add_space(10.0);
                }

                match self.view {
                    View::Main => {
                        draw_image(ui);

                        top_panel(ui, |ui| {
                            if ui.button("Settings").clicked() {
                                self.view = View::Settings;
                            }
                        });

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.set_max_width(100.0);
                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::Center)
                                    .with_main_justify(true),
                                |ui| {
                                    egui::containers::frame::Frame::none()
                                        .fill(Color32::RED)
                                        .show(ui, |ui| {
                                            ui.volume_meter(
                                                Vec2::new(50.0, 300.0),
                                                self.audio.rms(),
                                            );
                                        });

                                    ui.label(format!(
                                        "Talking: {}, time: {}",
                                        self.talking, self.time_release
                                    ));
                                },
                            );
                        });
                    }
                    View::Overlay => draw_image(ui),
                    View::Settings => {
                        top_panel(ui, |ui| {
                            if ui.button("Back").clicked() {
                                self.view = View::Main;
                            }
                        });

                        let mut current = self.current_device;
                        ComboBox::from_label("Select audio source")
                            .selected_text(&self.device_list[self.current_device])
                            .show_ui(ui, |ui| {
                                for (i, val) in self.device_list.iter().enumerate() {
                                    ui.selectable_value(&mut current, i, val);
                                }
                            });

                        if current != self.current_device {
                            match AudioInterface::new(Some(&self.device_list[current])) {
                                Ok(new_audio) => {
                                    self.audio = new_audio;
                                    self.current_device = current;
                                }
                                Err(e) => self.error_message = Some(e.to_string()),
                            }
                        }
                    }
                }

                let id = egui::Id::new("Error modal");
                let modal = egui::Modal::new(id);
                if let Some(e) = self.error_message.as_ref() {
                    let mut close = false;
                    let resp = modal.show(ctx, |ui| {
                        ui.label(RichText::new("Error").font(FontId::proportional(30.0)));
                        ui.add_space(5.0);
                        ui.vertical_centered_justified(|ui| ui.label(e));
                        ui.add_space(5.0);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Ok").clicked() {
                                close = true;
                            }
                        })
                    });
                    if close || resp.should_close() {
                        self.error_message = None;
                    }
                }
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}

fn get_top_padding(viewport: &ViewportInfo) -> Option<f32> {
    let inner = viewport.inner_rect?.top();
    let outer = viewport.outer_rect?.top();
    Some(inner - outer)
}
