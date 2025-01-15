pub mod camera;
pub mod environment;
pub mod movement;
pub mod obstacle;
pub mod spaceship;

use bevy::prelude::*;

use crate::states::GameState;
use crate::states::StatePlugin;
use camera::CameraPlugin;
use environment::LandscapePlugin;
use obstacle::ObstaclePlugin;
use spaceship::SpaceShipPlugin;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LandscapePlugin)
            .add_plugins(CameraPlugin)
            .add_plugins(SpaceShipPlugin)
            .add_plugins(ObstaclePlugin);
    }
}
