use super::bots::*;
use super::collider::*;
use super::explosion::ExplosibleObjectMarker;
use super::explosion::{Explosion, ExplosionEvent};
use super::movement::{Direction, Drag, Inertia, Position};
use super::swarm::*;
use super::turret::*;
use crate::asset_loader::{AudioAssets, SceneAssets};
use crate::controls::Controls;
use crate::events::{ThrottleDownEvent, ThrottleUpEvent};
use crate::sets::*;
use crate::states::*;
use bevy::audio::{PlaybackMode::*, Volume};
use bevy::ecs::query;
use bevy::prelude::*;
use bevy::utils::info;
use std::sync::{Arc, RwLock};

const DEFAULT_HEALTH: f32 = 100.0;
const DEFAULT_THRUST: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const DEFAULT_SPAWN: Vec3 = Vec3::ZERO;
const DEFAULT_ANGULAR_CHANGE: f32 = 50.0;
const DEFAULT_STEERING_BOOST: f32 = 30.;
const DEFAULT_ROLL_BOOST: f32 = 60.;
const DEFAULT_THRUST_LIMIT: f32 = 30.0;
const DEFAULT_ROLL_ANGULAR_CHANGE: f32 = 100.0;
const DEFAULT_DIRECTION: (Vec3, Vec3) = (Vec3::Y, Vec3::X);
const DEFAULT_DRAG: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const DEFAULT_SPEED_LIMIT: f32 = 1.5;
// const STARTING_TRANSLATION: Vec3 = Vec3::new(0.0, 20.0, 0.0);

#[derive(Resource, Default)]
pub struct Entities {
    pub player: Option<Entity>,
    pub camera: Option<Entity>,
    pub turret: Option<Entity>,
}

#[derive(Component)]
pub struct SpaceShip;

#[derive(Component)]
pub struct SpaceShipTurret;

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
        app
            // .add_systems(
            //     Startup,
            //     setup
            //         .in_set(SetupSet::InGame(InGameSet::SpaceShip))
            //         .run_if(in_state(GameState::InGame(InGameStates::Play))),
            // )
            .add_systems(
                Update,
                (
                    collision_response,
                    spaceship_controls
                        .in_set(InputSet::InGame(ControlsSet::InGame(InGameSet::SpaceShip))),
                    accelerate_spaceship,
                    move_spaceship,
                    spaceship_orientation,
                    // adjust_drag,
                )
                    .after(SetupSet::InGame)
                    .chain()
                    .in_set(UpdateSet::InGame), // .run_if(in_state(GameState::InGame(InGameStates::Play))), // .after(SetupSet::InGame(InGameSet::SpaceShip)),
            )
            .add_systems(
                Update,
                shoot_turret::<SpaceShipTurret>
                    // .after(InputSet::InGame(Controls::InGame(InGameSet::SpaceShip)))
                    .after(UpdateSet::InGame),
            );
    }
}

