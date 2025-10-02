use bevy::prelude::*;
use bevy_egui::{egui, EguiClipboard};

use crate::arrow::ArrowIO;
use super::{*, common::*};

pub fn arrow_ui(
    cmd: &mut Commands,
    ui: &mut egui::Ui,
    clip: &mut EguiClipboard,
    ArrowsQueryItem { ent, mut name, mut arrow, mut repr, computed, in_group_display, .. }: ArrowsQueryItem,
    mut events: &mut EventWriter<ApplyTransformCommand>,
) {
    ui.horizontal(|ui| {
        if let Some(mut igd) = in_group_display {
            let text = if igd.popped_out { "Return to group" } else { "Pop out" };
            if ui.button(text).clicked() {
                igd.popped_out = !igd.popped_out;
            }
        }

        if ui.button("Delete").clicked() {
            cmd.entity(ent).despawn();
        }

        if ui.button("Reset input fields").clicked() {
            // sync_objects will set input fields to the current values
            events.write(ApplyTransformCommand::recompute(ent));
        }
    });

    ui.collapsing("Options", |ui| {
        ui.horizontal(|ui| {
            ui.label("Name: ");
            name.mutate(|name| {
                ui.add(egui::TextEdit::singleline(name).desired_width(100.0));
            });
        });

        if repr_settings::repr_settings_ui(false, ui, repr.bypass_change_detection(), &*computed) {
            repr.set_changed();
        }
    });

    egui::CollapsingHeader::new("Values")
        .default_open(true)
        .show(ui, |ui| {
            display_position(ui, &mut *clip, ent, &mut arrow, &mut events);
            display_quaternion(ui, &mut *clip, ent, &mut arrow, &mut events);
            display_matrix(ui, &mut *clip, ent, &mut arrow, &mut events);
            display_euler(ui, &mut *clip, ent, &mut arrow, &mut events);
            // display_look(ui, &mut *clip, ent, &*coord, &mut arrow, tf.reborrow());
        });
}

fn display_position(
    ui: &mut egui::Ui,
    clip: &mut EguiClipboard,
    ent: Entity,
    arrow: &mut ArrowIO,
    events: &mut EventWriter<ApplyTransformCommand>,
) {
    let display_field = |ui: &mut egui::Ui, name: &'static str, buf: &mut f32| -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(name);
            let widget = egui::DragValue::new(buf).speed(SCROLL_SPEED_POS);
            changed = ui.add(widget).changed();
        });
        changed
    };

    let mut changed = false;
    ui.collapsing("Position", |ui| {
        changed |= display_field(ui, "X", &mut arrow.pos[0]);
        changed |= display_field(ui, "Y", &mut arrow.pos[1]);
        changed |= display_field(ui, "Z", &mut arrow.pos[2]);

        ui.horizontal(|ui| {
            if ui.button("Copy").clicked() {
                let s = conv::vec_to_strings(arrow.pos);
                clip_copy(clip, &s);
                changed = true;
            }
            if ui.button("Paste").clicked() {
                let mut s: [String; 3] = default();
                clip_paste(clip, &mut s);
                arrow.pos = conv::strings_to_vec(&s);
                changed = true;
            }
        });
    });

    if changed {
        events.write(ApplyTransformCommand::pos(ent, arrow.pos));
    }
}

fn display_quaternion(
    ui: &mut egui::Ui,
    clip: &mut EguiClipboard,
    ent: Entity,
    arrow: &mut ArrowIO,
    events: &mut EventWriter<ApplyTransformCommand>,
) {
    let display_field = |ui: &mut egui::Ui, name: &'static str, buf: &mut String| {
        ui.label(name);
        let widget = egui::TextEdit::singleline(buf).desired_width(100.0);
        let response = ui.add(widget);
        if response.lost_focus() {
            *buf = buf.parse().unwrap_or(0.0).to_string();
        }
        ui.end_row();
    };

    ui.collapsing("Quaternion", |ui| {
        egui::Grid::new(ent.index().to_string() + "quat")
            .num_columns(2)
            .show(ui, |ui| {
                display_field(ui, "W", &mut arrow.quat[0]);
                display_field(ui, "X", &mut arrow.quat[1]);
                display_field(ui, "Y", &mut arrow.quat[2]);
                display_field(ui, "Z", &mut arrow.quat[3]);
            });
        if ui.button("Apply").clicked() {
            events.write(ApplyTransformCommand::rot_quat(
                ent,
                conv::strings_to_quat(&arrow.quat, conv::QuatStrMode::WXYZ).normalize(),
            ));
        }
        if ui.button("Apply without normalization").clicked() {
            events.write(ApplyTransformCommand::rot_quat(
                ent,
                conv::strings_to_quat(&arrow.quat, conv::QuatStrMode::WXYZ),
            ));
        }

        ui.horizontal(|ui| {
            if ui.button("Copy").clicked() {
                clip_copy(clip, &arrow.quat);
            }
            if ui.button("Paste").clicked() {
                clip_paste(clip, &mut arrow.quat);
            }
        });
    });
}

