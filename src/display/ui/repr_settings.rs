use bevy::prelude::*;
use bevy_egui::egui;

use super::*;
use crate::display::representation as repr;

pub fn repr_settings_ui(
    is_always_on: bool,
    ui: &mut egui::Ui,
    repr: &mut repr::ReprSettings,
    computed: &repr::ComputedRepresentation,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        changed |= overridable_field(
            is_always_on,
            ui,
            "Color: ",
            &mut repr.color,
            &computed.color,
            |ui, color| {
                let mut rgb = color.to_f32_array_no_alpha();
                if egui::color_picker::color_edit_button_rgb(ui, &mut rgb).changed() {
                    *color = LinearRgba::from_f32_array_no_alpha(rgb).into();
                    true
                } else {
                    false
                }
            },
        );
    });
    ui.horizontal(|ui| {
        changed |= overridable_field(
            is_always_on,
            ui,
            "Length: ",
            &mut repr.length,
            &computed.length,
            |ui, length| {
                let widget = egui::DragValue::new(length)
                    .speed(SCROLL_SPEED_POS)
                    .range(0.01..=f32::INFINITY);
                ui.add(widget).changed()
            },
        );
    });
    ui.horizontal(|ui| {
        changed |= overridable_field(
            is_always_on,
            ui,
            "Scale: ",
            &mut repr.scale,
            &computed.scale,
            |ui, scale| {
                let widget = egui::DragValue::new(scale)
                    .speed(SCROLL_SPEED_POS)
                    .range(0.01..=f32::INFINITY);
                ui.add(widget).changed()
            },
        );
    });
    // ui.horizontal(|ui| {
    //     changed |= overridable_field(
    //         is_always_on,
    //         ui,
    //         "Pos: ",
    //         &mut repr.pos_mode,
    //         &computed.pos_mode,
    //         |ui, mode| {
    //             if ui
    //                 .selectable_label(*mode == geometry::PositionMode::Flat, "flat")
    //                 .clicked()
    //             {
    //                 *mode = geometry::PositionMode::Flat;
    //                 return true;
    //             }
    //             if ui
    //                 .selectable_label(*mode == geometry::PositionMode::Rotated, "rotated")
    //                 .clicked()
    //             {
    //                 *mode = geometry::PositionMode::Rotated;
    //                 return true;
    //             }
    //             false
    //         },
    //     );
    // });

    changed
}

fn overridable_field<T, F>(
    is_always_on: bool,
    ui: &mut egui::Ui,
    name: &str,
    editable_value: &mut Option<T>,
    background_value: &T,
    cb: F,
) -> bool
where
    T: Clone,
    F: FnOnce(&mut egui::Ui, &mut T) -> bool,
{
    let mut enabled = editable_value.is_some();
    let mut changed = false;

    if is_always_on {
        ui.label(name);
        if editable_value.is_none() {
            *editable_value = Some(background_value.clone());
        }
    } else {
        ui.checkbox(&mut enabled, name);
        if enabled != editable_value.is_some() {
            changed = true;
            if !enabled {
                editable_value.take();
            } else {
                *editable_value = Some(background_value.clone());
            }
        }
    }

    ui.add_enabled_ui(enabled, |ui| {
        let mut dummy_value = if enabled {
            None
        } else {
            Some(background_value.clone())
        };
        changed |= cb(
            ui,
            editable_value.as_mut().or(dummy_value.as_mut()).unwrap(),
        );
    });

    changed
}
