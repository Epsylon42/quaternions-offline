use bevy::prelude::*;
use crate::{ui, RenderingResources};

pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (sync_axes, sync_objects)
                .chain()
                .after(ui::UiSet)
        );
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

#[derive(Component, Clone, Copy)]
pub struct Config {
    pub up: Axis,
    pub forward: Axis,
    pub up_sign: f32,
    pub forward_sign: f32,
    pub hand: Hand,

    /// if true, changing coordinate system will preserve numeric values of the quaternion
    /// instead of its direction in the internal coordinate system
    pub keep_numerics: bool,
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
        }
    }
}

#[derive(Component)]
pub struct CoordinateSystem {
    /// Map<Axis, Entity>
    pub entities: [Entity; 3],

    pub user2internal: Mat3,
    pub internal2user: Mat3,
}

fn convert_quaternion(mat: Mat3, mut quat: Quat) -> Quat {
    let converted = mat * quat.xyz();
    quat.x = converted.x;
    quat.y = converted.y;
    quat.z = converted.z;
    quat
}

pub fn convert_rotation(coord: &CoordinateSystem, from: Quat) -> Quat {
    convert_quaternion(coord.user2internal, from)
}

pub fn sync_axes(
    config_q: Query<Ref<Config>>,
    mut coord_q: Query<&mut CoordinateSystem>,
    mut axes_q: Query<&mut Transform, (Without<MainPlane>, Without<ui::QuatObject>)>,
    mut objects_q: Query<&mut Transform, With<ui::QuatObject>>,
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

    coord.user2internal = to_internal_basis * to_user_basis.transpose();
    coord.internal2user = coord.user2internal.transpose();

    for axis in Axis::all() {
        let mut tf = axes_q.get_mut(coord.entities[axis as usize]).unwrap();

        let axis = axis.to_vec();
        tf.rotation = Quat::from_rotation_arc(axis, coord.user2internal * axis);
    }

    if config.keep_numerics {
        for mut tf in objects_q.iter_mut() {
            let num = convert_quaternion(prev_internal2user, tf.rotation);
            tf.rotation = convert_quaternion(coord.user2internal, num);
        }
    }
}

pub fn sync_objects(
    mut cmd: Commands,
    coord_q: Query<Ref<CoordinateSystem>>,
    res: Res<RenderingResources>,
    mut objects_q: Query<(&mut ui::QuatObject, Ref<Transform>)>,
    new_objects_q: Query<Entity, (With<ui::QuatObject>, Without<Name>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let coord = coord_q.single().unwrap();

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

        let mat = Mat3::from_quat(quat).to_cols_array();
        for (from, to) in mat.into_iter().zip(&mut obj.mat) {
            *to = from.to_string();
        }

        let look = coord.internal2user * (tf.rotation * Vec3::NEG_Z);
        obj.look[0] = look.x.to_string();
        obj.look[1] = look.y.to_string();
        obj.look[2] = look.z.to_string();
    }

    for ent in new_objects_q.iter() {
        cmd.entity(ent).insert((
            Name::new(format!("Quat{i}")),
            Mesh3d(res.obj_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                depth_bias: -0.5,
                unlit: true,
                ..Color::BLACK.into()
            })),
        ));
        i += 1;
    }
}
