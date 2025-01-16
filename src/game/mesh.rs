use bevy::prelude::*;
use bevy::render::{
    mesh::{Indices, VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};

pub struct TestMeshPlugin;
impl Plugin for TestMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_mesh);
    }
}

fn setup_mesh(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
}

fn create_triangle() -> Mesh {}
