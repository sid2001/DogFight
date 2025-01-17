use bevy::prelude::*;
trait MenuItem {}
trait InGameState {}

impl InGameState for InGameStates {}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MenuItems {
    #[default]
    Home,
    Settings,
    CoOp,
}
impl MenuItem for MenuItems {}
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum InGameStates {
    Paused,
    #[default]
    Play,
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
    Menu(MenuItems),
    // #[default]
    InGame(InGameStates),
    // InGame(InGameStates::Play),
}

impl Default for GameState {
    fn default() -> Self {
        Self::InGame(InGameStates::Play)
    }
}

pub struct StatePlugin;
impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(Update, state_event_control);
    }
}

// pub struct GameStatePlugin;
// impl Plugin for GameStatePlugin {
//   fn build(&self, app: &mut App) {
//     app.add_state::<InGameState>()
//     .add_systems()
//   }
// }

fn state_event_control(
    curr_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    error!("Press");
    match keys.just_pressed(KeyCode::Tab) {
        true => match curr_state.get() {
            GameState::InGame(InGameStates::Paused) => {
                next_state.set(GameState::InGame(InGameStates::Play));
            }
            GameState::InGame(InGameStates::Play) => {
                next_state.set(GameState::InGame(InGameStates::Paused));
            }
            _ => (),
        },
        _ => (),
    }
}
