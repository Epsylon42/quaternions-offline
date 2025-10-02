use bevy::{ecs::query::QueryData, prelude::*};
use bevy_egui::{EguiClipboard, EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use super::{conversion as conv, representation as repr};
use crate::{
    display::representation::{GroupedObjects, InGroup, InGroupDisplaySettings},
    geometry::{self, ApplyTransformCommand, Axis, Hand, PositionMode},
};

mod arrow;
mod repr_settings;

use arrow::arrow_ui;
use repr_settings::repr_settings_ui;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiSet;

struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                repr::sync_repr,
                repr::sync_display_arrow,
                init_group_names,
                init_arrow_names,
                sync_arrowio,
            )
                .in_set(UiSet),
        );
        app.add_systems(
            EguiPrimaryContextPass,
            (settings_ui_system, arrows_ui_system).chain().in_set(UiSet),
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
#[require(repr::ReprSettings, Transform, Visibility)]
pub struct Config {
    pub up: Axis,
    pub forward: Axis,
    pub up_sign: f32,
    pub forward_sign: f32,
    pub hand: Hand,
    /// if true, changing coordinate system will preserve numeric values of the quaternion
    /// instead of its direction in the internal coordinate system
    pub keep_numerics: bool,

    pub positions_scale: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            up: Axis::Y,
            up_sign: 1.0,
            forward: Axis::Z,
            forward_sign: -1.0,
            hand: Hand::Right,
            keep_numerics: false,
            positions_scale: 1.0,
        }
    }
}

#[derive(Component)]
#[require(Transform, Visibility, crate::geometry::UserTransform)]
pub struct ArrowIO {
    pub pos: Vec3,
    pub quat: [String; 4],
    pub euler: Vec3,
    pub mat: [String; 9],
    // pub look: [String; 3],
    // pub up: Axis,
}

impl Default for ArrowIO {
    fn default() -> Self {
        Self {
            pos: default(),
            quat: default(),
            euler: default(),
            mat: default(),
            // look: default(),
            // up: Axis::Y,
        }
    }
}

fn sync_arrowio(
    mut arrow_q: Query<
        (&mut ArrowIO, &repr::ComputedRepresentation, Ref<geometry::UserTransform>),
        Changed<geometry::UserTransform>,
    >,
) {
    for (mut arrow, repr, utf) in arrow_q.iter_mut() {
        let tf = &utf.0;
        arrow.pos = match repr.pos_mode {
            PositionMode::Flat => tf.translation,
            PositionMode::Rotated => tf.rotation.inverse() * tf.translation,
        };

        let quat = tf.rotation;
        arrow.quat = conv::quat_to_strings(quat, conv::QuatStrMode::WXYZ);

        let (x, y, z) = quat.to_euler(EulerRot::XYZ);
        arrow.euler = Vec3::new(x, y, z).map(f32::to_degrees);

        let mat = Mat3::from_quat(quat);
        arrow.mat = conv::mat3_to_strings(&mat, conv::MatStrMode::RowMajor);
    }
}

#[derive(Default)]
struct ArrowsCreatedCounter(usize);

fn init_arrow_names(
    mut cmd: Commands,
    arrow_q: Query<Entity, (With<crate::objects::Arrow>, Without<Name>)>,
    mut counter: Local<ArrowsCreatedCounter>,
) {
    for ent in arrow_q.iter() {
        counter.0 += 1;
        cmd.entity(ent)
            .insert(Name::new(format!("Arrow {}", counter.0)));
    }
}

#[derive(Default)]
struct GroupsCreatedCounter(usize);

fn init_group_names(
    mut cmd: Commands,
    groups_q: Query<Entity, (With<crate::objects::Group>, Without<Name>)>,
    mut counter: Local<GroupsCreatedCounter>,
) {
    for ent in groups_q.iter() {
        counter.0 += 1;
        cmd.entity(ent)
            .insert(Name::new(format!("Group {}", counter.0)));
    }
}

fn clip_copy(clip: &mut EguiClipboard, data: &[String]) {
    clip.set_text(&data.join(","));
}

fn clip_paste(clip: &mut EguiClipboard, data: &mut [String]) {
    clip.get_text()
        .unwrap_or_default()
        .split(",")
        .chain(std::iter::repeat("0"))
        .map(str::trim)
        .map(String::from)
        .zip(data)
        .for_each(|(value, data)| *data = value);
}

const SCROLL_SPEED_POS: f32 = 0.01;
const SCROLL_SPEED_DEG: f32 = 0.1;
const SCROLL_SPEED_SCALE: f32 = 0.01;

