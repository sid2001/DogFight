use super::movement::{Direction, Drag, Inertia, Position};
use super::turret::*;
use crate::asset_loader::{AudioAssets, SceneAssets};
use crate::events::{ThrottleDownEvent, ThrottleUpEvent};
use crate::sets::*;
use crate::states::*;
use bevy::audio::{PlaybackMode::*, Volume};
use bevy::prelude::*;

const DEFAULT_HEALTH: f32 = 100.0;
const DEFAULT_THRUST: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const DEFAULT_SPAWN: Vec3 = Vec3::ZERO;
const DEFAULT_ANGULAR_CHANGE: f32 = 50.0;
const DEFAULT_STEERING_BOOST: f32 = 0.;
const DEFAULT_ROLL_BOOST: f32 = 60.;
const DEFAULT_THRUST_LIMIT: f32 = 18.0;
const DEFAULT_ROLL_ANGULAR_CHANGE: f32 = 100.0;
const DEFAULT_DIRECTION: (Vec3, Vec3) = (Vec3::Y, Vec3::X);
const DEFAULT_DRAG: Vec3 = Vec3::new(0.2, 0.2, 0.2);
const DEFAULT_SPEED_LIMIT: f32 = 1.5;
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
    pub model: SceneRoot,
    pub transform: Transform,
    // pub engine_audio: AudioBundle,
    pub playback_settings: PlaybackSettings,
    pub throttle_audio: AudioPlayer,
    pub drag: Drag,
}
pub struct SpaceShipPlugin;

impl Plugin for SpaceShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            spawn_spaceship
                .in_set(SetupSet::InGame(InGameSet::SpaceShip))
                .run_if(in_state(GameState::InGame(InGameStates::Play))),
        )
        .add_systems(
            Update,
            (
                spaceship_controls.in_set(InputSet::InGame(Controls::InGame(InGameSet::SpaceShip))),
                accelerate_spaceship,
                move_spaceship,
                spaceship_orientation,
                // adjust_drag,
            )
                .chain()
                .in_set(UpdateSet::InGame(InGameSet::SpaceShip))
                .run_if(in_state(GameState::InGame(InGameStates::Play))),
            // .after(SetupSet::InGame(InGameSet::SpaceShip))
        );
    }
}

