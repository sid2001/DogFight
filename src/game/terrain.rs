use bevy::color::Color;
use bevy::prelude::*;

#[derive(Component)]
pub struct TerrainMarker;
pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
    }
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::new(Vec3::Z, Vec2::new(50., 50.))
                    .mesh()
                    .subdivisions(50),
            ),
        ),
        TerrainMarker,
        // MeshMaterial3d(materials.add(Color::from(Color::Srgba(Srgba::RED)))),
        Transform::from_xyz(0., 0., 0.),
    ));
}
