use crate::asset_loader::SceneAssets;
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;

#[derive(Component)]
pub enum ObstacleMarker {
    Obstacle(u32),
    None,
}

#[derive(Component)]
pub struct Obstacle;

#[derive(Bundle)]
pub struct ObstacleBundle {
    pub obstacle: Obstacle,
    pub marker: ObstacleMarker,
    pub scene: SceneRoot,
    pub transform: Transform,
    pub wireframe: Wireframe,
}

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_obstacles);
    }
}

fn spawn_obstacles(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    commands.spawn((ObstacleBundle {
        obstacle: Obstacle,
        marker: ObstacleMarker::Obstacle(1),
        scene: SceneRoot(scene_assets.terrain.clone()),
        wireframe: Wireframe,
        transform: Transform::from_xyz(1., 5., 5.).with_scale(Vec3::new(50., 50., 50.)),
    },));

    commands.spawn((ObstacleBundle {
        obstacle: Obstacle,
        marker: ObstacleMarker::Obstacle(2),
        scene: SceneRoot(scene_assets.asteroid.clone()),
        wireframe: Wireframe,
        transform: Transform::from_xyz(1., 1., -5.).with_scale(Vec3::new(1., 1., 1.)),
    },));
}