fn spaceship_controls(
    mut spaceship_query: Query<(&mut Inertia, &mut Direction), With<SpaceShip>>,
    mut turret_query: Query<(Entity, &mut Turret), (With<TurretMarker>, Without<SpaceShip>)>,
    mut ev_throttle_up: EventWriter<ThrottleUpEvent>,
    mut ev_throttle_down: EventWriter<ThrottleDownEvent>,
    mut ev_turret_off: EventWriter<ShootTurretEventOff>,
    mut ev_turret_on: EventWriter<ShootTurretEventOn>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    entity: Res<Entities>,
) {
    let (ref mut inertia, ref mut dir) = spaceship_query
        .get_mut(entity.player.unwrap())
        .expect("Can't get entity!");

    if keys.just_pressed(KeyCode::KeyJ) {
        // if inertia.thrust == 0. {
        // }
        if inertia.thrust != DEFAULT_THRUST_LIMIT {
            ev_throttle_up.send(ThrottleUpEvent(entity.player.unwrap()));
            inertia.thrust += 2.0;
        }
    }
    if keys.just_pressed(KeyCode::KeyK) {
        if inertia.thrust != -DEFAULT_THRUST_LIMIT {
            inertia.thrust -= 2.0;
            ev_throttle_down.send(ThrottleDownEvent(entity.player.unwrap()));
        }
        // if inertia.thrust == 0. {
        // }
    }

    {
        let mut ang = DEFAULT_ANGULAR_CHANGE;
        let mut ang_roll = DEFAULT_ROLL_ANGULAR_CHANGE;
        if keys.pressed(KeyCode::ShiftLeft) {
            ang += DEFAULT_STEERING_BOOST;
            ang_roll += DEFAULT_ROLL_BOOST;
        }
        if keys.pressed(KeyCode::KeyS) {
            // let target = dir.0.cross(dir.1);
            let rotation = Quat::from_axis_angle(dir.1, ang.to_radians() * time.delta_secs());
            dir.0 = rotation.mul_vec3(dir.0);
        }

        if keys.pressed(KeyCode::KeyW) {
            let rotation = Quat::from_axis_angle(-dir.1, ang.to_radians() * time.delta_secs());
            dir.0 = rotation.mul_vec3(dir.0);
        }
        if keys.pressed(KeyCode::KeyA) {
            let rotation = Quat::from_axis_angle(-dir.0, ang_roll.to_radians() * time.delta_secs());
            dir.1 = rotation.mul_vec3(dir.1);
        }

        if keys.pressed(KeyCode::KeyD) {
            let rotation = Quat::from_axis_angle(dir.0, ang_roll.to_radians() * time.delta_secs());
            dir.1 = rotation.mul_vec3(dir.1);
        }
    }
    if keys.pressed(KeyCode::Space) {
        let Vec3 { x, y, z } = inertia.velocity.0;
        inertia.velocity.0.x = if x.abs() < 0.1 {
            0.0
        } else {
            x / (1.0 + 1. * time.delta_secs())
        };
        inertia.velocity.0.y = if y.abs() < 0.1 {
            0.0
        } else {
            y / (1.0 + 1. * time.delta_secs())
        };
        inertia.velocity.0.z = if z.abs() < 0.1 {
            0.0
        } else {
            z / (1.0 + 1. * time.delta_secs())
        };
    }

    if keys.pressed(KeyCode::KeyL) {
        for (ent, mut tur) in turret_query.iter_mut() {
            error!("relesed");
            tur.0.shooting = true;
            tur.0.bullet_inertial_velocity = inertia.velocity.0.clone();
            ev_turret_on.send(ShootTurretEventOn(ent.clone()));
        }
    }
    if keys.just_released(KeyCode::KeyL) {
        for (ent, mut tur) in turret_query.iter_mut() {
            tur.0.shooting = false;
            ev_turret_off.send(ShootTurretEventOff(ent.clone()));
        }
    }
}

fn accelerate_spaceship(
    mut spaceship_query: Query<(&mut Inertia, &mut Drag, &Direction), With<SpaceShip>>,
    time: Res<Time>,
    entity: Res<Entities>,
) {
    let (mut inertia, mut drag, dir) = spaceship_query
        .get_mut(entity.player.unwrap())
        .expect("Can't get entitiy!");

    let acc = dir.0.normalize_or_zero() * inertia.thrust;
    // info!("acc: {:?}", acc);
    // info!("vel: {:?}", inertia.velocity.0);

    let Vec3 {
        mut x,
        mut y,
        mut z,
    } = inertia.velocity.0.clone() + acc * time.delta_secs();

    y = if y.abs() <= drag.0.y * time.delta_secs() {
        0.0
    } else {
        (y / y.abs()) * (y.abs() - drag.0.y * time.delta_secs())
    };
    x = if x.abs() <= drag.0.x * time.delta_secs() {
        0.0
    } else {
        (x / x.abs()) * (x.abs() - drag.0.x * time.delta_secs())
    };
    z = if z.abs() <= drag.0.z * time.delta_secs() {
        0.0
    } else {
        (z / z.abs()) * (z.abs() - drag.0.z * time.delta_secs())
    };
    inertia.velocity.0 = Vec3 { x, y, z };

    //* come up with a better drag function
    drag.0 = (inertia.velocity.0.clone() * 2.).abs();
}

