use crate::ui;
use bevy::{ecs::query::QueryData, prelude::*};

pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (sync_axes, sync_objects).chain().after(ui::UiSet));
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
    pub arrow_defaults: ui::ArrowDisplay,
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
            arrow_defaults: default(),
        }
    }
}

#[derive(Component)]
pub struct CoordinateSystem {
    /// Map<Axis, Entity>
    pub entities: [Entity; 3],

    pub user2internal: Mat3,
    pub internal2user: Mat3,
    pub positions_scale: f32,
}

impl Default for CoordinateSystem {
    fn default() -> Self {
        Self {
            entities: [Entity::PLACEHOLDER; 3],
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

pub fn prepare_position(coord: &CoordinateSystem, from: Vec3) -> Vec3 {
    convert_position(coord.user2internal, from) * coord.positions_scale
}

pub fn sync_axes(
    config_q: Query<Ref<Config>>,
    mut coord_q: Query<&mut CoordinateSystem>,
    mut axes_q: Query<&mut Transform, (Without<MainPlane>, Without<ui::ArrowIO>)>,
    mut arrows_q: Query<&mut Transform, With<ui::ArrowIO>>,
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

    for axis in Axis::all() {
        let mut tf = axes_q.get_mut(coord.entities[axis as usize]).unwrap();

        let axis = axis.to_vec();
        tf.rotation = Quat::from_rotation_arc(axis, coord.user2internal * axis);
    }

    if config.keep_numerics {
        for mut tf in arrows_q.iter_mut() {
            let num_rot = convert_quaternion(prev_internal2user, tf.rotation);
            let num_pos = convert_position(prev_internal2user, tf.translation) / prev_scale;
            tf.rotation = convert_quaternion(coord.user2internal, num_rot);
            tf.translation = convert_position(coord.user2internal, num_pos) * coord.positions_scale;
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct SyncObjectsArrowQuery<'a> {
    ent: Entity,
    tf: Ref<'a, Transform>,
    arrow: &'a mut ui::ArrowIO,
    display: &'a mut ui::ArrowDisplay,
    material: Option<&'a MeshMaterial3d<StandardMaterial>>,
    has_name: Has<Name>,
}

pub fn sync_objects(
    mut cmd: Commands,
    coord_q: Query<Ref<CoordinateSystem>>,
    mut arrows_q: Query<SyncObjectsArrowQuery>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let coord = coord_q.single().unwrap();

    let mut i = 0;
    for SyncObjectsArrowQueryItem { tf, mut arrow, has_name, .. } in arrows_q.iter_mut() {
        if !has_name {
            continue;
        }

        i += 1;
        if !tf.is_changed() && !coord.is_changed() {
            continue;
        }

        let pos = coord.internal2user * tf.translation / coord.positions_scale;
        arrow.pos = pos;
        // arrow.pos[0] = pos.x.to_string();
        // arrow.pos[1] = pos.y.to_string();
        // arrow.pos[2] = pos.z.to_string();

        let quat = convert_quaternion(coord.internal2user, tf.rotation);
        arrow.quat[0] = quat.w.to_string();
        arrow.quat[1] = quat.x.to_string();
        arrow.quat[2] = quat.y.to_string();
        arrow.quat[3] = quat.z.to_string();

        let (x, y, z) = quat.to_euler(EulerRot::XYZ);
        arrow.euler = Vec3::new(x, y, z).map(f32::to_degrees);

        let mat = Mat3::from_quat(quat).to_cols_array();
        for (from, to) in mat.into_iter().zip(&mut arrow.mat) {
            *to = from.to_string();
        }

        // TODO: take position into account
        let look = coord.internal2user * (tf.rotation * Vec3::NEG_Z);
        arrow.look[0] = look.x.to_string();
        arrow.look[1] = look.y.to_string();
        arrow.look[2] = look.z.to_string();
    }


    for SyncObjectsArrowQueryReadOnlyItem { ent, display, has_name, .. } in arrows_q.iter() {
        if has_name {
            continue;
        }

        let material = materials.add(StandardMaterial {
            depth_bias: -0.5,
            unlit: true,
            ..Color::from(display.default_color).into()
        });

        cmd.entity(ent).insert((
            Name::new(format!("Arrow {i}")),
            MeshMaterial3d(material.clone()),
        ));
        i += 1;
    }

    for SyncObjectsArrowQueryItem { ent, mut display, material, .. } in arrows_q.iter_mut() {
        if !display.model_changed || material.is_none() {
            continue;
        }
        display.model_changed = false;

        cmd.entity(ent)
            .despawn_related::<Children>()
            .with_children(|cmd| {
                crate::mesh::spawn_arrow(
                    &mut *meshes,
                    cmd,
                    display.length,
                    display.scale,
                    material.unwrap().0.clone(),
                );
            });
    }
}
