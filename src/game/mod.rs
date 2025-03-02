pub mod bots;
pub mod camera;
pub mod collider;
pub mod debug;
pub mod environment;
pub mod explosion;
pub mod hud;
pub mod mesh;
pub mod movement;
pub mod obstacle;
mod oct_tree;
pub mod pause_menu;
pub mod spaceship;
pub mod swarm;
mod terrain;
pub mod turret;

use crate::sets::*;
use crate::states::GameState;
use crate::{events::TurretEventPlugin, states::InGameStates};
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
use pause_menu::PauseMenuPlugin;
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
        .add_plugins(BotPlugin)
        .add_plugins(DebugPlugin)
        // .add_plugins(TerrainPlugin)
        // .add_plugins(OctTreePlugin);
        .add_plugins(ExplosionPlugin)
        .add_plugins(PauseMenuPlugin)
        .configure_sets(
            Update,
            UpdateSet::InGame
                .run_if(in_state(GameState::Game))
                .run_if(in_state(InGameStates::Play)),
        )
        .add_systems(
            OnEnter(GameState::Game),
            (
                spaceship::setup,
                bots::setup,
                camera::setup,
                swarm::setup,
                turret::setup,
            )
                .in_set(SetupSet::InGame),
        )
        .add_systems(
            Update,
            in_game_state_action.run_if(in_state(GameState::Game)),
        )
        .add_systems(OnExit(GameState::Game), despawn_screen);
    }
}

fn in_game_state_action(
    keys: Res<ButtonInput<KeyCode>>,
    mut in_game_state: ResMut<NextState<InGameStates>>,
    curr_in_game_state: Res<State<InGameStates>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match curr_in_game_state.get() {
            InGameStates::Play => {
                in_game_state.set(InGameStates::Paused);
            }
            InGameStates::Paused => {
                in_game_state.set(InGameStates::Play);
            }
        }
    }
}

fn despawn_screen(mut commands: Commands, query: Query<Entity>) {
    for ent in query.iter() {
        commands.entity(ent).despawn_recursive();
    }
}
