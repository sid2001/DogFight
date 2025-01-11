mod asset_loader;
mod camera;
mod movement;
mod spaceship;

use bevy::prelude::*;

use asset_loader::AssetLoaderPlugin;
use camera::CameraPlugin;
use spaceship::SpaceShipPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(SpaceShipPlugin)
        .run();
}
