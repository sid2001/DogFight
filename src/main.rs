mod asset_loader;
mod camera;
mod environment;
mod movement;
mod spaceship;

use bevy::prelude::*;

use asset_loader::AssetLoaderPlugin;
use camera::CameraPlugin;
use environment::LandscapePlugin;
use spaceship::{Entities, SpaceShipPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AssetLoaderPlugin)
        .init_resource::<Entities>()
        // .add_plugins(LandscapePlugin)
        .add_plugins(SpaceShipPlugin)
        .add_plugins(CameraPlugin)
        .run();
}
