use bevy::prelude::*;
use bevy::color::palettes::css as pallette;
use bevy_rich_text3d as text3d;

mod camera;
mod mesh;
mod ui;
mod geometry;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(pallette::ANTIQUE_WHITE.into()))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 100.0,
            ..default()
        })
        .add_plugins(bevy_embedded_assets::EmbeddedAssetPlugin {
            mode: bevy_embedded_assets::PluginMode::ReplaceDefault
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(text3d::Text3dPlugin::default())
        .insert_resource(text3d::LoadFonts {
            font_embedded: vec![
                include_bytes!("../assets/FiraSans-Medium.ttf")
            ],
            ..default()
        })
        .add_plugins(bevy_obj::ObjPlugin::default())
        .add_plugins(ui::UiPlugins)
        .add_plugins(geometry::GeometryPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, camera::pan_orbit_camera)
        ;


    app.run();
}

#[derive(Component)]
struct Billboarded;

#[derive(Component)]
struct MainCamera;

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
        Camera3d::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        camera::PanOrbitCamera {
            radius,
            ..default()
        },
        Transform::from_translation(Vec3::splat(1.0).normalize() * radius)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let mesh = meshes.add(mesh::create_plane_mesh());
    let material = materials.add(StandardMaterial {
        cull_mode: None,
        unlit: true,
        depth_bias: -1.0,
        alpha_mode: AlphaMode::Blend,
        ..Color::from(pallette::GRAY).into()
    });

    let res = RenderingResources {
        red: materials.add(StandardMaterial {
            unlit: true,
            depth_bias: -0.5,
            ..Color::from(pallette::RED).into()
        }),
        green: materials.add(StandardMaterial {
            unlit: true,
            depth_bias: -0.5,
            ..Color::from(pallette::GREEN).into()
        }),
        blue: materials.add(StandardMaterial {
            unlit: true,
            depth_bias: -0.5,
            ..Color::from(pallette::BLUE).into()
        }),

        obj_mesh: assets.load("arrow.obj"),
    };


    cmd.spawn((
        geometry::MainPlane,
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone())
    ));

    let axis_mesh = meshes.add(Cylinder::new(0.012, 1.0).mesh().resolution(10).segments(1));

    let mut coord = geometry::CoordinateSystem {
        entities: [Entity::PLACEHOLDER; 3],
        user2internal: Mat3::IDENTITY,
        internal2user: Mat3::IDENTITY,
    };

    for (axis, color, up) in [
        (geometry::Axis::X, Color::from(pallette::RED), Vec3::Y),
        (geometry::Axis::Y, Color::from(pallette::GREEN), Vec3::Z),
        (geometry::Axis::Z, Color::from(pallette::BLUE), Vec3::Y),
    ] {
        let material = materials.add(StandardMaterial {
            base_color: color,
            depth_bias: -0.5,
            unlit: true,
            cull_mode: None,
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        });
        let text_material = materials.add(StandardMaterial {
            base_color_texture: Some(text3d::TextAtlas::DEFAULT_IMAGE.clone_weak()),
            base_color: color,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        });

        let neg_color = {
            let mut hsla = Hsla::from(color);
            hsla.saturation /= 1.5;
            hsla.lightness = hsla.lightness.powf(0.25);
            Color::from(hsla)
        };
        let neg_material = materials.add(StandardMaterial {
            base_color: neg_color,
            depth_bias: -0.5,
            unlit: true,
            ..default()
        });

        let ent = cmd
            .spawn((Transform::default(), Visibility::default()))
            .with_children(|cmd| {
                cmd.spawn((
                    Mesh3d(axis_mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::default()
                        .looking_to(axis.to_vec(), up)
                        .mul_transform(
                            Transform::from_xyz(0.0, 0.0, -0.5)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::TAU / 4.0)),
                        ),
                ));

                cmd.spawn((
                    Mesh3d(axis_mesh.clone()),
                    MeshMaterial3d(neg_material),
                    Transform::default()
                        .looking_to(-axis.to_vec(), up)
                        .mul_transform(
                            Transform::from_xyz(0.0, 0.0, -0.3)
                                .with_scale(Vec3::new(0.4, 0.6, 0.4))
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::TAU / 4.0)),
                        ),
                ));

                cmd.spawn((
                    Transform::default()
                        .looking_to(axis.to_vec(), up)
                        .mul_transform(
                            Transform::from_xyz(0.0, 0.0, 0.0)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::TAU / 4.0)),
                        ),
                    Visibility::default()
                ))
                .with_children(|cmd| {
                    cmd.spawn((
                        Transform::from_xyz(0.0, 1.1, 0.0),
                        text3d::Text3d::new(axis.name()),
                        text3d::Text3dStyling {
                            size: 64.0,
                            world_scale: Some(Vec2::splat(0.1)),
                            anchor: text3d::TextAnchor::BOTTOM_CENTER,
                            ..default()
                        },
                        Mesh3d::default(),
                        MeshMaterial3d(text_material)
                    ));
                });
            })
            .id();

        coord.entities[axis as usize] = ent;
    }

    cmd.spawn(coord);

    cmd.insert_resource(res);

    cmd.spawn(geometry::Config::default());

    cmd.spawn(ui::QuatObject::default());
}
