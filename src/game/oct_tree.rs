use crate::asset_loader::*;
use std::thread::sleep;
use std::time::Duration;

use bevy::gltf::GltfMeshExtras;
use bevy::gltf::GltfPrimitive;
use bevy::prelude::*;

use super::mesh::*;
use super::obstacle::*;
use super::spaceship::SpaceShip;

pub struct OctTreePlugin;
impl Plugin for OctTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, print_landscape);
    }
}

fn print_landscape(
    mut commands: Commands,
    // res: Res<MeshResource>,
    mut mes: ResMut<Assets<Mesh>>,
    // gltf_asset: Res<Assets<GltfMesh>>,
    ass: Res<AssetServer>,
    scene_ass: Res<SceneAssets>,
    gltf_primitive: Res<Assets<GltfPrimitive>>,
    query: Query<&Parent, With<SpaceShip>>,
) {
    // let mut scene;
    // for (s, gltf_mesh) in query.iter() {
    //     if let Some(mesh) = gltf_mesh {
    //         info!("mesh {:?}", mesh);
    //     } else {
    //         info!("Cannot load mesh");
    //     }
    // }
    info!("hello");
    // let mesh: Handle<Mesh> = ass.load("Spaceship2.glb#Mesh0/Primitive0");
    // sleep(Duration::new(1, 0));
    // // let handle = mes.add(mesh.ass);
    // for mesh in gltf_asset.iter() {
    //     // for x in mesh {
    //     println!("Mesh successfully retrieved: {:?}", mesh.1.primitives);
    //     // }
    // }
    // else {
    //     println!("Mesh not loaded yet.");
    // }
    // let a = mes.get(&mesh).unwrap();
    // for bb in a.count_vertices() {
    //     info!("{:?}", bb.1);
    // }
    // info!("{}", a.count_vertices());
    // let a = gltf_primitive.get(scene.id());
    // if let Some(gltf) = gltf_asset.get_handle_provider(). {}
}
