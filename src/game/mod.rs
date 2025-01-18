pub mod bots;
pub mod camera;
pub mod environment;
pub mod mesh;
pub mod movement;
pub mod obstacle;
pub mod spaceship;
pub mod turret;

use crate::events::TurretEventPlugin;
use bevy::prelude::*;
use bots::BotPlugin;
use camera::CameraPlugin;
use environment::LandscapePlugin;
use mesh::TestMeshPlugin;
use obstacle::ObstaclePlugin;
use spaceship::SpaceShipPlugin;
use turret::TurretPlugin;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TurretPlugin {
            bullet_scene_path: String::from("lazer_bullet.glb#Scene0"),
        })
        // .add_plugins(LandscapePlugin)
        .add_plugins(TurretEventPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(SpaceShipPlugin)
        .add_plugins(ObstaclePlugin)
        .add_plugins(TestMeshPlugin)
        .add_plugins(BotPlugin);
    }
}
