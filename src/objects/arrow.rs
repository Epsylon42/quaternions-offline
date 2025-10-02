use bevy::prelude::*;

use crate::conversion as conv;
use crate::{ repr, geometry::{self, PositionMode} };

#[derive(Component)]
#[require(
    ArrowIO,
    repr::ReprSettings
)]
pub struct Arrow;

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

#[derive(Default)]
pub struct ArrowsCreatedCounter(usize);

pub fn system_init_arrow_names(
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

pub fn system_sync_arrow_io(
    mut arrow_q: Query<
        (&mut ArrowIO, &repr::ComputedRepresentation, Ref<geometry::UserTransform>),
        Or<(Changed<geometry::UserTransform>, Changed<repr::ComputedRepresentation>)>,
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

pub fn system_sync_display_arrow(
    mut cmd: Commands,
    arrow_q: Query<
        (Entity, &repr::ComputedRepresentation),
        (With<Arrow>, Changed<repr::ComputedRepresentation>),
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
