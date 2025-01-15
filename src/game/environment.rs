use bevy::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};

#[derive(Component)]
pub struct LandscapeMarker;

#[derive(Component)]
pub struct LandscapePlugin;

impl Plugin for LandscapePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, generate_land);
    }
}

fn generate_land(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let land = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane::from_size(15.0))),
        material: materials.add(Color::DARK_GREEN.into()),
        ..default()
    };

    commands.spawn(land);
}
