use bevy::prelude::*;

use crate::repr;

#[derive(Component)]
#[require(
    GroupIO,
    repr::ReprSettings
)]
pub struct Group;

#[derive(Component, Default)]
pub struct GroupIO {
    pub selected_object: Option<Entity>,
}

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

#[derive(Default)]
pub struct GroupsCreatedCounter(usize);

pub fn system_init_group_names(
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

