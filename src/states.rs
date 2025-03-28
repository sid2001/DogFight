use crate::asset_loader::AssetsLoading;
use bevy::{prelude::*, state::commands};
// trait MenuItem {}
// trait InGameState {}

// impl InGameState for InGameStates {}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MenuState {
    Home,
    #[default]
    Loading,
    Exit,
    Settings,
    CoOp,
}
// impl MenuItem for MenuItems {}
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum InGameStates {
    Paused,
    #[default]
    Play,
    Quit,
    Over,
    Restart,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States)]
pub enum SpaceShipActionState {
    Shooting(Action),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States)]
pub enum Action {
    True,
    // #[default]
    False,
}

impl Default for SpaceShipActionState {
    fn default() -> Self {
        Self::Shooting(Action::False)
    }
}

// pub enum GameState<T, U>
// where
//     T: MenuItems,
//     U: InGameState,
// {
//     #[default]
//     Menu(MenuItems),
//     InGame(InGameState),
// }
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States)]
pub enum GameState {
    Menu,
    // #[default]
    Game,
    Loading,
    // InGame(InGameStates::Play),
}

impl Default for GameState {
    fn default() -> Self {
        Self::Loading
    }
}

pub struct StatePlugin;
impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<InGameStates>()
            .init_state::<MenuState>()
            .add_systems(
                Update,
                game_startup_state.run_if(in_state(GameState::Loading)),
            );
        // .add_systems(Update, state_event_control);
    }
}

// pub struct GameStatePlugin;
// impl Plugin for GameStatePlugin {
//   fn build(&self, app: &mut App) {
//     app.add_state::<InGameState>()
//     .add_systems()
//   }
// }

fn game_startup_state(
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
) {
    use bevy::asset::LoadState;
    info!("game_start");
    for handle in loading.0.iter() {
        match asset_server.get_load_state(handle.id()).unwrap() {
            LoadState::Failed(_) => {
                // one of our assets had an error
                panic!("Failed loading asset");
            }
            LoadState::Loaded => {
                // all assets are now ready

                // this might be a good place to transition into your in-game state
                info!("Loaded");
                // remove the resource to drop the tracking handles
                // commands.remove_resource::<AssetsLoading>();
                // (note: if you don't have any other handles to the assets
                // elsewhere, they will get unloaded after this)
            }
            LoadState::Loading | LoadState::NotLoaded => {
                info!("Loading assets");
                return;
            }
        }
    }
    next_state.set(GameState::Menu);
}

fn state_event_control(
    curr_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // error!("Press");
    // match keys.just_pressed(KeyCode::Tab) {
    //     true => match curr_state.get() {
    //         GameState::InGame(InGameStates::Paused) => {
    //             next_state.set(GameState::InGame(InGameStates::Play));
    //         }
    //         GameState::InGame(InGameStates::Play) => {
    //             next_state.set(GameState::InGame(InGameStates::Paused));
    //         }
    //         _ => (),
    //     },
    //     _ => (),
    // }
}
