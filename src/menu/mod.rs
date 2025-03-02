use crate::asset_loader::MenuAssets;
use crate::sets::*;
use crate::states::*;
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;

const PRESSED_BUTTON_COLOR: Color = Color::srgba(1.07433, 0.77008, -0.30753, 0.5);
const HOVERED_BUTTON_COLOR: Color = Color::srgba(1.07433, 0.77008, -0.30753, 1.0);
const NORMAL_BUTTON_COLOR: Color = Color::WHITE;

#[derive(Component)]
pub enum MenuAction {
    Home,
    Start,
    Settings,
    Exit,
}

#[derive(Component)]
pub struct MenuObjectMarker;

#[derive(Resource)]
pub struct RouteStack(Vec<MenuState>);

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .configure_sets(Update, UpdateSet::Menu.run_if(in_state(GameState::Menu)))
            .add_systems(OnEnter(GameState::Menu), setup.in_set(MenuSet::Startup))
            .add_systems(OnEnter(MenuState::Home), home_setup)
            .add_systems(Update, (button_hover, menu_action).in_set(UpdateSet::Menu))
            .add_systems(OnExit(GameState::Menu), despawn_screen::<MenuObjectMarker>);
    }
}

fn setup(mut menu_state: ResMut<NextState<MenuState>>, mut commands: Commands) {
    commands.spawn((Camera2d::default(), MenuObjectMarker));
    menu_state.set(MenuState::Home);
}

fn home_setup(mut commands: Commands, asset_loader: Res<MenuAssets>, assets: Res<Assets<Image>>) {
    let menu_screen_node = Node {
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

    let menu_node = Node {
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

    let button_start = commands
        .spawn((
            button_node.clone(),
            Name::from("button_start"),
            ImageNode {
                image: asset_loader.start.clone().into(),
                image_mode: NodeImageMode::Auto,
                ..Default::default()
            },
            Button,
            MenuAction::Start,
            MenuObjectMarker,
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
            MenuAction::Settings,
            MenuObjectMarker,
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
            MenuAction::Exit,
            MenuObjectMarker,
        ))
        .id();

    let menu_container = commands
        .spawn((
            menu_node,
            BorderRadius::all(Val::Px(5.)),
            BoxShadow {
                color: Color::BLACK,
                spread_radius: Val::Px(10.),
                blur_radius: Val::Px(5.),
                x_offset: Val::ZERO,
                y_offset: Val::ZERO,
                ..default()
            },
            Name::from("menu_container"),
            ImageNode {
                image: asset_loader.menu_background.clone().into(),
                image_mode: NodeImageMode::Auto,
                ..Default::default()
            },
            MenuObjectMarker,
        ))
        .id();
    let menu_screen = commands
        .spawn((
            menu_screen_node,
            Name::from("menu_screen"),
            ImageNode {
                image: asset_loader.screen_background.clone().into(),
                image_mode: NodeImageMode::Auto,
                ..Default::default()
            },
            MenuObjectMarker,
        ))
        .id();
    commands.entity(menu_screen).add_children(&[menu_container]);
    commands
        .entity(menu_container)
        .add_children(&[button_start, button_settings, button_exit]);
}

fn button_hover(
    mut interaction_query: Query<
        (&Interaction, &mut ImageNode),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut image_node) in &mut interaction_query {
        image_node.color = match interaction {
            Interaction::Pressed => PRESSED_BUTTON_COLOR,
            Interaction::Hovered => HOVERED_BUTTON_COLOR,
            // (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON_COLOR,
            // _ => NORMAL_BUTTON_COLOR,
        }
    }
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, Option<&MenuAction>),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    // menu_state: ResMut<NextState<MenuState>>,
) {
    for (interaction, menu_action) in interaction_query.iter() {
        match (*interaction, menu_action) {
            (Interaction::Pressed, Some(MenuAction::Start)) => {
                game_state.set(GameState::Game);
            }
            _ => (),
        }
    }
}

fn despawn_screen<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for ent in query.iter() {
        commands.entity(ent).despawn_recursive();
    }
}
