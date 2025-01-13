mod asset_loader;
mod camera;
mod environment;
mod events;
mod movement;
mod obstacle;
mod spaceship;

use bevy::prelude::*;

use asset_loader::AssetLoaderPlugin;
use camera::CameraPlugin;
use environment::LandscapePlugin;
use events::EventPlugin;
use obstacle::ObstaclePlugin;
use spaceship::{Entities, SpaceShipPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AssetLoaderPlugin)
        .init_resource::<Entities>()
        .add_plugins(EventPlugin)
        .add_plugins(LandscapePlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(SpaceShipPlugin)
        .add_plugins(ObstaclePlugin)
        .run();
}
