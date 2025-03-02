use crate::states::{GameState, InGameStates};
use crate::{asset_loader::MenuAssets, sets::*};
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use bevy_inspector_egui::egui::Order;

pub struct PauseMenuPlugin;
impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameStates::Paused),
            setup
                .in_set(SetupSet::InGame)
                .run_if(in_state(GameState::Game)),
        )
        .add_systems(
            OnExit(InGameStates::Paused),
            despawn_screen::<PauseMenuObjectMarker>,
        )
        .add_systems(
            Update,
            pause_menu_action
                .run_if(in_state(InGameStates::Paused))
                .run_if(in_state(GameState::Game)),
        );
    }
}

#[derive(Component)]
pub struct PauseMenuObjectMarker;

#[derive(Component)]
pub enum PauseMenuAction {
    Resume,
    Exit,
    Settings,
}
fn setup(mut commands: Commands, asset_loader: Res<MenuAssets>) {
    error!("Game Paused");
    commands.spawn((
        Camera2d,
        Camera {
            order: 3,
            ..Default::default()
        },
        PauseMenuObjectMarker,
    ));
    let pause_menu_screen_node = Node {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        padding: UiRect::all(Val::Px(10.)),
        display: Display::Flex,
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        position_type: PositionType::Absolute,
        ..default()
    };

    let pause_menu_node = Node {
        width: Val::Percent(25.),
        height: Val::Auto,
        padding: UiRect::all(Val::Px(10.)),
        flex_direction: FlexDirection::Column,
        display: Display::Flex,
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::Center,
        overflow: Overflow::clip(),
        position_type: PositionType::Absolute,
        ..default()
    };

    let button_node = Node {
        width: Val::Auto,
        height: Val::Px(50.),
        margin: UiRect::vertical(Val::Percent(10.0)),
        align_items: AlignItems::Center,
        ..default()
    };

    let button_resume = commands
        .spawn((
            button_node.clone(),
            Name::from("button_start"),
            ImageNode {
                image: asset_loader.resume.clone().into(),
                image_mode: NodeImageMode::Auto,
                ..Default::default()
            },
            Button,
            PauseMenuAction::Resume,
            PauseMenuObjectMarker,
        ))
        .id();

    let button_settings = commands
        .spawn((
            button_node.clone(),
            Name::from("button_start"),
            ImageNode {
                image: asset_loader.settings.clone().into(),
                image_mode: NodeImageMode::Auto,
                ..Default::default()
            },
            Button,
            PauseMenuAction::Settings,
            PauseMenuObjectMarker,
        ))
        .id();

    let button_exit = commands
        .spawn((
            button_node.clone(),
            Name::from("button_start"),
            ImageNode {
                image: asset_loader.exit.clone().into(),
                image_mode: NodeImageMode::Auto,
                ..Default::default()
            },
            Button,
            PauseMenuAction::Exit,
            PauseMenuObjectMarker,
        ))
        .id();

    let pause_menu_container = commands
        .spawn((
            pause_menu_node,
            BorderRadius::all(Val::Px(5.)),
            BoxShadow {
                color: Color::BLACK,
                spread_radius: Val::Px(10.),
                blur_radius: Val::Px(5.),
                x_offset: Val::ZERO,
                y_offset: Val::ZERO,
                ..default()
            },
            Name::from("pause_menu_container"),
            PauseMenuObjectMarker,
        ))
        .id();
    let pause_menu_screen = commands
        .spawn((
            pause_menu_screen_node,
            Name::from("pause_menu_screen"),
            BoxShadow {
                color: Color::BLACK,
                x_offset: Val::ZERO,
                y_offset: Val::ZERO,
                spread_radius: Val::Percent(10.),
                blur_radius: Val::Percent(10.),
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.5)),
            PauseMenuObjectMarker,
        ))
        .id();
    commands
        .entity(pause_menu_screen)
        .add_children(&[pause_menu_container]);
    commands.entity(pause_menu_container).add_children(&[
        button_resume,
        button_settings,
        button_exit,
    ]);
}

fn despawn_screen<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for ent in query.iter() {
        commands.entity(ent).despawn_recursive();
    }
}

fn pause_menu_action(
    interaction_query: Query<
        (&Interaction, Option<&PauseMenuAction>),
        (Changed<Interaction>, With<Button>),
    >,
    mut in_game_state: ResMut<NextState<InGameStates>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_action) in interaction_query.iter() {
        match (*interaction, menu_action) {
            (Interaction::Pressed, Some(PauseMenuAction::Resume)) => {
                in_game_state.set(InGameStates::Play);
            }
            (Interaction::Pressed, Some(PauseMenuAction::Exit)) => {
                game_state.set(GameState::Menu);
            }
            _ => (),
        }
    }
}