fn settings_ui_system(
    mut cmd: Commands,
    mut ctx: EguiContexts,
    mut config_q: Query<(
        Entity,
        &mut Config,
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
            if repr_settings_ui(true, ui, repr.bypass_change_detection(), &computed) {
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
                    let response = ui.checkbox(
                        &mut config.bypass_change_detection().keep_numerics,
                        "keep numerics",
                    );
                    if response.clicked() {
                        config.set_changed();
                    }
                });
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
        })
    });
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ArrowsQuery<'a> {
    ent: Entity,
    name: &'a mut Name,
    arrow: &'a mut ArrowIO,
    repr: &'a mut repr::ReprSettings,
    computed: &'a repr::ComputedRepresentation,
    in_group: Option<&'a InGroup>,
    in_group_display: Option<&'a mut InGroupDisplaySettings>,
}

#[derive(Component, Default)]
pub struct GroupIO {
    selected_object: Option<Entity>,
}

fn arrows_ui_system(
    mut cmd: Commands,
    mut ctx: EguiContexts,
    mut clip: ResMut<EguiClipboard>,
    config_q: Query<Entity, With<Config>>,
    mut groups_q: Query<
        (
            Entity,
            &mut Name,
            &mut GroupIO,
            Option<&GroupedObjects>,
            &mut repr::ReprSettings,
            &repr::ComputedRepresentation,
        ),
        With<crate::objects::Group>
    >,

    mut arrows_q: Query<ArrowsQuery, Without<crate::objects::Group>>,

    mut tf_events: EventWriter<ApplyTransformCommand>,
) {
    let ctx = ctx.ctx_mut().unwrap();
    let config_ent = config_q.single().unwrap();

    for arrow in arrows_q.iter_mut() {
        if arrow.in_group.is_none() || arrow.in_group.unwrap().0 == config_ent {
            egui::Window::new(arrow.name.as_str())
                .id(egui::Id::new(arrow.ent.index()))
                .show(ctx, |ui| {
                    arrow_ui(&mut cmd, ui, &mut clip, arrow, &mut tf_events);
                });
        }
    }


    for (ent, mut name, mut group, grouped, mut repr, computed) in groups_q.iter_mut() {
        if let Some(selected) = group.selected_object {
            if !arrows_q.contains(selected) {
                group.selected_object = None;
            }
        }

        for arrow_ent in grouped.into_iter().flatten() {
            let arrow = arrows_q.get_mut(arrow_ent).unwrap();
            if arrow.in_group_display.as_ref().unwrap().popped_out {
                egui::Window::new(arrow.name.as_str())
                    .id(egui::Id::new(arrow.ent.index()))
                    .show(ctx, |ui| {
                        arrow_ui(&mut cmd, ui, &mut clip, arrow, &mut tf_events);
                    });
            }
        }

        egui::Window::new(name.as_str())
            .id(egui::Id::new(ent.index()))
            .show(ctx, |ui| {
                if ui.button("Add Arrow").clicked() {
                    cmd.spawn((crate::objects::Arrow, InGroup(ent), InGroupDisplaySettings::default()));
                }

                if ui.button("Delete").clicked() {
                    cmd.entity(ent).despawn();
                }

                ui.collapsing("Settings", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name: ");
                        name.mutate(|name| {
                            ui.add(egui::TextEdit::singleline(name).desired_width(100.0));
                        });
                    });

                    if repr_settings_ui(false, ui, repr.bypass_change_detection(), &computed) {
                        repr.set_changed();
                    }
                });

                egui::CollapsingHeader::new("Arrows")
                    .default_open(true)
                    .show(ui, |ui| {
                        for arrow_ent in grouped.into_iter().flatten() {
                            let arrow = arrows_q.get(arrow_ent).unwrap();
                            ui.add_enabled_ui(!arrow.in_group_display.unwrap().popped_out, |ui| {
                                let mut label = arrow.name.to_string();
                                if arrow.in_group_display.unwrap().popped_out {
                                    label += " ->";
                                }

                                let this_selected = group.selected_object == Some(arrow_ent);
                                let response = ui.selectable_label(this_selected, label);
                                if response.clicked() {
                                    group.selected_object = Some(arrow_ent);
                                }
                                if this_selected && response.clicked_by(egui::PointerButton::Secondary) {
                                    group.selected_object = None;
                                }
                            });
                        }
                    });

                if let Some(selected) = group.selected_object {
                    let arrow = arrows_q.get_mut(selected).unwrap();
                    if !arrow.in_group_display.as_ref().unwrap().popped_out {
                        egui::CollapsingHeader::new("Selected Arrow")
                            .default_open(true)
                            .show(ui, |ui| {
                                arrow_ui(&mut cmd, ui, &mut clip, arrow, &mut tf_events);
                            });
                    }

                }
            });
    }
}
