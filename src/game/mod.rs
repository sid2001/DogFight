pub mod bots;
pub mod camera;
pub mod collider;
pub mod debug;
pub mod environment;
pub mod explosion;
pub mod hud;
pub mod map_one;
pub mod mesh;
pub mod missile;
pub mod movement;
pub mod obstacle;
mod oct_tree;
pub mod pause_menu;
pub mod spaceship;
pub mod swarm;
mod terrain;
pub mod turret;

use std::alloc::GlobalAlloc;
use std::collections::VecDeque;

use oct_tree::*;

use crate::sets::*;
use crate::states::{GameState, MenuState};
use crate::{events::TurretEventPlugin, states::InGameStates};
use bevy::prelude::*;
use bevy::state::commands;
// use bevy_inspector_egui::egui::menu::MenuState;
use bots::BotPlugin;
use camera::CameraPlugin;
use collider::{Collider, ColliderInfo, ColliderMarker, ColliderPlugin, ColliderType};
use debug::DebugPlugin;
use environment::LandscapePlugin;
use explosion::ExplosionPlugin;
use map_one::MapOnePlugin;
use mesh::TestMeshPlugin;
use missile::MissilePlugin;
use obstacle::ObstaclePlugin;
use oct_tree::{NodeEntities, OctTree, OctTreePlugin};
use pause_menu::PauseMenuPlugin;
use spaceship::SpaceShipPlugin;
use swarm::SwarmPlugin;
use terrain::TerrainPlugin;
use turret::TurretPlugin;
#[derive(Component)]
pub struct GameObjectMarker;

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
        // .add_plugins(SwarmPlugin)
        // .add_plugins(ObstaclePlugin);
        // .add_plugins(TestMeshPlugin);
        .add_plugins(BotPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(MissilePlugin)
        .add_plugins(MapOnePlugin)
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
                // bots::setup,
                camera::setup,
                swarm::setup,
                turret::setup,
            )
                .in_set(SetupSet::InGame),
        )
        .add_systems(
            Update,
            (in_game_state_action, visualize_oct_tree)
                .chain()
                .run_if(in_state(GameState::Game)),
        )
        .add_systems(
            OnExit(GameState::Game),
            despawn_game_entities::<GameObjectMarker>,
        );
    }
}

#[derive(Component)]
pub struct OctNodeMarker;

fn visualize_oct_tree(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut gizmos: Gizmos,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &GlobalTransform, &ColliderInfo), With<ColliderMarker>>,
) {
    let mut oct_tree = OctTree::default();
    for (entity, gt, ci) in query.iter() {
        let mut radius: f32 = 0.;
        let center = gt.translation();
        match ci.collider_type {
            ColliderType::Sphere => {
                radius = ci.collider.read().unwrap().get_radius().unwrap();
                oct_tree
                    .pending_insertions
                    .write()
                    .unwrap()
                    .push(NodeEntities {
                        entity,
                        center,
                        radius,
                    });
            }
            _ => (),
        }
    }
    oct_tree.build_tree();
    let mut q: VecDeque<OctNode> = VecDeque::new();
    q.push_back(oct_tree.root.as_ref().clone());
    let mat = MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgba(0.1, 0., 0., 0.5),
        ..Default::default()
    }));
    while !q.is_empty() {
        let node = q.pop_front().unwrap();
        let cube = meshes.add(Cuboid {
            half_size: Vec3::splat(node.half_length),
        });
        // gizmos.cuboid(
        //     Transform::from_translation(node.center).with_scale(Vec3::splat(node.half_length * 2.)),
        //     Color::WHITE,
        // );
        // commands.spawn((
        //     Mesh3d(cube),
        //     mat.clone(),
        //     Transform::from_translation(node.center),
        //     OctNodeMarker,
        // ));
        if node.children.is_some() {
            for n in node.children.unwrap() {
                q.push_back(n.read().unwrap().clone());
            }
        }
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
            _ => (),
        }
    }
}

fn despawn_game_entities<T: Component>(
    mut commands: Commands,
    query: Query<Entity, With<T>>,
    mut menu_state: ResMut<NextState<MenuState>>,
) {
    for ent in query.iter() {
        commands.entity(ent).despawn_recursive();
    }
    menu_state.set(MenuState::Loading);
}
