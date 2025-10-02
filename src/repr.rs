use std::collections::VecDeque;

use bevy::prelude::*;

use crate::geometry::PositionMode;
use crate::group::{InGroup, GroupedObjects};

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

pub fn system_propagate_repr_settings(
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
