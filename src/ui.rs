use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::{Axis, Config, CoordinateSystem, Hand, QuatObjectBundle, RenderingResources};

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiSet;

struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (settings_ui, objects_ui, sync_objects)
                .chain()
                .in_set(UiSet),
        );
    }
}

pub struct UiPlugins;

impl PluginGroup for UiPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        bevy::app::PluginGroupBuilder::start::<Self>()
            .add(UiPlugin)
            .add(EguiPlugin)
    }
}

#[derive(Component)]
pub struct QuatObject {
    quat: [String; 4],
    euler: [String; 3],
    look: [String; 3],
    up: Axis,
}

impl Default for QuatObject {
    fn default() -> Self {
        Self {
            quat: Default::default(),
            euler: Default::default(),
            look: Default::default(),
            up: Axis::Y,
        }
    }
}

pub fn settings_ui(mut cmd: Commands, mut ctx: EguiContexts, mut config_q: Query<&mut Config>) {
    let mut config = config_q.single_mut();

    egui::Window::new("Settings").show(ctx.ctx_mut(), |ui| {
        if ui.button("Add object").clicked() {
            cmd.spawn(QuatObjectBundle { ..default() });
        }

        ui.label("Coordinate System");
        egui::Grid::new("___CoordinatesGrid")
            .num_columns(3)
            .show(ui, |ui| {
                ui.label("Up");
                ui.label("Fw");
                ui.label("Hnd");
                ui.end_row();

                if ui.selectable_label(config.up == Axis::X, "X").clicked() {
                    if config.forward == Axis::X {
                        config.forward = config.up;
                    }
                    config.up = Axis::X;
                }
                if ui
                    .selectable_label(config.forward == Axis::X, "X")
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

                if ui.selectable_label(config.up == Axis::Y, "Y").clicked() {
                    if config.forward == Axis::Y {
                        config.forward = config.up;
                    }
                    config.up = Axis::Y;
                }
                if ui
                    .selectable_label(config.forward == Axis::Y, "Y")
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

                if ui.selectable_label(config.up == Axis::Z, "Z").clicked() {
                    if config.forward == Axis::Z {
                        config.forward = config.up;
                    }
                    config.up = Axis::Z;
                }
                if ui
                    .selectable_label(config.forward == Axis::Z, "Z")
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
    });
}

pub fn objects_ui(
    mut cmd: Commands,
    mut ctx: EguiContexts,
    coord_q: Query<&CoordinateSystem>,
    mut objects_q: Query<(
        Entity,
        &Name,
        &mut QuatObject,
        &mut Transform,
        &Handle<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let coord = coord_q.single();

    for (ent, name, mut obj, mut tf, material) in objects_q.iter_mut() {
        egui::Window::new(name.as_str()).show(ctx.ctx_mut(), |ui| {
            if ui.button("Delete").clicked() {
                cmd.entity(ent).despawn_recursive();
            }

            let material = materials.get_mut(material).unwrap();
            let mut rgb = match material.base_color.as_linear_rgba_f32() {
                [r, g, b, _] => [r, g, b],
            };

            ui.horizontal(|ui| {
                ui.label("Color: ");
                if egui::color_picker::color_edit_button_rgb(ui, &mut rgb).changed() {
                    material.base_color = Color::rgb_linear(rgb[0], rgb[1], rgb[2]);
                }
            });

            display_quaternion(ui, ent, &*coord, &mut obj, tf.reborrow());
            display_euler(ui, ent, &*coord, &mut obj, tf.reborrow());
            display_look(ui, ent, &*coord, &mut obj, tf.reborrow());
        });
    }
}

fn display_quaternion(
    ui: &mut egui::Ui,
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

    egui::CollapsingHeader::new("Quaternion")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new(ent.index().to_string() + "quat")
                .num_columns(2)
                .show(ui, |ui| {
                    display_field(ui, "W", &mut obj.quat[0]);
                    display_field(ui, "X", &mut obj.quat[1]);
                    display_field(ui, "Y", &mut obj.quat[2]);
                    display_field(ui, "Z", &mut obj.quat[3]);
                });
            if ui.button("Apply").clicked() {
                tf.rotation = convert_quaternion(
                    coord.user2internal,
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
                tf.rotation = convert_quaternion(
                    coord.user2internal,
                    Quat::from_xyzw(
                        obj.quat[1].parse().unwrap(),
                        obj.quat[2].parse().unwrap(),
                        obj.quat[3].parse().unwrap(),
                        obj.quat[0].parse().unwrap(),
                    ),
                );
            }
        });
}

fn display_euler(
    ui: &mut egui::Ui,
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
            tf.rotation = convert_quaternion(
                coord.user2internal,
                Quat::from_euler(
                    EulerRot::XYZ,
                    obj.euler[0].parse::<f32>().unwrap().to_radians(),
                    obj.euler[1].parse::<f32>().unwrap().to_radians(),
                    obj.euler[2].parse::<f32>().unwrap().to_radians(),
                ),
            )
            .normalize();
        }
    });
}

fn display_look(
    ui: &mut egui::Ui,
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
    });
}

fn convert_quaternion(mat: Mat3, mut quat: Quat) -> Quat {
    let converted = mat * quat.xyz();
    quat.x = converted.x;
    quat.y = converted.y;
    quat.z = converted.z;
    quat
}

fn sync_objects(
    mut cmd: Commands,
    coord_q: Query<Ref<CoordinateSystem>>,
    res: Res<RenderingResources>,
    mut objects_q: Query<(&mut QuatObject, Ref<Transform>)>,
    new_objects_q: Query<Entity, (With<QuatObject>, Without<Name>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let coord = coord_q.single();

    let mut i = 0;
    for (mut obj, tf) in objects_q.iter_mut() {
        i += 1;
        if !tf.is_changed() && !coord.is_changed() {
            continue;
        }

        let quat = convert_quaternion(coord.internal2user, tf.rotation);

        obj.quat[0] = quat.w.to_string();
        obj.quat[1] = quat.x.to_string();
        obj.quat[2] = quat.y.to_string();
        obj.quat[3] = quat.z.to_string();

        let (x, y, z) = quat.to_euler(EulerRot::XYZ);
        obj.euler[0] = x.to_degrees().to_string();
        obj.euler[1] = y.to_degrees().to_string();
        obj.euler[2] = z.to_degrees().to_string();

        let look = coord.internal2user * (tf.rotation * Vec3::NEG_Z);
        obj.look[0] = look.x.to_string();
        obj.look[1] = look.y.to_string();
        obj.look[2] = look.z.to_string();
    }

    for ent in new_objects_q.iter() {
        cmd.entity(ent).insert((
            MaterialMeshBundle {
                mesh: res.obj_mesh.clone(),
                material: materials.add(StandardMaterial {
                    depth_bias: -0.5,
                    unlit: true,
                    ..Color::BLACK.into()
                }),
                ..default()
            },
            Name::new(format!("Quat{}", i)),
        ));
        i += 1;
    }
}
