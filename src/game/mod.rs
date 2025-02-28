pub mod bots;
pub mod camera;
pub mod collider;
pub mod debug;
pub mod environment;
pub mod explosion;
pub mod mesh;
pub mod movement;
pub mod obstacle;
mod oct_tree;
pub mod spaceship;
pub mod swarm;
mod terrain;
pub mod turret;

use crate::events::TurretEventPlugin;
use bevy::prelude::*;
use bots::BotPlugin;
use camera::CameraPlugin;
use collider::{Collider, ColliderPlugin};
use debug::DebugPlugin;
use environment::LandscapePlugin;
use explosion::ExplosionPlugin;
use mesh::TestMeshPlugin;
use obstacle::ObstaclePlugin;
use oct_tree::OctTreePlugin;
use spaceship::SpaceShipPlugin;
use swarm::SwarmPlugin;
use terrain::TerrainPlugin;
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
        .add_plugins(ColliderPlugin)
        .add_plugins(SwarmPlugin)
        // .add_plugins(ObstaclePlugin);
        // .add_plugins(TestMeshPlugin);
        // .add_plugins(BotPlugin)
        // .add_plugins(DebugPlugin);
        // .add_plugins(TerrainPlugin)
        // .add_plugins(OctTreePlugin);
        .add_plugins(ExplosionPlugin);
    }
}
