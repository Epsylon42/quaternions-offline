use bevy::prelude::*;
use bevy_egui::{EguiClipboard, EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use crate::geometry::{Axis, Config, CoordinateSystem, Hand, convert_rotation};

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiSet;

struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            (settings_ui, objects_ui).chain().in_set(UiSet),
        );
    }
}

pub struct UiPlugins;

impl PluginGroup for UiPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        bevy::app::PluginGroupBuilder::start::<Self>()
            .add(UiPlugin)
            .add(EguiPlugin::default())
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct QuatObject {
    pub quat: [String; 4],
    pub euler: [String; 3],
    pub look: [String; 3],
    pub mat: [String; 9],
    pub up: Axis,
}

impl Default for QuatObject {
    fn default() -> Self {
        Self {
            quat: default(),
            euler: default(),
            look: default(),
            mat: default(),
            up: Axis::Y,
        }
    }
}

fn clip_copy(clip: &mut EguiClipboard, data: &[String]) {
    clip.set_text(&data.join(","));
}

fn clip_paste(clip: &mut EguiClipboard, data: &mut [String]) {
    clip.get_text()
        .unwrap_or_default()
        .split(",")
        .map(str::trim)
        .map(String::from)
        .zip(data)
        .for_each(|(value, data)| *data = value);
}

fn transpose_mat_io(from: &[String; 9]) -> [String; 9] {
    let mut to: [String; 9] = default();
    for i in 0..3 {
        for j in 0..3 {
            to[i * 3 + j] = from[j * 3 + i].clone();
        }
    }
    to
}

pub fn settings_ui(mut cmd: Commands, mut ctx: EguiContexts, mut config_q: Query<&mut Config>) {
    let mut config = config_q.single_mut().unwrap();
    let ctx = ctx.ctx_mut().unwrap();

    egui::Window::new("Settings").show(ctx, |ui| {
        if ui.button("Add object").clicked() {
            cmd.spawn(QuatObject::default());
        }

        ui.label("Coordinate System");

        let x_label = || egui::RichText::new("X").color(egui::Color32::RED);
        let y_label = || egui::RichText::new("Y").color(egui::Color32::GREEN);
        let z_label = || egui::RichText::new("Z").color(egui::Color32::from_rgb(125, 125, 255));
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
            let response = ui.checkbox(
                &mut config.bypass_change_detection().keep_numerics,
                "keep numerics",
            );
            if response.clicked() {
                config.set_changed();
            }
        })
    });
}

pub fn objects_ui(
    mut cmd: Commands,
    mut ctx: EguiContexts,
    mut clip: ResMut<EguiClipboard>,
    coord_q: Query<&CoordinateSystem>,
    mut objects_q: Query<(
        Entity,
        &mut Name,
        &mut QuatObject,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let coord = coord_q.single().unwrap();
    let ctx = ctx.ctx_mut().unwrap();

    for (ent, mut name, mut obj, mut tf, material) in objects_q.iter_mut() {
        egui::Window::new(name.as_str())
            .id(egui::Id::new(ent.index()))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        cmd.entity(ent).despawn();
                    }

                    if ui.button("Reset input fields").clicked() {
                        // sync_objects will set input fields to the current values
                        tf.set_changed();
                    }
                });

                ui.collapsing("Options", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name: ");
                        name.mutate(|name| {
                            ui.add(egui::TextEdit::singleline(name).desired_width(100.0));
                        });
                    });
                    ui.horizontal(|ui| {
                        let material = materials.get_mut(material).unwrap();
                        let mut rgb = LinearRgba::from(material.base_color).to_f32_array_no_alpha();
                        ui.label("Color: ");
                        if egui::color_picker::color_edit_button_rgb(ui, &mut rgb).changed() {
                            material.base_color = LinearRgba::rgb(rgb[0], rgb[1], rgb[2]).into();
                        }
                    });
                });

                egui::CollapsingHeader::new("Values")
                    .default_open(true)
                    .show(ui, |ui| {
                        display_quaternion(ui, &mut *clip, ent, &*coord, &mut obj, tf.reborrow());
                        display_matrix(ui, &mut *clip, ent, &*coord, &mut obj, tf.reborrow());
                        display_euler(ui, &mut *clip, ent, &*coord, &mut obj, tf.reborrow());
                        display_look(ui, &mut *clip, ent, &*coord, &mut obj, tf.reborrow());
                    });
            });
    }
}

fn display_quaternion(
    ui: &mut egui::Ui,
    clip: &mut EguiClipboard,
    ent: Entity,
    coord: &CoordinateSystem,
    obj: &mut QuatObject,
    mut tf: Mut<Transform>,
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

    egui::CollapsingHeader::new("Quaternion").show(ui, |ui| {
        egui::Grid::new(ent.index().to_string() + "quat")
            .num_columns(2)
            .show(ui, |ui| {
                display_field(ui, "W", &mut obj.quat[0]);
                display_field(ui, "X", &mut obj.quat[1]);
                display_field(ui, "Y", &mut obj.quat[2]);
                display_field(ui, "Z", &mut obj.quat[3]);
            });
        if ui.button("Apply").clicked() {
            tf.rotation = convert_rotation(
                coord,
                Quat::from_xyzw(
                    obj.quat[1].parse().unwrap(),
                    obj.quat[2].parse().unwrap(),
                    obj.quat[3].parse().unwrap(),
                    obj.quat[0].parse().unwrap(),
                ),
            )
            .normalize();
        }
        if ui.button("Apply without normalization").clicked() {
            tf.rotation = convert_rotation(
                coord,
                Quat::from_xyzw(
                    obj.quat[1].parse().unwrap(),
                    obj.quat[2].parse().unwrap(),
                    obj.quat[3].parse().unwrap(),
                    obj.quat[0].parse().unwrap(),
                ),
            );
        }

        ui.horizontal(|ui| {
            if ui.button("Copy").clicked() {
                clip_copy(clip, &obj.quat);
            }
            if ui.button("Paste").clicked() {
                clip_paste(clip, &mut obj.quat);
            }
        });
    });
}

