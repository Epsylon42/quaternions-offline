use std::collections::VecDeque;

use bevy::prelude::*;

use crate::geometry::PositionMode;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
#[relationship(relationship_target = GroupedObjects)]
pub struct InGroup(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = InGroup, linked_spawn)]
pub struct GroupedObjects(Vec<Entity>);

#[derive(Component, Default, Clone, Copy)]
pub struct InGroupDisplaySettings {
    pub popped_out: bool
}

impl<'a> IntoIterator for &'a GroupedObjects {
    type Item = Entity;
    type IntoIter = std::iter::Copied<<&'a Vec<Entity> as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

#[derive(Component, Clone)]
pub struct ComputedRepresentation {
    pub color: LinearRgba,
    pub pos_mode: PositionMode,
    pub length: f32,
    pub scale: f32,
}

impl Default for ComputedRepresentation {
    fn default() -> Self {
        ComputedRepresentation {
            color: LinearRgba::BLACK,
            pos_mode: default(),
            length: 1.0,
            scale: 1.0,
        }
    }
}

#[derive(Component, Default)]
#[require(ComputedRepresentation)]
pub struct ReprSettings {
    pub color: Option<LinearRgba>,
    pub pos_mode: Option<PositionMode>,
    pub length: Option<f32>,
    pub scale: Option<f32>,
}

pub fn sync_repr(
    mut values_q: Query<(Entity, Ref<ReprSettings>, Mut<ComputedRepresentation>)>,
    hierarchy_q: Query<(Option<&InGroup>, Option<&GroupedObjects>), With<ComputedRepresentation>>,
) {
    let sync = |repr: &ReprSettings,
                computed: &mut ComputedRepresentation,
                parent_value: &ComputedRepresentation| {
        computed.color = repr.color.unwrap_or(parent_value.color);
        computed.pos_mode = repr.pos_mode.unwrap_or(parent_value.pos_mode);
        computed.length = repr.length.unwrap_or(parent_value.length);
        computed.scale = repr.scale.unwrap_or(parent_value.scale);
    };

    let mut queue = VecDeque::new();

    for (ent, repr, computed) in values_q.iter() {
        if repr.is_changed() || computed.is_changed() {
            queue.push_back(ent);
        }
    }

    while let Some(ent) = queue.pop_front() {
        let (child_of, children) = hierarchy_q.get(ent).unwrap();

        let parent_value = match child_of {
            Some(child_of) => {
                let (_, _, computed) = values_q.get(child_of.0).unwrap();
                computed.clone()
            }

            None => default()
        };

        if let Ok((_, repr, mut computed)) = values_q.get_mut(ent) {
            sync(&*repr, &mut *computed, &parent_value);
            queue.extend(children.into_iter().flatten());
        }
    }
}

pub fn sync_display_arrow(
    mut cmd: Commands,
    arrow_q: Query<
        (Entity, &ComputedRepresentation),
        (With<crate::objects::Arrow>, Changed<ComputedRepresentation>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (ent, repr) in arrow_q.iter() {
        let material = materials.add(StandardMaterial {
            depth_bias: -0.5,
            unlit: true,
            ..Color::from(repr.color).into()
        });

        cmd.entity(ent)
            .despawn_related::<Children>()
            .with_children(|cmd| {
                crate::mesh::spawn_arrow(&mut *meshes, cmd, repr.length, repr.scale, material);
            });
    }
}