fn display_euler(
    ui: &mut egui::Ui,
    clip: &mut EguiClipboard,
    ent: Entity,
    arrow: &mut ArrowIO,
    events: &mut EventWriter<ApplyTransformCommand>,
) {
    let display_field = |ui: &mut egui::Ui, name: &'static str, buf: &mut f32| -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(name);
            let widget = egui::DragValue::new(buf).speed(SCROLL_SPEED_DEG);
            changed = ui.add(widget).changed();
        });
        changed
    };

    let mut changed = false;
    ui.collapsing("Euler angles (XYZ)", |ui| {
        changed |= display_field(ui, "X", &mut arrow.euler[0]);
        changed |= display_field(ui, "Y", &mut arrow.euler[1]);
        changed |= display_field(ui, "Z", &mut arrow.euler[2]);

        ui.horizontal(|ui| {
            if ui.button("Copy").clicked() {
                let s = conv::vec_to_strings(arrow.euler);
                clip_copy(clip, &s);
                changed = true;
            }
            if ui.button("Paste").clicked() {
                let mut s: [String; 3] = default();
                clip_paste(clip, &mut s);
                arrow.euler = conv::strings_to_vec(&s);
                changed = true;
            }
        });
    });

    if changed {
        events.write(ApplyTransformCommand::rot_euler(
            ent,
            arrow.euler.map(f32::to_radians),
        ));
    }
}

fn display_matrix(
    ui: &mut egui::Ui,
    clip: &mut EguiClipboard,
    ent: Entity,
    arrow: &mut ArrowIO,
    events: &mut EventWriter<ApplyTransformCommand>,
) {
    let display_field = |ui: &mut egui::Ui, buf: &mut String| {
        let widget = egui::TextEdit::singleline(buf);
        let response = ui.add(widget);
        if response.lost_focus() {
            *buf = buf.parse().unwrap_or(0.0).to_string();
        }
    };

    ui.collapsing("Rotation Matrix", |ui| {
        egui::Grid::new(ent.index().to_string() + "mat")
            .num_columns(3)
            .min_col_width(60.0)
            .max_col_width(60.0)
            .show(ui, |ui| {
                display_field(ui, &mut arrow.mat[0]);
                display_field(ui, &mut arrow.mat[1]);
                display_field(ui, &mut arrow.mat[2]);
                ui.end_row();
                display_field(ui, &mut arrow.mat[3]);
                display_field(ui, &mut arrow.mat[4]);
                display_field(ui, &mut arrow.mat[5]);
                ui.end_row();
                display_field(ui, &mut arrow.mat[6]);
                display_field(ui, &mut arrow.mat[7]);
                display_field(ui, &mut arrow.mat[8]);
                ui.end_row();
            });

        if ui.button("Apply").clicked() {
            events.write(ApplyTransformCommand::rot_mat(
                ent,
                conv::strings_to_mat3(&arrow.mat, conv::MatStrMode::RowMajor),
            ));
        }

        egui::Grid::new(ent.index().to_string() + "mat_io")
            .num_columns(2)
            .min_col_width(60.0)
            .show(ui, |ui| {
                if ui.button("Copy RM").clicked() {
                    clip_copy(clip, &arrow.mat);
                }
                if ui.button("Copy CM").clicked() {
                    clip_copy(clip, &conv::transpose_mat_io(&arrow.mat));
                }
                ui.end_row();

                if ui.button("Paste RM").clicked() {
                    clip_paste(clip, &mut arrow.mat);
                }
                if ui.button("Paste CM").clicked() {
                    let mut tmp: [String; 9] = default();
                    clip_paste(clip, &mut tmp);
                    arrow.mat = conv::transpose_mat_io(&tmp);
                }
                ui.end_row();
            });
    });
}