fn spaceship_controls(
    mut spaceship_query: Query<(Entity, &mut Inertia, &mut Direction), With<SpaceShip>>,
    mut turret_query: Query<(Entity, &mut Turret), (With<SpaceShipTurret>, Without<SpaceShip>)>,
    mut ev_throttle_up: EventWriter<ThrottleUpEvent>,
    mut ev_throttle_down: EventWriter<ThrottleDownEvent>,
    mut ev_turret_off: EventWriter<ShootTurretEventOff>,
    mut ev_turret_on: EventWriter<ShootTurretEventOn>,
    controls: Res<Controls>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    entity: Res<Entities>,
) {
    let (sp_ent, ref mut inertia, ref mut dir) = spaceship_query
        .get_mut(entity.player.unwrap())
        .expect("Can't get entity!");

    if keys.just_pressed(controls.thrust.unwrap()) {
        // if inertia.thrust == 0. {
        // }
        if inertia.thrust != DEFAULT_THRUST_LIMIT {
            inertia.thrust += 2.0;
            ev_throttle_up.send(ThrottleUpEvent(sp_ent.clone()));
        }
    }
    if keys.just_pressed(controls.back_thrust.unwrap()) {
        if inertia.thrust != -DEFAULT_THRUST_LIMIT {
            inertia.thrust -= 2.0;
            ev_throttle_down.send(ThrottleDownEvent(sp_ent.clone()));
        }
    }

    {
        let mut ang = DEFAULT_ANGULAR_CHANGE;
        let mut ang_roll = DEFAULT_ROLL_ANGULAR_CHANGE;
        if keys.pressed(controls.steer_boost.unwrap()) {
            ang += DEFAULT_STEERING_BOOST;
            ang_roll += DEFAULT_ROLL_BOOST;
        }
        if keys.pressed(controls.down.unwrap()) {
            // let target = dir.0.cross(dir.1);
            let rotation = Quat::from_axis_angle(dir.1, ang.to_radians() * time.delta_secs());
            dir.0 = rotation.mul_vec3(dir.0);
        }

        if keys.pressed(controls.up.unwrap()) {
            let rotation = Quat::from_axis_angle(-dir.1, ang.to_radians() * time.delta_secs());
            dir.0 = rotation.mul_vec3(dir.0);
        }
        if keys.pressed(controls.roll_l.unwrap()) {
            let rotation = Quat::from_axis_angle(-dir.0, ang_roll.to_radians() * time.delta_secs());
            dir.1 = rotation.mul_vec3(dir.1);
        }

        if keys.pressed(controls.roll_r.unwrap()) {
            let rotation = Quat::from_axis_angle(dir.0, ang_roll.to_radians() * time.delta_secs());
            dir.1 = rotation.mul_vec3(dir.1);
        }
    }
    if keys.pressed(controls.brake.unwrap()) {
        let Vec3 {
            mut x,
            mut y,
            mut z,
        } = inertia.velocity.0;
        x = if x.abs() < 0.1 {
            0.0
        } else {
            x / (1.0 + 1. * time.delta_secs())
        };
        y = if y.abs() < 0.1 {
            0.0
        } else {
            y / (1.0 + 1. * time.delta_secs())
        };
        z = if z.abs() < 0.1 {
            0.0
        } else {
            z / (1.0 + 1. * time.delta_secs())
        };
        inertia.velocity.0 = Vec3 { x, y, z };
    }

    if keys.pressed(controls.shoot.unwrap()) {
        for (ent, mut tur) in turret_query.iter_mut() {
            // error!("relesed");
            tur.0.shooting = true;
            tur.0.bullet_inertial_velocity = inertia.velocity.0.clone();
            ev_turret_on.send(ShootTurretEventOn(ent.clone()));
        }
    }
    if keys.just_released(controls.shoot.unwrap()) {
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

    let Vec3 { x, y, z } = inertia.velocity.0.clone() + (acc + drag.0) * time.delta_secs();

    inertia.velocity.0 = Vec3 { x, y, z };

    //* come up with a better drag function
    drag.0 = -inertia.velocity.0.clone() * 2.;
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
    // info!("{}", iner.velocity.0.length());
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

fn collision_response(
    mut query: Query<(Entity, &Transform, &ColliderInfo, &mut Health), With<SpaceShip>>,
    mut ev_reader: EventReader<CollisionEvents>,
    mut ev_explode: EventWriter<ExplosionEvent>,
) {
    // let health = query.single();
    for msg in ev_reader.read() {
        match msg {
            CollisionEvents::TakeDamage(e, d) => {
                if let Ok((ent, trans, collider, mut health)) = query.get_mut(e.clone()) {
                    if d.from.is_some_and(|e| e != ent) || d.from.is_none() {
                        if health.0 <= 0. {
                            continue;
                        }
                        health.0 -= d.damage;
                        if health.0 <= 0. {
                            ev_explode.send(ExplosionEvent {
                                transform: trans.clone(),
                                explosion: Explosion {
                                    half_extent: 0.15,
                                    ..default()
                                },
                            });
                        }
                        info!("Health {}", health.0);
                    }
                }
            }
        }
    }
}

pub fn setup(
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
        let listener = SpatialListener::new(1.);
        entities.player = Some(
            commands
                .spawn((
                    SpaceShipBundle {
                        model: SceneRoot(spaceship_scene),
                        health: Health(DEFAULT_HEALTH),
                        marker: SpaceShip,
                        position: Position(DEFAULT_SPAWN.clone()),
                        drag: Drag(DEFAULT_DRAG.clone()),
                        inertia: Inertia {
                            thrust: 0.,
                            ..default()
                        },
                        direction: Direction(
                            DEFAULT_DIRECTION.0.clone(),
                            DEFAULT_DIRECTION.1.clone(),
                        ),
                        transform: Transform::from_translation(Vec3::new(0., -6., 0.))
                            // .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
                            // .with_translation(Vec3::new(0.0, 5., 0.0))
                            // .with_scale(Vec3::new(1., 1., 1.))
                            .looking_at(Vec3::ZERO, Vec3::Z),
                        throttle_audio: AudioPlayer(audio_assets.throttle_up.clone()),
                        playback_settings: PlaybackSettings {
                            mode: Loop,
                            paused: true,
                            volume: Volume::new(0.0),
                            ..default()
                        },
                    },
                    SwarmTarget,
                    TargetMarker,
                    CollisionDamage {
                        damage: 10.,
                        from: None,
                    },
                    ColliderInfo {
                        collider_type: ColliderType::Sphere,
                        collider: Arc::new(RwLock::new(SphericalCollider {
                            radius: 0.3,
                            center: Vec3::ZERO,
                        })),
                    },
                    ColliderMarker,
                    ExplosibleObjectMarker,
                    listener.clone(),
                ))
                .with_children(|parent| {
                    entities.turret = Some(
                        parent
                            .spawn((
                                Transform::from_xyz(0.085, 0., 0.16)
                                    .with_scale(Vec3::new(0.001, 0.001, 0.001)),
                                Turret(TurretBundle {
                                    shooting: false,
                                    speed: 10.,
                                    bullet_size: 0.0002,
                                    shooter: Some(parent.parent_entity()),
                                    ..default()
                                }),
                                AudioPlayer(audio_assets.laser_turret.clone()),
                                PlaybackSettings {
                                    mode: Loop,
                                    spatial: true,
                                    paused: true,
                                    ..default()
                                },
                                TurretMarker,
                                SpaceShipTurret,
                                SceneRoot(scene_assets.player_turret.clone()),
                            ))
                            .id(),
                    );

                    parent.spawn((
                        Transform::from_xyz(-0.085, 0., 0.16)
                            .with_scale(Vec3::new(0.001, 0.001, 0.001)),
                        Turret(TurretBundle {
                            shooting: false,
                            speed: 10.,
                            bullet_size: 0.0002,
                            shooter: Some(parent.parent_entity()),
                            ..default()
                        }),
                        TurretMarker,
                        SpaceShipTurret,
                        SceneRoot(scene_assets.player_turret.clone()),
                    ));
                })
                .with_children(|parent| {
                    parent.spawn((Transform::from_translation(listener.left_ear_offset),));
                    parent.spawn((Transform::from_translation(listener.right_ear_offset),));
                })
                .id(),
        );
    } else {
        info!("Asset not loaded!")
    }
}
