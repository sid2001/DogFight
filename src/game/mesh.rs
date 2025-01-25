use bevy::prelude::*;
use bevy::render::{
    mesh::{Indices, VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};

#[derive(Component)]
pub struct CustomMesh;
#[derive(Resource, Default)]
pub struct MeshResource {
    pub Landscape: Option<Handle<Mesh>>,
    pub Trees: Option<Handle<Mesh>>,
    pub Player: Option<Handle<Mesh>>,
}

pub struct TestMeshPlugin;
impl Plugin for TestMeshPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshResource>()
            .add_systems(Startup, setup_mesh);
    }
}

fn setup_mesh(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_res: ResMut<MeshResource>,
) {
    let length: f32 = 100.;
    let l_pieces: u32 = 300;
    let w_pieces: u32 = 300;
    let axes = (Vec3::Y, Vec3::Z);
    let origin = Vec3::new(0., 0., 0.);

    mesh_res.Landscape = Some(meshes.add(create_mesh(length, l_pieces, w_pieces, axes, origin)));
    commands.spawn((
        Mesh3d(mesh_res.Landscape.as_ref().unwrap().clone()),
        MeshMaterial3d(materials.add(Color::srgb(0.255, 0.0, 0.0))),
        CustomMesh,
    ));
}

fn create_mesh(
    length: f32,
    l_pieces: u32,
    w_pieces: u32,
    axes: (Vec3, Vec3),
    origin: Vec3,
) -> Mesh {
    let l_int: f32 = length / (l_pieces as f32);
    let w_int: f32 = length / (w_pieces as f32);
    let mut vertices = vec![];
    for i in 0..w_pieces {
        for j in 0..l_pieces {
            let d1 = (j as f32) * l_int;
            let d2 = (i as f32) * w_int;
            // info!("{:?}", get_tridinate(d1, d2, &origin, &axes));
            vertices.push(get_tridinate(d1, d2, &origin, &axes))
        }
    }

    let mut triads = vec![];
    for i in 1..(w_pieces - 1) {
        for j in 1..(l_pieces - 1) {
            triads.push(j - 1 + (l_pieces * (i - 1)));
            triads.push(j + (l_pieces * (i - 1)));
            triads.push(i * l_pieces + j - 1);

            triads.push(i * l_pieces + j + 1);
            triads.push(j + (l_pieces * (i - 1)));
            triads.push(i * l_pieces + j - 1);
        }
    }

    let mut normals = vec![];
    for _ in 0..w_pieces {
        for _ in 0..l_pieces {
            normals.push(Vec3::Z);
            normals.push(Vec3::Z);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(triads))
}

fn get_tridinate(d1: f32, d2: f32, origin: &Vec3, axes: &(Vec3, Vec3)) -> Vec3 {
    let mut vertex = *origin + d1 * axes.0 + d2 * axes.1;
    let y = if vertex.y.sin() > 0. {
        vertex.y.sin()
    } else {
        0.
    };
    let z = if vertex.z.sin() > 0. {
        vertex.z.sin()
    } else {
        0.
    };
    vertex.x = y * 4.;
    vertex
}