fn move_spaceship(
    mut query: Query<(&mut Transform, &Inertia), With<SpaceShip>>,
    entity: Res<Entities>,
    time: Res<Time>,
) {
    let (mut trans, iner) = query
        .get_mut(entity.player.unwrap())
        .expect("Error getting entity player!");

    trans.translation += iner.velocity.0 * time.delta_secs();
    info!("{}", iner.velocity.0.length());
}

fn spaceship_orientation(
    mut query: Query<(&mut Transform, &Direction), With<SpaceShip>>,
    entities: Res<Entities>,
    // time: Res<Time>,
) {
    let (ref mut trans, dir) = query
        .get_mut(entities.player.unwrap())
        .expect("Cannot get player entity!");

    // let mut curr_dir = Vec3::ZERO;
    let curr_dir = trans.forward();
    let target_dir = dir.0.clone();

    let roll_curr = trans.right();
    let roll_target = dir.1.clone();

    // info!("{:?}", curr_dir);
    // info!("{:?}", target_dir);

    let rotation =
        Quat::from_rotation_arc(curr_dir.normalize_or_zero(), target_dir.normalize_or_zero());

    let roll = Quat::from_rotation_arc(
        roll_curr.normalize_or_zero(),
        roll_target.normalize_or_zero(),
    );
    trans.rotate(rotation);
    trans.rotate(roll);
}

pub fn spawn_spaceship(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    audio_assets: Res<AudioAssets>,
    mut entities: ResMut<Entities>,
) {
    if let Some(spaceship_scene) = scene_assets.spaceship.clone().into() {
        info!("spawning spacehip");
        commands.spawn((
            AudioPlayer(audio_assets.engine_humming.clone()),
            PlaybackSettings {
                mode: Loop,
                paused: false,
                volume: Volume::new(2.),
                ..default()
            },
        ));
        entities.player = Some(
            commands
                .spawn((SpaceShipBundle {
                    health: Health(DEFAULT_HEALTH),
                    marker: SpaceShip,
                    position: Position(DEFAULT_SPAWN.clone()),
                    drag: Drag(DEFAULT_DRAG.clone()),
                    inertia: Inertia {
                        thrust: 0.,
                        ..default()
                    },
                    direction: Direction(DEFAULT_DIRECTION.0.clone(), DEFAULT_DIRECTION.1.clone()),
                    model: SceneRoot(spaceship_scene),
                    transform: Transform::from_translation(Vec3::new(0., 10., 0.))
                        // .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
                        .with_translation(Vec3::new(0.0, 0., 0.0))
                        .with_scale(Vec3::new(0.5, 0.5, 0.5))
                        .looking_at(Vec3::Y, Vec3::Z),
                    throttle_audio: AudioPlayer(audio_assets.throttle_up.clone()),
                    playback_settings: PlaybackSettings {
                        mode: Loop,
                        paused: true,
                        volume: Volume::new(0.0),
                        ..default()
                    },
                },))
                .with_children(|parent| {
                    parent.spawn((
                        Transform::from_xyz(0.085, 0., 0.16)
                            .with_scale(Vec3::new(0.001, 0.001, 0.001)),
                        Turret(TurretBundle {
                            shooting: false,
                            speed: 10.,
                            bullet_size: 0.0001,
                            ..default()
                        }),
                        AudioPlayer(audio_assets.laser_turret.clone()),
                        PlaybackSettings {
                            mode: Loop,
                            paused: true,
                            ..default()
                        },
                        TurretMarker,
                        SceneRoot(scene_assets.player_turret.clone()),
                    ));
                })
                .with_children(|parent| {
                    parent.spawn((
                        Transform::from_xyz(-0.085, 0., 0.16)
                            .with_scale(Vec3::new(0.001, 0.001, 0.001)),
                        Turret(TurretBundle {
                            shooting: false,
                            speed: 10.,
                            bullet_size: 0.0001,
                            ..default()
                        }),
                        TurretMarker,
                        SceneRoot(scene_assets.player_turret.clone()),
                    ));
                })
                .id(),
        );
    } else {
        info!("Asset not loaded!")
    }
}
