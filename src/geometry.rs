use crate::{ui, repr};
use bevy::prelude::*;

pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().spawn(CoordinateSystem::default());

        app.add_systems(
            Update,
            (
                system_sync_coordinates,
                system_sync_objects,
                system_process_transform_commands,
            )
                .chain()
                .after(ui::UiSet),
        )
        .add_event::<ApplyTransformCommand>();
    }
}

#[derive(Component)]
pub struct MainPlane;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn to_vec(self) -> Vec3 {
        match self {
            Axis::X => Vec3::X,
            Axis::Y => Vec3::Y,
            Axis::Z => Vec3::Z,
        }
    }

    // pub fn all() -> [Self; 3] {
    //     [Axis::X, Axis::Y, Axis::Z]
    // }

    pub fn name(&self) -> &'static str {
        match self {
            Axis::X => "X",
            Axis::Y => "Y",
            Axis::Z => "Z",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum PositionMode {
    #[default]
    Flat,
    Rotated,
}

#[derive(Component)]
pub struct CoordinateSystem {
    pub user2internal: Mat3,
    pub internal2user: Mat3,
    pub position_mode: PositionMode,
    pub positions_scale: f32,
}

impl Default for CoordinateSystem {
    fn default() -> Self {
        Self {
            user2internal: Mat3::IDENTITY,
            internal2user: Mat3::IDENTITY,
            position_mode: default(),
            positions_scale: 1.0,
        }
    }
}

pub fn convert_rotation(mat: &Mat3, mut quat: Quat) -> Quat {
    let converted = *mat * quat.xyz();
    quat.x = converted.x;
    quat.y = converted.y;
    quat.z = converted.z;
    quat
}

pub fn convert_position(
    mat: &Mat3,
    scale: f32,
    mode: PositionMode,
    rot: Quat,
    from: Vec3,
) -> Vec3 {
    let pos = (*mat * from) / scale;
    match mode {
        PositionMode::Flat => pos,
        PositionMode::Rotated => rot * pos,
    }
}

fn system_sync_coordinates(
    config_q: Query<Ref<ui::ConfigIO>>,
    mut coord_q: Query<&mut CoordinateSystem>,
    mut axes_q: Query<(&mut Transform, &Axis), Without<crate::arrow::Arrow>>,
    mut arrows_q: Query<(&mut Transform, Ref<repr::ComputedRepresentation>), With<crate::arrow::Arrow>>,
) {
    let mut coord = coord_q.single_mut().unwrap();
    let config = config_q.single().unwrap();

    let prev_internal2user = coord.internal2user;
    let prev_scale = config.positions_scale;
    let prev_pos_mode = coord.position_mode;
    if config.is_changed() {
        let forward_direction = config.forward.to_vec() * config.forward_sign;
        let up_direction = config.up.to_vec() * config.up_sign;
        let side_direction =
            forward_direction.cross(up_direction) * if config.hand == Hand::Left { -1.0 } else { 1.0 };

        let to_internal_basis = Mat3::from_cols(Vec3::X, Vec3::Y, Vec3::NEG_Z);
        let to_user_basis = Mat3::from_cols(side_direction, up_direction, forward_direction);

        coord.positions_scale = config.positions_scale;
        coord.position_mode = config.position_mode;
        coord.user2internal = to_internal_basis * to_user_basis.transpose();
        coord.internal2user = coord.user2internal.transpose();

        for (mut tf, axis) in axes_q.iter_mut() {
            let axis = axis.to_vec();
            tf.rotation = Quat::from_rotation_arc(axis, coord.user2internal * axis);
        }
    }

    if config.keep_numbers {
        for (mut tf, repr) in arrows_q.iter_mut() {
            if !config.is_changed() && !repr.is_changed() {
                continue;
            }

            let num_rot = convert_rotation(&prev_internal2user, tf.rotation);
            let num_pos = convert_position(&prev_internal2user, prev_scale.recip(), prev_pos_mode, num_rot.inverse(), tf.translation);
            tf.rotation = convert_rotation(&coord.user2internal, num_rot);
            tf.translation = convert_position(&coord.user2internal, coord.positions_scale, coord.position_mode, tf.rotation, num_pos);
        }
    }
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct UserTransform(pub Transform);

fn system_sync_objects(
    coord_q: Query<Ref<CoordinateSystem>>,
    mut arrows_q: Query<(Ref<Transform>, &mut UserTransform)>,
) {
    let coord = coord_q.single().unwrap();

    for (tf, mut utf) in arrows_q.iter_mut() {
        if !tf.is_changed() && !coord.is_changed() {
            continue;
        }

        let quat = convert_rotation(&coord.internal2user, tf.rotation);
        let pos = convert_position(&coord.internal2user, coord.positions_scale.recip(), coord.position_mode, quat.inverse(), tf.translation);

        utf.0 = Transform::default()
            .with_translation(pos)
            .with_rotation(quat);
    }
}

#[derive(Clone, Copy)]
pub enum AppliedTransform {
    Recompute,
    Position(Vec3),
    RotationQuat(Quat),
    RotationMat(Mat3),
    RotationEuler(Vec3),
    TransformMat(Mat4),
}

#[derive(Event)]
pub struct ApplyTransformCommand {
    pub target: Entity,
    pub transform: AppliedTransform,
}

impl ApplyTransformCommand {
    pub fn recompute(target: Entity) -> Self {
        ApplyTransformCommand {
            target,
            transform: AppliedTransform::Recompute,
        }
    }

    pub fn pos(target: Entity, pos: Vec3) -> Self {
        ApplyTransformCommand {
            target,
            transform: AppliedTransform::Position(pos),
        }
    }

    pub fn rot_quat(target: Entity, rot: Quat) -> Self {
        ApplyTransformCommand {
            target,
            transform: AppliedTransform::RotationQuat(rot),
        }
    }

    pub fn rot_mat(target: Entity, rot: Mat3) -> Self {
        ApplyTransformCommand {
            target,
            transform: AppliedTransform::RotationMat(rot),
        }
    }

    pub fn rot_euler(target: Entity, rot: Vec3) -> Self {
        ApplyTransformCommand {
            target,
            transform: AppliedTransform::RotationEuler(rot),
        }
    }

    pub fn tf_mat(target: Entity, mat: Mat4) -> Self {
        ApplyTransformCommand {
            target,
            transform: AppliedTransform::TransformMat(mat),
        }
    }
}

fn system_process_transform_commands(
    mut events: EventReader<ApplyTransformCommand>,
    coord_q: Query<&CoordinateSystem>,
    mut arrows_q: Query<&mut Transform>,
) {
    let coord = coord_q.single().unwrap();

    for event in events.read() {
        let mut tf = if let Ok(tf) = arrows_q.get_mut(event.target) {
            tf
        } else {
            continue;
        };

        match event.transform {
            AppliedTransform::Recompute => {
                tf.set_changed();
            }

            AppliedTransform::Position(pos) => {
                tf.translation = convert_position(&coord.user2internal, coord.positions_scale, coord.position_mode, tf.rotation, pos);
            }

            AppliedTransform::RotationQuat(quat) => {
                let num_pos = convert_position(&Mat3::IDENTITY, 1.0, coord.position_mode, tf.rotation.inverse(), tf.translation);
                tf.rotation = convert_rotation(&coord.user2internal, quat);
                tf.translation = convert_position(&Mat3::IDENTITY, 1.0, coord.position_mode, tf.rotation, num_pos);
            }

            AppliedTransform::RotationMat(mat) => {
                let num_pos = convert_position(&Mat3::IDENTITY, 1.0, coord.position_mode, tf.rotation.inverse(), tf.translation);
                tf.rotation = convert_rotation(&coord.user2internal, Quat::from_mat3(&mat));
                tf.translation = convert_position(&Mat3::IDENTITY, 1.0, coord.position_mode, tf.rotation, num_pos);
            }

            AppliedTransform::RotationEuler(Vec3 { x, y, z }) => {
                let num_pos = convert_position(&Mat3::IDENTITY, 1.0, coord.position_mode, tf.rotation.inverse(), tf.translation);
                tf.rotation = convert_rotation(&coord.user2internal, Quat::from_euler(EulerRot::XYZ, x, y, z));
                tf.translation = convert_position(&Mat3::IDENTITY, 1.0, coord.position_mode, tf.rotation, num_pos);
            }

            AppliedTransform::TransformMat(mat) => {
                let mat = Transform::from_matrix(mat);

                tf.scale = mat.scale;
                tf.rotation = convert_rotation(&coord.user2internal, mat.rotation);
                tf.translation = convert_position(&coord.user2internal, coord.positions_scale, coord.position_mode, tf.rotation, mat.translation);
            }
        }
    }
}
