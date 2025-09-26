use bevy::{asset::RenderAssetUsages, prelude::*, render::mesh::CylinderAnchor};

pub fn create_plane_mesh() -> Mesh {
    let size: i16 = 5;
    let full = size as u16 * 2 + 1;
    let grid_scale = 5.0;

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::LineList,
        RenderAssetUsages::RENDER_WORLD,
    );

    let positions: Vec<_> = (-size..=size)
        .flat_map(|x| (-size..=size).map(move |y| Vec3::new(x as f32, 0.0, y as f32) / grid_scale))
        .collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    let mut indices = Vec::new();
    for i in 0..full {
        for j in 0..full {
            if i != full - 1 {
                indices.push(j * full + i);
                indices.push(j * full + i + 1);
            }
            if j != full - 1 {
                indices.push(j * full + i);
                indices.push((j + 1) * full + i);
            }
        }
    }

    mesh.insert_indices(bevy::render::mesh::Indices::U16(indices));

    mesh
}

pub fn spawn_arrow(
    meshes: &mut Assets<Mesh>,
    cmd: &mut ChildSpawnerCommands,
    length: f32,
    radius_scale: f32,
    material: Handle<StandardMaterial>,
) {
    let radius = 0.011 * radius_scale;

    let base = Cylinder::new(radius, length)
        .mesh()
        .anchor(CylinderAnchor::Bottom)
        .resolution(10)
        .segments(1);
    cmd.spawn((
        Transform::default().looking_at(Vec3::Y, -Vec3::Z),
        Mesh3d(meshes.add(base)),
        MeshMaterial3d(material.clone()),
    ));

    let hook = Cylinder::new(radius, 0.2 * radius_scale)
        .mesh()
        .anchor(CylinderAnchor::Bottom)
        .resolution(10)
        .segments(1);
    cmd.spawn((
        Transform::default()
            .looking_at(Vec3::Y - Vec3::Z, Vec3::Y)
            .with_translation(Vec3::new(0.0, 0.0, -length)),
        Mesh3d(meshes.add(hook)),
        MeshMaterial3d(material.clone()),
    ));

    let top = Sphere::new(radius).mesh();
    cmd.spawn((
        Transform::from_xyz(0.0, 0.0, -length),
        Mesh3d(meshes.add(top)),
        MeshMaterial3d(material),
    ));
}
