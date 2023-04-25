use bevy::prelude::*;
use bevy_text_mesh::prelude::*;

mod camera;
mod mesh;
mod ui;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::ANTIQUE_WHITE))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 100.0,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_obj::ObjPlugin)
        .add_plugin(bevy_text_mesh::TextMeshPlugin)
        .add_plugins(ui::UiPlugins)
        .add_system(camera::pan_orbit_camera)
        .add_startup_system(setup)
        .add_system(sync_axes.run_if(config_changed).after(ui::UiSet))
        .run();
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn to_vec(self) -> Vec3 {
        match self {
            Axis::X => Vec3::X,
            Axis::Y => Vec3::Y,
            Axis::Z => Vec3::Z,
        }
    }

    fn all() -> [Self; 3] {
        [Axis::X, Axis::Y, Axis::Z]
    }

    fn name(&self) -> &'static str {
        match self {
            Axis::X => "X",
            Axis::Y => "Y",
            Axis::Z => "Z",
        }
    }
}

#[derive(Component)]
struct Billboarded;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hand {
    Left,
    Right,
}

#[derive(Component, Clone, Copy)]
pub struct Config {
    up: Axis,
    forward: Axis,
    up_sign: f32,
    forward_sign: f32,
    hand: Hand,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            up: Axis::Y,
            up_sign: 1.0,
            forward: Axis::Z,
            forward_sign: -1.0,
            hand: Hand::Right,
        }
    }
}

#[derive(Component)]
struct MainPlane;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
pub struct CoordinateSystem {
    /// Map<Axis, Entity>
    entities: [Entity; 3],

    user2internal: Mat3,
    internal2user: Mat3,
}

#[derive(Bundle, Default)]
struct QuatObjectBundle {
    spatial: SpatialBundle,
    quat_obj: ui::QuatObject,
}

#[derive(Resource)]
#[allow(dead_code)]
struct RenderingResources {
    red: Handle<StandardMaterial>,
    green: Handle<StandardMaterial>,
    blue: Handle<StandardMaterial>,

    obj_mesh: Handle<Mesh>,
}

fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    let radius = 3.0;

    cmd.spawn((
        MainCamera,
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::splat(1.0).normalize() * radius)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        camera::PanOrbitCamera {
            radius,
            ..default()
        },
    ));

    let mesh = meshes.add(mesh::create_plane_mesh());
    let material = materials.add(StandardMaterial {
        cull_mode: None,
        unlit: true,
        depth_bias: -1.0,
        ..Color::GRAY.into()
    });
    let font: Handle<TextMeshFont> = assets.load("FiraSans-Medium.ttf#mesh");

    let res = RenderingResources {
        red: materials.add(StandardMaterial {
            unlit: true,
            depth_bias: -0.5,
            ..Color::RED.into()
        }),
        green: materials.add(StandardMaterial {
            unlit: true,
            depth_bias: -0.5,
            ..Color::GREEN.into()
        }),
        blue: materials.add(StandardMaterial {
            unlit: true,
            depth_bias: -0.5,
            ..Color::BLUE.into()
        }),

        obj_mesh: assets.load("arrow.obj"),
    };

    cmd.spawn((
        MainPlane,
        MaterialMeshBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            ..default()
        },
    ));

    let axis_mesh = meshes.add(
        shape::Cylinder {
            radius: 0.012,
            height: 1.0,
            resolution: 3,
            segments: 1,
        }
        .into(),
    );

    let mut coord = CoordinateSystem {
        entities: [Entity::PLACEHOLDER; 3],
        user2internal: Mat3::IDENTITY,
        internal2user: Mat3::IDENTITY,
    };

    for (axis, color, up) in [
        (Axis::X, Color::RED, Vec3::Y),
        (Axis::Y, Color::GREEN, Vec3::Z),
        (Axis::Z, Color::BLUE, Vec3::Y),
    ] {
        let material = materials.add(StandardMaterial {
            depth_bias: -0.5,
            unlit: true,
            ..color.into()
        });

        let neg_color = match color.as_hsla_f32() {
            [h, s, l, a] => Color::hsla(h, s / 1.5, l.sqrt().sqrt(), a),
        };
        let neg_material = materials.add(StandardMaterial {
            depth_bias: -0.5,
            unlit: true,
            ..neg_color.into()
        });

        let ent = cmd
            .spawn(SpatialBundle::default())
            .with_children(|cmd| {
                cmd.spawn(MaterialMeshBundle {
                    transform: Transform::default()
                        .looking_to(axis.to_vec(), up)
                        .mul_transform(
                            Transform::from_xyz(0.0, 0.0, -0.5)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::TAU / 4.0)),
                        ),
                    mesh: axis_mesh.clone(),
                    material: material.clone(),
                    ..default()
                });

                cmd.spawn(MaterialMeshBundle {
                    transform: Transform::default()
                        .looking_to(-axis.to_vec(), up)
                        .mul_transform(
                            Transform::from_xyz(0.0, 0.0, -0.3)
                                .with_scale(Vec3::new(0.4, 0.6, 0.4))
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::TAU / 4.0)),
                        ),
                    mesh: axis_mesh.clone(),
                    material: neg_material,
                    ..default()
                });

                cmd.spawn(SpatialBundle {
                    transform: Transform::default()
                        .looking_to(axis.to_vec(), up)
                        .mul_transform(
                            Transform::from_xyz(0.0, 0.0, 0.0)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::TAU / 4.0)),
                        ),
                    ..default()
                })
                .with_children(|cmd| {
                    cmd.spawn((
                        TextMeshBundle {
                            transform: Transform::from_xyz(-0.025, 1.05, 0.0),
                            text_mesh: TextMesh {
                                text: axis.name().to_owned(),
                                style: TextMeshStyle {
                                    font: font.clone(),
                                    font_size: SizeUnit::NonStandard(8.0),
                                    color,
                                    ..default()
                                },
                                ..default()
                            },
                            ..default()
                        },
                    ));
                });
            })
            .id();

        coord.entities[axis as usize] = ent;
    }

    cmd.spawn(coord);

    cmd.insert_resource(res);

    cmd.spawn(Config::default());

    cmd.spawn(QuatObjectBundle::default());
}

fn config_changed(config_q: Query<Entity, Changed<Config>>) -> bool {
    !config_q.is_empty()
}

fn sync_axes(
    config_q: Query<&Config>,
    mut coord_q: Query<&mut CoordinateSystem>,
    mut axes_q: Query<&mut Transform, Without<MainPlane>>,
) {
    let mut coord = coord_q.single_mut();
    let config = config_q.single();

    let forward_direction = config.forward.to_vec() * config.forward_sign;
    let up_direction = config.up.to_vec() * config.up_sign;
    let side_direction =
        forward_direction.cross(up_direction) * if config.hand == Hand::Left { -1.0 } else { 1.0 };

    let to_internal_basis = Mat3::from_cols(Vec3::X, Vec3::Y, Vec3::NEG_Z);
    let to_user_basis = Mat3::from_cols(side_direction, up_direction, forward_direction);

    coord.user2internal = to_internal_basis * to_user_basis.transpose();
    coord.internal2user = coord.user2internal.transpose();

    for axis in Axis::all() {
        let mut tf = axes_q.get_mut(coord.entities[axis as usize]).unwrap();

        let axis = axis.to_vec();
        tf.rotation = Quat::from_rotation_arc(axis, coord.user2internal * axis);
    }
}
