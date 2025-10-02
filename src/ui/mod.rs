use bevy::{ecs::query::QueryData, prelude::*};
use bevy_egui::{EguiClipboard, EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use crate::{
    repr,
    conversion as conv,
    group::{GroupedObjects, InGroup, InGroupDisplaySettings},
    geometry::{ApplyTransformCommand, Axis, Hand, PositionMode},
};

mod common;
mod arrow;
mod settings;
mod repr_settings;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiSet;

struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().spawn(ConfigIO::default());

        app.add_systems(
            EguiPrimaryContextPass,
            (settings::system_settings_ui, system_arrows_ui).chain().in_set(UiSet),
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
#[require(repr::ReprSettings)]
pub struct ConfigIO {
    pub up: Axis,
    pub forward: Axis,
    pub up_sign: f32,
    pub forward_sign: f32,
    pub hand: Hand,
    /// if true, changing coordinate system will preserve numeric values of the quaternion
    /// instead of its direction in the internal coordinate system
    pub keep_numbers: bool,

    pub position_mode: PositionMode,
    pub positions_scale: f32,
}

impl Default for ConfigIO {
    fn default() -> Self {
        Self {
            up: Axis::Y,
            up_sign: 1.0,
            forward: Axis::Z,
            forward_sign: -1.0,
            hand: Hand::Right,
            keep_numbers: false,
            position_mode: default(),
            positions_scale: 1.0,
        }
    }
}

const SCROLL_SPEED_POS: f32 = 0.01;
const SCROLL_SPEED_DEG: f32 = 0.1;
const SCROLL_SPEED_SCALE: f32 = 0.01;


#[derive(QueryData)]
#[query_data(mutable)]
struct ArrowsQuery<'a> {
    ent: Entity,
    name: &'a mut Name,
    arrow: &'a mut crate::arrow::ArrowIO,
    repr: &'a mut repr::ReprSettings,
    computed: &'a repr::ComputedRepresentation,
    in_group: Option<&'a InGroup>,
    in_group_display: Option<&'a mut InGroupDisplaySettings>,
}


fn system_arrows_ui(
    mut cmd: Commands,
    mut ctx: EguiContexts,
    mut clip: ResMut<EguiClipboard>,
    config_q: Query<Entity, With<ConfigIO>>,
    mut groups_q: Query<
        (
            Entity,
            &mut Name,
            &mut crate::group::GroupIO,
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
                    arrow::arrow_ui(&mut cmd, ui, &mut clip, arrow, &mut tf_events);
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
                        arrow::arrow_ui(&mut cmd, ui, &mut clip, arrow, &mut tf_events);
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

                    if repr_settings::repr_settings_ui(false, ui, repr.bypass_change_detection(), &computed) {
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
                                arrow::arrow_ui(&mut cmd, ui, &mut clip, arrow, &mut tf_events);
                            });
                    }

                }
            });
    }
}
