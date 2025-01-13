use bevy::prelude*;

#[derive(Component)]
pub enum ObstacleMarker {
  Obstacle(u32),
  None
}

#[derive(Component)]
pub struct Obstacle{
  scene: SceneBundle
}

#[derive(Bundle)]
pub struct ObstacleBundle {
  pub scale: Vec3,
  pub obstacle: Obstacle,
  pub marker: ObstacleMarker
}

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, spawn_obstacles)
  }
}