fn display_euler(
    ui: &mut egui::Ui,
    clip: &mut EguiClipboard,
    ent: Entity,
    coord: &CoordinateSystem,
    obj: &mut QuatObject,
    mut tf: Mut<Transform>,
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

    ui.collapsing("Euler angles (XYZ)", |ui| {
        egui::Grid::new(ent.index().to_string() + "euler")
            .num_columns(2)
            .show(ui, |ui| {
                display_field(ui, "X", &mut obj.euler[0]);
                display_field(ui, "Y", &mut obj.euler[1]);
                display_field(ui, "Z", &mut obj.euler[2]);
            });
        if ui.button("Apply").clicked() {
            tf.rotation = convert_rotation(
                coord,
                Quat::from_euler(
                    EulerRot::XYZ,
                    obj.euler[0].parse::<f32>().unwrap().to_radians(),
                    obj.euler[1].parse::<f32>().unwrap().to_radians(),
                    obj.euler[2].parse::<f32>().unwrap().to_radians(),
                ),
            )
            .normalize();
        }

        ui.horizontal(|ui| {
            if ui.button("Copy").clicked() {
                clip_copy(clip, &obj.euler);
            }
            if ui.button("Paste").clicked() {
                clip_paste(clip, &mut obj.euler);
            }
        });
    });
}

fn display_look(
    ui: &mut egui::Ui,
    clip: &mut EguiClipboard,
    ent: Entity,
    coord: &CoordinateSystem,
    obj: &mut QuatObject,
    mut tf: Mut<Transform>,
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

    ui.collapsing("Look", |ui| {
        egui::Grid::new(ent.index().to_string() + "look")
            .num_columns(2)
            .show(ui, |ui| {
                display_field(ui, "X", &mut obj.look[0]);
                display_field(ui, "Y", &mut obj.look[1]);
                display_field(ui, "Z", &mut obj.look[2]);
                ui.label("Up");
                ui.horizontal(|ui| {
                    if ui.selectable_label(obj.up == Axis::X, "X").clicked() {
                        obj.up = Axis::X;
                    }
                    if ui.selectable_label(obj.up == Axis::Y, "Y").clicked() {
                        obj.up = Axis::Y;
                    }
                    if ui.selectable_label(obj.up == Axis::Z, "Z").clicked() {
                        obj.up = Axis::Z;
                    }
                });
            });
        if ui.button("Apply").clicked() {
            tf.look_to(
                coord.user2internal
                    * Vec3::new(
                        obj.look[0].parse().unwrap(),
                        obj.look[1].parse().unwrap(),
                        obj.look[2].parse().unwrap(),
                    ),
                obj.up.to_vec(),
            );
        }

        ui.horizontal(|ui| {
            if ui.button("Copy").clicked() {
                clip_copy(clip, &obj.look);
            }
            if ui.button("Paste").clicked() {
                clip_paste(clip, &mut obj.look);
            }
        });
    });
}

fn display_matrix(
    ui: &mut egui::Ui,
    clip: &mut EguiClipboard,
    ent: Entity,
    coord: &CoordinateSystem,
    obj: &mut QuatObject,
    mut tf: Mut<Transform>,
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
                display_field(ui, &mut obj.mat[0]);
                display_field(ui, &mut obj.mat[1]);
                display_field(ui, &mut obj.mat[2]);
                ui.end_row();
                display_field(ui, &mut obj.mat[3]);
                display_field(ui, &mut obj.mat[4]);
                display_field(ui, &mut obj.mat[5]);
                ui.end_row();
                display_field(ui, &mut obj.mat[6]);
                display_field(ui, &mut obj.mat[7]);
                display_field(ui, &mut obj.mat[8]);
                ui.end_row();
            });

        if ui.button("Apply").clicked() {
            let mut parsed = [0.0; 9];
            for (from, to) in obj.mat.iter().zip(&mut parsed) {
                *to = from.parse::<f32>().unwrap();
            }
            tf.rotation = convert_rotation(coord, Quat::from_mat3(&Mat3::from_cols_array(&parsed)))
                .normalize();
        }

        egui::Grid::new(ent.index().to_string() + "mat_io")
            .num_columns(2)
            .min_col_width(60.0)
            .show(ui, |ui| {
                if ui.button("Copy RM").clicked() {
                    clip_copy(clip, &obj.mat);
                }
                if ui.button("Copy CM").clicked() {
                    clip_copy(clip, &transpose_mat_io(&obj.mat));
                }
                ui.end_row();

                if ui.button("Paste RM").clicked() {
                    clip_paste(clip, &mut obj.mat);
                }
                if ui.button("Paste CM").clicked() {
                    let mut tmp: [String; 9] = default();
                    clip_paste(clip, &mut tmp);
                    obj.mat = transpose_mat_io(&tmp);
                }
                ui.end_row();
            });
    });
}
