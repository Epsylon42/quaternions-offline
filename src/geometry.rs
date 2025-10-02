use crate::display::ui;
use bevy::{ecs::query::QueryData, prelude::*};

pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (sync_coordinates, sync_objects, process_transform_commands)
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

    pub fn all() -> [Self; 3] {
        [Axis::X, Axis::Y, Axis::Z]
    }

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

#[derive(Component)]
pub struct CoordinateSystem {
    pub user2internal: Mat3,
    pub internal2user: Mat3,
    pub positions_scale: f32,
}

impl Default for CoordinateSystem {
    fn default() -> Self {
        Self {
            user2internal: Mat3::IDENTITY,
            internal2user: Mat3::IDENTITY,
            positions_scale: 1.0,
        }
    }
}

fn convert_quaternion(mat: Mat3, mut quat: Quat) -> Quat {
    let converted = mat * quat.xyz();
    quat.x = converted.x;
    quat.y = converted.y;
    quat.z = converted.z;
    quat
}

fn convert_position(mat: Mat3, pos: Vec3) -> Vec3 {
    mat * pos
}

pub fn prepare_rotation(coord: &CoordinateSystem, from: Quat) -> Quat {
    convert_quaternion(coord.user2internal, from)
}

pub fn prepare_position(coord: &CoordinateSystem, rot: Quat, mode: PositionMode, from: Vec3) -> Vec3 {
    let pos = convert_position(coord.user2internal, from) / coord.positions_scale;
    match mode {
        PositionMode::Flat => pos,
        PositionMode::Rotated => rot * pos,
    }
}

fn sync_coordinates(
    config_q: Query<Ref<ui::Config>>,
    mut coord_q: Query<&mut CoordinateSystem>,
    mut axes_q: Query<(&mut Transform, &Axis), Without<crate::objects::Arrow>>,
    mut arrows_q: Query<&mut Transform, With<crate::objects::Arrow>>,
) {
    let mut coord = coord_q.single_mut().unwrap();
    let config = config_q.single().unwrap();
    if !config.is_changed() {
        return;
    }

    let forward_direction = config.forward.to_vec() * config.forward_sign;
    let up_direction = config.up.to_vec() * config.up_sign;
    let side_direction =
        forward_direction.cross(up_direction) * if config.hand == Hand::Left { -1.0 } else { 1.0 };

    let to_internal_basis = Mat3::from_cols(Vec3::X, Vec3::Y, Vec3::NEG_Z);
    let to_user_basis = Mat3::from_cols(side_direction, up_direction, forward_direction);

    let prev_internal2user = coord.internal2user;
    let prev_scale = coord.positions_scale;

    coord.positions_scale = config.positions_scale;
    coord.user2internal = to_internal_basis * to_user_basis.transpose();
    coord.internal2user = coord.user2internal.transpose();

    for (mut tf, axis) in axes_q.iter_mut() {
        let axis = axis.to_vec();
        tf.rotation = Quat::from_rotation_arc(axis, coord.user2internal * axis);
    }

    if config.keep_numerics {
        for mut tf in arrows_q.iter_mut() {
            let num_rot = convert_quaternion(prev_internal2user, tf.rotation);
            let num_pos = convert_position(prev_internal2user, tf.translation) / prev_scale;
            tf.rotation = convert_quaternion(coord.user2internal, num_rot);
            tf.translation = convert_position(coord.user2internal, num_pos) / coord.positions_scale;
        }
    }
}

#[derive(Component, Default)]
pub struct UserTransform(pub Transform);

fn sync_objects(
    coord_q: Query<Ref<CoordinateSystem>>,
    mut arrows_q: Query<(Ref<Transform>, &mut UserTransform)>,
) {
    let coord = coord_q.single().unwrap();

    for (tf, mut utf) in arrows_q.iter_mut() {
        if !tf.is_changed() && !coord.is_changed() {
            continue;
        }

        let pos = coord.internal2user * tf.translation * coord.positions_scale;
        let quat = convert_quaternion(coord.internal2user, tf.rotation);

        utf.0 = Transform::default().with_translation(pos).with_rotation(quat);
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum PositionMode {
    #[default]
    Flat,
    Rotated,
}

#[derive(Clone, Copy)]
pub enum AppliedTransform {
    Recompute,
    Position(Vec3, PositionMode),
    RotationQuat(Quat),
    RotationMat(Mat3),
    RotationEuler(Vec3),
}

#[derive(Event)]
pub struct ApplyTransformCommand {
    pub target: Entity,
    pub transform: AppliedTransform,
}

impl ApplyTransformCommand {
    pub fn recompute(target: Entity) -> Self {
        ApplyTransformCommand { target, transform: AppliedTransform::Recompute }
    }

    pub fn pos(target: Entity, pos: Vec3, mode: PositionMode) -> Self {
        ApplyTransformCommand {
            target,
            transform: AppliedTransform::Position(pos, mode),
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
}

fn process_transform_commands(
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

            AppliedTransform::Position(pos, mode) => {
                tf.translation = prepare_position(coord, tf.rotation, mode, pos);
            }

            AppliedTransform::RotationQuat(quat) => {
                tf.rotation = prepare_rotation(coord, quat);
            }

            AppliedTransform::RotationMat(mat) => {
                tf.rotation = prepare_rotation(coord, Quat::from_mat3(&mat));
            }

            AppliedTransform::RotationEuler(Vec3 { x, y, z }) => {
                tf.rotation = prepare_rotation(coord, Quat::from_euler(EulerRot::XYZ, x, y, z));
            }
        }
    }
}
