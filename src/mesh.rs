use bevy::prelude::*;

pub fn create_plane_mesh() -> Mesh {
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::LineList);

    let positions: Vec<_> = (-5..=5)
        .flat_map(|x| (-5..=5).map(move |y| Vec3::new(x as f32, 0.0, y as f32) / 5.0))
        .collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    let mut indices = Vec::new();

    for i in 0..=11 {
        for j in 0..=10 {
            if i != 10 {
                indices.push(j * 11 + i);
                indices.push(j * 11 + i + 1);
            }
            if j != 10 {
                indices.push(j * 11 + i);
                indices.push((j + 1) * 11 + i);
            }
        }
    }

    mesh.set_indices(Some(bevy::render::mesh::Indices::U16(indices)));

    mesh
}
