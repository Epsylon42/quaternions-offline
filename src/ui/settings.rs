use super::*;

pub fn system_settings_ui(
    mut cmd: Commands,
    mut ctx: EguiContexts,
    mut config_q: Query<(
        Entity,
        &mut ConfigIO,
        &mut repr::ReprSettings,
        &repr::ComputedRepresentation,
    )>,
) {
    let (config_ent, mut config, mut repr, computed) = config_q.single_mut().unwrap();
    let ctx = ctx.ctx_mut().unwrap();

    egui::Window::new("Settings").show(ctx, |ui| {
        if ui.button("Add Group").clicked() {
            cmd.spawn((crate::objects::Group, InGroup(config_ent)));
        }

        if ui.button("Add Arrow").clicked() {
            cmd.spawn((crate::objects::Arrow, InGroup(config_ent)));
        }

        ui.collapsing("Default Settings", |ui| {
            if repr_settings::repr_settings_ui(true, ui, repr.bypass_change_detection(), &computed)
            {
                repr.set_changed();
            }
        });

        egui::CollapsingHeader::new("Coordinate System")
            .default_open(true)
            .show(ui, |ui| {
                let x_label = || egui::RichText::new("X").color(egui::Color32::RED);
                let y_label = || egui::RichText::new("Y").color(egui::Color32::GREEN);
                let z_label =
                    || egui::RichText::new("Z").color(egui::Color32::from_rgb(125, 125, 255));
                egui::Grid::new("___CoordinatesGrid")
                    .num_columns(3)
                    .show(ui, |ui| {
                        ui.label("Up");
                        ui.label("Fw");
                        ui.label("Hnd");
                        ui.end_row();

                        if ui
                            .selectable_label(config.up == Axis::X, x_label())
                            .clicked()
                        {
                            if config.forward == Axis::X {
                                config.forward = config.up;
                            }
                            config.up = Axis::X;
                        }
                        if ui
                            .selectable_label(config.forward == Axis::X, x_label())
                            .clicked()
                        {
                            if config.up == Axis::X {
                                config.up = config.forward;
                            }
                            config.forward = Axis::X;
                        }
                        if ui
                            .selectable_label(config.hand == Hand::Left, "L")
                            .clicked()
                        {
                            config.hand = Hand::Left;
                        }
                        ui.end_row();

                        if ui
                            .selectable_label(config.up == Axis::Y, y_label())
                            .clicked()
                        {
                            if config.forward == Axis::Y {
                                config.forward = config.up;
                            }
                            config.up = Axis::Y;
                        }
                        if ui
                            .selectable_label(config.forward == Axis::Y, y_label())
                            .clicked()
                        {
                            if config.up == Axis::Y {
                                config.up = config.forward;
                            }
                            config.forward = Axis::Y;
                        }
                        if ui
                            .selectable_label(config.hand == Hand::Right, "R")
                            .clicked()
                        {
                            config.hand = Hand::Right;
                        }
                        ui.end_row();

                        if ui
                            .selectable_label(config.up == Axis::Z, z_label())
                            .clicked()
                        {
                            if config.forward == Axis::Z {
                                config.forward = config.up;
                            }
                            config.up = Axis::Z;
                        }
                        if ui
                            .selectable_label(config.forward == Axis::Z, z_label())
                            .clicked()
                        {
                            if config.up == Axis::Z {
                                config.up = config.forward;
                            }
                            config.forward = Axis::Z;
                        }
                        ui.end_row();

                        if ui.selectable_label(config.up_sign < 0.0, "-").clicked() {
                            config.up_sign *= -1.0;
                        }
                        if ui
                            .selectable_label(config.forward_sign < 0.0, "-")
                            .clicked()
                        {
                            config.forward_sign *= -1.0;
                        }
                        ui.end_row();
                    });

                    ui.horizontal(|ui| {
                        ui.label("pos mode");
                        if ui
                            .selectable_label(config.position_mode == PositionMode::Flat, "flat")
                            .clicked()
                        {
                            config.position_mode = PositionMode::Flat;
                        }
                        if ui
                            .selectable_label(config.position_mode == PositionMode::Rotated, "rotated")
                            .clicked()
                        {
                            config.position_mode = PositionMode::Rotated;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("pos scale");
                        let widget =
                            egui::DragValue::new(&mut config.bypass_change_detection().positions_scale)
                                .range(0.00001..=f32::INFINITY)
                                .speed(SCROLL_SPEED_SCALE);
                        let response = ui.add(widget);
                        if response.changed() {
                            config.set_changed();
                        }
                    });

                    ui.horizontal(|ui| {
                        let response = ui.checkbox(
                            &mut config.bypass_change_detection().keep_numbers,
                            "keep numbers on change",
                        );
                        if response.clicked() {
                            config.set_changed();
                        }
                    });
                });
            });
}
