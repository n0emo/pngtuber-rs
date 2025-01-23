use egui::{Color32, Pos2, Rect, Rounding, Ui, Vec2};

use crate::audio::gain_to_db;

pub trait AudioUiExt {
    fn volume_meter(&mut self, size: Vec2, gain: f32);
}

impl AudioUiExt for Ui {
    fn volume_meter(&mut self, size: Vec2, gain: f32) {
        let volume = gain_to_db(gain);
        let value = (60.0 + volume).max(0.0) / 60.0;

        egui::Frame::default()
            .stroke(egui::Stroke::new(2.0, Color32::WHITE))
            .show(self, |ui| {
                ui.set_min_size(size);
                let painter = ui.painter();
                let pos = ui.min_rect().min;

                let h = size.y * value;
                painter.rect_filled(
                    Rect::from_min_size(Pos2::new(pos.x, pos.y + size.y - h), Vec2::new(size.x, h)),
                    Rounding::ZERO,
                    Color32::GREEN,
                );
            });
    }
}
