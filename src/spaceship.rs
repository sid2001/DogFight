use crate::asset_loader::SceneAssets;
use crate::movement::{Direction, Inertia, Position, Velocity};
use bevy::audio::PlaybackMode::*;
use bevy::prelude::*;

const DEFAULT_HEALTH: f32 = 100.0;
const DEFAULT_THRUST: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const DEFAULT_SPAWN: Vec3 = Vec3::ZERO;
const DEFAULT_DIRECTION: Vec3 = Vec3::Y;
// const STARTING_TRANSLATION: Vec3 = Vec3::new(0.0, 20.0, 0.0);

#[derive(Resource, Default)]
pub struct Entities {
    pub player: Option<Entity>,
    pub camera: Option<Entity>,
}

#[derive(Component)]
pub struct SpaceShip;

#[derive(Component)]
pub struct Health(f32);

#[derive(Bundle)]
pub struct SpaceShipBundle {
    pub health: Health,
    pub marker: SpaceShip,
    pub position: Position,
    pub inertia: Inertia,
    pub direction: Direction,
    pub model: SceneBundle,
    pub audio: AudioBundle,
}
pub struct SpaceShipPlugin;

impl Plugin for SpaceShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_spaceship).add_systems(
            Update,
            (
                spaceship_controls,
                accelerate_spaceship,
                move_spaceship,
                spaceship_orientation,
            )
                .chain(),
        );
    }
}

fn spaceship_controls(
    keys: Res<Input<KeyCode>>,
    mut spaceship_query: Query<(&mut Inertia, &mut Direction), With<SpaceShip>>,
    time: Res<Time>,
    entity: Res<Entities>,
) {
    let (ref mut inertia, ref mut dir) = spaceship_query
        .get_mut(entity.player.unwrap())
        .expect("Can't get entity!");
    if keys.just_pressed(KeyCode::J) {
        if inertia.thrust != 2.0 {
            inertia.thrust += 1.0;
        }
    }
    if keys.just_pressed(KeyCode::K) {
        if inertia.thrust != -1.0 {
            inertia.thrust -= 1.0;
        }
    }

    if keys.pressed(KeyCode::Space) {
        let Vec3 { x, y, z } = inertia.velocity.0;
        inertia.velocity.0.x = if x.abs() < 0.1 {
            0.0
        } else {
            x / (1.0 + 1. * time.delta_seconds())
        };
        inertia.velocity.0.y = if y.abs() < 0.1 {
            0.0
        } else {
            y / (1.0 + 1. * time.delta_seconds())
        };
        inertia.velocity.0.z = if z.abs() < 0.1 {
            0.0
        } else {
            z / (1.0 + 1. * time.delta_seconds())
        };
    }
}

fn accelerate_spaceship(
    mut spaceship_query: Query<&mut Inertia, With<SpaceShip>>,
    time: Res<Time>,
    entity: Res<Entities>,
) {
    let mut inertia = spaceship_query
        .get_mut(entity.player.unwrap())
        .expect("Can't get entitiy!");

    inertia.velocity.0.y += inertia.thrust * time.delta_seconds();
    inertia.velocity.0.x += inertia.thrust * time.delta_seconds();
    inertia.velocity.0.z += inertia.thrust * time.delta_seconds();
}

fn move_spaceship(
    mut query: Query<(&mut Transform, &Inertia), With<SpaceShip>>,
    entity: Res<Entities>,
    time: Res<Time>,
) {
    let (mut trans, iner) = query
        .get_mut(entity.player.unwrap())
        .expect("Error getting entity player!");

    trans.translation.x += iner.velocity.0.x * time.delta_seconds();
    trans.translation.y += iner.velocity.0.y * time.delta_seconds();
    trans.translation.z += iner.velocity.0.z * time.delta_seconds();
}

fn spaceship_orientation(
    mut query: Query<(&mut Transform, &Inertia), With<SpaceShip>>,
    entities: Res<Entities>,
    time: Res<Time>,
) {
    let (ref mut trans, iner) = query
        .get_mut(entities.player.unwrap())
        .expect("Cannot get player entity!");
    let mut curr_dir = Vec3::ZERO;

    let mut target_dir = if iner.velocity.0.length() != 0.0 {
        curr_dir += trans.up() + trans.forward() + trans.right();
        curr_dir = curr_dir.normalize_or_zero();
        -iner.velocity.0.clone().normalize()
    } else {
        trans.forward()
    };
    let res_vector = curr_dir + target_dir;
    target_dir = curr_dir + res_vector.normalize_or_zero() * time.delta_seconds();
    // target_dir.normalize_or_zero();
    info!("{:?}", curr_dir);
    let rotation = Quat::from_rotation_arc(curr_dir, target_dir.normalize_or_zero());

    trans.rotate(rotation);
}

fn spawn_spaceship(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    asset_server: Res<AssetServer>,
    mut entities: ResMut<Entities>,
) {
    info!("spawning spacehip");
    entities.player = Some(
        commands
            .spawn((SpaceShipBundle {
                health: Health(DEFAULT_HEALTH),
                marker: SpaceShip,
                position: Position(DEFAULT_SPAWN.clone()),
                inertia: Inertia {
                    thrust: 0.,
                    ..default()
                },
                direction: Direction(DEFAULT_DIRECTION.clone()),
                model: SceneBundle {
                    scene: scene_assets.spaceship.clone(),
                    transform: Transform::from_xyz(0.0, 2.0, -4.0).looking_at(Vec3::ZERO, Vec3::Y),
                    ..default()
                },
                audio: AudioBundle {
                    source: asset_server.load("sounds/ambient-spacecraft-hum-33119.ogg"),
                    settings: PlaybackSettings {
                        mode: Loop,
                        paused: false,
                        ..default()
                    },
                },
            },))
            .id(),
    );
}
