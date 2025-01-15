mod asset_loader;
mod events;
mod game;
mod sets;
mod states;

use asset_loader::AssetLoaderPlugin;
use bevy::prelude::*;
use events::EventPlugin;
use game::spaceship::Entities;
use game::GamePlugin;
use states::GameState;
use states::StatePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AssetLoaderPlugin)
        .init_resource::<Entities>()
        .add_plugins(EventPlugin)
        .add_plugins(StatePlugin)
        .add_plugins(GamePlugin)
        // .add_plugins(MenuPlugin)
        .run();
}
