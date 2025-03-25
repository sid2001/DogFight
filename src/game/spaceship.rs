use super::bots::*;
use super::collider::*;
use super::explosion::ExplosibleObjectMarker;
use super::explosion::{Explosion, ExplosionEvent};
use super::missile::Missile;
use super::missile::{HomingMissileShootEvent, HomingMissileTarget, *};
use super::movement::{Direction, Drag, Inertia, Position};
use super::swarm::*;
use super::turret::*;
use crate::asset_loader::{AudioAssets, SceneAssets};
use crate::controls::Controls;
use crate::events::{ThrottleDownEvent, ThrottleUpEvent};
use crate::game::missile::{HomingMissileLauncher, MissileType, SwarmMissileLauncher};
use crate::game::GameObjectMarker;
use crate::sets::*;
use crate::states::*;
use bevy::audio::{PlaybackMode::*, Volume};
use bevy::ecs::query;
use bevy::gizmos;
use bevy::input::common_conditions::input_just_pressed;
use bevy::log::tracing_subscriber::fmt::writer::OrElse;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::state::commands;
use bevy::utils::info;
use rand::Rng;
use std::f32::consts::PI;
use std::sync::{Arc, RwLock};
use std::time::Duration;

const DEFAULT_HEALTH: f32 = 100.0;
const DEFAULT_THRUST: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const DEFAULT_SPAWN: Vec3 = Vec3::ZERO;
const DEFAULT_ANGULAR_CHANGE: f32 = 50.0;
const DEFAULT_STEERING_BOOST: f32 = 30.;
const DEFAULT_ROLL_BOOST: f32 = 60.;
const DEFAULT_THRUST_LIMIT: f32 = 10.0;
const DEFAULT_ROLL_ANGULAR_CHANGE: f32 = 100.0;
const DEFAULT_DIRECTION: (Vec3, Vec3) = (Vec3::Y, Vec3::X);
const DEFAULT_DRAG: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const DEFAULT_SPEED_LIMIT: f32 = 1.5;
const DEFAULT_ROLL_THRUST: f32 = 180.;

const BOT_MISSILE_OFFSET: Transform = Transform {
    translation: Vec3::new(0., -0.08, -0.1),
    scale: Vec3::splat(0.5),
    rotation: Quat::IDENTITY,
};
const SWARM_MISSILE_OFFSET_LEFT: Transform = Transform {
    translation: Vec3::new(0.215, 0.004, 0.01),
    scale: Vec3::splat(0.3),
    rotation: Quat::IDENTITY,
};
const SWARM_MISSILE_OFFSET_RIGHT: Transform = Transform {
    translation: Vec3::new(-0.215, 0.004, 0.01),
    scale: Vec3::splat(0.3),
    rotation: Quat::IDENTITY,
};
// const STARTING_TRANSLATION: Vec3 = Vec3::new(0.0, 20.0, 0.0);

#[derive(Resource, Default)]
pub struct Entities {
    pub player: Option<Entity>,
    pub camera: Option<Entity>,
    pub turret: Option<Entity>,
}

#[derive(Component)]
pub struct SpaceShip;

#[derive(Resource)]
pub struct MissileEquipped(MissileType);

#[derive(Component)]
pub struct SpaceShipTurret;

#[derive(Component)]
pub struct Health(pub f32);

impl Health {
    fn new(x: f32) -> Self {
        Self(x)
    }
}

#[derive(Resource, Default)]
pub struct SpaceShipMissileLauncher {
    pub homing_launcher: Option<Entity>,
    pub swarm_launcher_right: Option<Entity>,
    pub swarm_launcher_left: Option<Entity>,
}

#[derive(Resource)]
pub struct SpaceShipHomingTarget(Option<Entity>, Duration);
impl SpaceShipHomingTarget {
    fn reset(&mut self) {
        self.0 = None;
        self.1 = Duration::ZERO;
    }
}

#[derive(Bundle)]
pub struct SpaceShipBundle {
    pub health: Health,
    pub marker: SpaceShip,
    pub position: Position,
    pub inertia: Inertia,
    pub direction: Direction,
    pub model: SceneRoot,
    pub transform: Transform,
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
            .add_systems(OnExit(GameState::Game), clean_resources)
            .add_systems(
                Update,
                (
                    aim_homing,
                    draw_aim_lock,
                    collision_response::<SpaceShip>,
                    spaceship_controls
                        .in_set(InputSet::InGame(ControlsSet::InGame(InGameSet::SpaceShip))),
                    accelerate_spaceship,
                    move_spaceship,
                    spaceship_orientation,
                    missile_control,
                    // adjust_drag,
                )
                    .after(SetupSet::InGame)
                    .chain()
                    .in_set(UpdateSet::InGame), // .run_if(in_state(GameState::InGame(InGameStates::Play))), // .after(SetupSet::InGame(InGameSet::SpaceShip)),
            )
            .add_systems(
                Update,
                shoot_turret::<SpaceShipTurret>.after(UpdateSet::InGame),
            );
    }
}

fn spaceship_controls(
    mut spaceship_query: Query<(Entity, &mut Inertia, &mut Direction), With<SpaceShip>>,
    mut turret_query: Query<(Entity, &mut Turret), (With<SpaceShipTurret>, Without<SpaceShip>)>,
    mut ev_throttle_up: EventWriter<ThrottleUpEvent>,
    mut ev_turret_off: EventWriter<ShootTurretEventOff>,
    mut ev_turret_on: EventWriter<ShootTurretEventOn>,
    mut missile_equipped: ResMut<MissileEquipped>,
    mut ev_missile: EventWriter<HomingMissileShootEvent>,
    missile_launcher: Res<SpaceShipMissileLauncher>,
    controls: Res<Controls>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    entity: Res<Entities>,
    mut homing_target: ResMut<SpaceShipHomingTarget>,
) {
    let (sp_ent, ref mut inertia, ref mut dir) = spaceship_query
        .get_mut(entity.player.unwrap())
        .expect("Can't get entity!");

    if keys.just_pressed(controls.thrust.unwrap()) {
        // if inertia.thrust == 0. {
        // }
        if inertia.thrust != DEFAULT_THRUST_LIMIT {
            inertia.thrust += 2.0;
            ev_throttle_up.send(ThrottleUpEvent(sp_ent.clone(), inertia.thrust));
        }
    }
    if keys.just_pressed(controls.back_thrust.unwrap()) {
        if inertia.thrust != -DEFAULT_THRUST_LIMIT {
            inertia.thrust -= 2.0;
            ev_throttle_up.send(ThrottleUpEvent(sp_ent.clone(), inertia.thrust));
        }
    }

    {
        let mut ang = DEFAULT_ANGULAR_CHANGE;
        let mut ang_roll = DEFAULT_ROLL_ANGULAR_CHANGE;
        // let mut ang_roll = inertia.angular_velocity;
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
            // inertia.angular_velocity =
            // inertia.angular_velocity - DEFAULT_ROLL_THRUST * time.delta_secs();
            let rotation = Quat::from_axis_angle(-dir.0, ang_roll.to_radians() * time.delta_secs());
            dir.1 = rotation.mul_vec3(dir.1);
        }
        if keys.pressed(controls.roll_r.unwrap()) {
            // inertia.angular_velocity =
            // inertia.angular_velocity + DEFAULT_ROLL_THRUST * time.delta_secs();
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
            if tur.0.overheat == true {
                tur.0.shooting = false;
                ev_turret_off.send(ShootTurretEventOff(ent.clone()));
                continue;
            }
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

    if keys.just_pressed(controls.missile_switch.unwrap()) {
        missile_equipped.0 = match missile_equipped.0 {
            MissileType::HomingMissile => {
                // clear target if switched while aiming
                homing_target.reset();
                MissileType::SwarmMissile
            }
            MissileType::SwarmMissile => MissileType::HomingMissile,
        }
    }
    if keys.just_pressed(controls.missile_shoot.unwrap()) {
        let missile = Missile {
            source: sp_ent,
            is_locked: homing_target.1.as_secs_f32() > 2.,
            target: homing_target.0,
            initial_speed: inertia.velocity.0.length(),
            thrust: 30.,
            timer: Duration::ZERO,
            damage: 100.,
            velocity: inertia.velocity.0,
            drag: Vec3::ZERO,
            angular_speed: 720.,
        };
        ev_missile.send(HomingMissileShootEvent {
            missile,
            launcher: missile_launcher.homing_launcher.unwrap(),
        });
    }
}

fn missile_control(
    mut ev_swarm_missile: EventWriter<SwarmMissileShootEvent>,
    query: Query<(&Inertia, &Transform), With<SpaceShip>>,
    missile_equipped: Res<MissileEquipped>,
    missile_launcher: Res<SpaceShipMissileLauncher>,
    s_query: Query<(Entity, &Transform), With<SwarmMissileTarget>>,
    keys: Res<ButtonInput<KeyCode>>,
    controls: Res<Controls>,
) {
    // will panic if more than two, will change later
    let (inertia, trans) = query.single();
    if keys.just_pressed(controls.missile_shoot.unwrap()) {
        match missile_equipped.0 {
            MissileType::SwarmMissile => {
                let mut rng = rand::rng();
                let mut dir;
                let mut axis = Vec3::Y;
                for (target, s_trans) in s_query.iter() {
                    if (s_trans.translation - trans.translation).length_squared() <= 100.0 {
                        loop {
                            let (x, y, z) = (
                                rng.random_range(-1.0..1.0),
                                rng.random_range(-1.0..1.0),
                                rng.random_range(-1.0..1.0),
                            );
                            if let Some(res) =
                                trans.forward().cross(Vec3::new(x, y, z)).try_normalize()
                            {
                                axis = res;
                                break;
                            }
                        }

                        let angle = (rng.random_range(80.0..90.0) as f32).to_radians();
                        {
                            let mut temp = trans.clone();
                            temp.rotate_axis(Dir3::new(axis).unwrap(), angle);
                            dir = temp.forward();
                        }

                        let missile = SwarmMissile {
                            stage: SwarmMissileStage::Stage1(dir),
                            angluar_speed: 120.,
                            initial_speed: inertia.velocity.0.length(),
                            converge_point: (trans.translation
                                + (angle * 3.0 / 30.) * trans.forward().as_vec3()),
                            speed: 0.,
                            timer: Duration::ZERO,
                            target: Some(target),
                        };
                        ev_swarm_missile.send(SwarmMissileShootEvent {
                            launcher: missile_launcher.swarm_launcher_left.unwrap(),
                            missile,
                        });
                    }
                }
            }
            MissileType::HomingMissile => {}
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

    // inertia.angular_velocity =
    //     inertia.angular_velocity - inertia.angular_velocity * 2. * time.delta_secs();

    //* come up with a better drag function
    drag.0 = -inertia.velocity.0.clone() * 2.;
}

fn aim_homing(
    controls: Res<Controls>,
    keys: Res<ButtonInput<KeyCode>>,
    missile_equipped: Res<MissileEquipped>,
    mut target: ResMut<SpaceShipHomingTarget>,
    hm_query: Query<&GlobalTransform, With<HomingMissileLauncher>>,
    launcher: Res<SpaceShipMissileLauncher>,
    ht_query: Query<(Entity, &Transform), With<HomingMissileTarget>>,
    time: Res<Time>,
) {
    if keys.pressed(controls.missile_aim.unwrap()) {
        match missile_equipped.0 {
            MissileType::HomingMissile => {
                // check laucher was already locking
                if target.0.is_some() {
                    // if that target still exists
                    if let Ok((_, t_trans)) = ht_query.get(target.0.unwrap()) {
                        // if launcher exists (in case the player dies while aiming)
                        if let Ok(l_trans) = hm_query.get(launcher.homing_launcher.unwrap()) {
                            let dir_vec = t_trans.translation - l_trans.translation();
                            let dist = dir_vec.length_squared();
                            let angle = l_trans.forward().dot(dir_vec.normalize_or_zero()).acos();
                            // if the target is still within the range and view
                            if dist < 900. && angle < PI / 2. {
                                target.1 += time.delta();
                            } else {
                                target.reset();
                            }
                        } else {
                            target.reset();
                        }
                    } else {
                        target.reset();
                    }
                }
                if target.0.is_none() {
                    // search for new target
                    let mut f_dist = 101.;
                    let mut f_ent = None;
                    if let Ok(l_trans) = hm_query.get(launcher.homing_launcher.unwrap()) {
                        for (ent, t_trans) in ht_query.iter() {
                            let dir_vec = t_trans.translation - l_trans.translation();
                            let dist = dir_vec.length_squared();
                            let angle = l_trans.forward().dot(dir_vec.normalize_or_zero()).acos();
                            if dist < 100. && angle < PI / 2. && dist < f_dist {
                                f_dist = dist;
                                f_ent = Some(ent);
                            }
                        }
                    }
                    if f_ent.is_some() {
                        target.0 = f_ent;
                    }
                }
            }
            MissileType::SwarmMissile => {}
        }
    }
    if keys.just_released(controls.missile_aim.unwrap()) {
        target.reset();
    }
}

fn draw_aim_lock(
    mut gizmos: Gizmos,
    l_target: Res<SpaceShipHomingTarget>,
    query: Query<&Transform, With<HomingMissileTarget>>,
) {
    if l_target.0.is_some() {
        if let Ok(trans) = query.get(l_target.0.unwrap()) {
            if l_target.1.as_secs_f32() < 2. {
                gizmos.cuboid(*trans, Color::linear_rgb(0., 255., 0.));
            } else {
                gizmos.cuboid(*trans, Color::linear_rgb(255., 0., 0.));
            }
        }
    }
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

fn clean_resources(mut commands: Commands) {
    commands.remove_resource::<MissileEquipped>();
    commands.remove_resource::<SpaceShipHomingTarget>();
    commands.remove_resource::<SpaceShipMissileLauncher>();
}

pub fn setup(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    audio_assets: Res<AudioAssets>,
    mut entities: ResMut<Entities>,
) {
    commands.insert_resource(MissileEquipped(MissileType::HomingMissile));
    commands.insert_resource(SpaceShipHomingTarget(None, Duration::ZERO));
    let mut launchers = SpaceShipMissileLauncher::default();
    if let Some(spaceship_scene) = scene_assets.spaceship.clone().into() {
        info!("spawning spacehip");
        commands.spawn((
            AudioPlayer(audio_assets.engine_humming.clone()),
            PlaybackSettings {
                mode: Loop,
                paused: false,
                volume: Volume::new(1.),
                ..default()
            },
            GameObjectMarker,
        ));

        let listener = SpatialListener::new(0.25);
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
                        transform: Transform::from_xyz(30., 30., 30.),
                        // .looking_at(Vec3::ZERO, Vec3::Z),
                        throttle_audio: AudioPlayer(audio_assets.throttle_up.clone()),
                        playback_settings: PlaybackSettings {
                            mode: Loop,
                            paused: true,
                            volume: Volume::new(0.0),
                            ..default()
                        },
                    },
                    Name::new("SpaceShip"),
                    GameObjectMarker,
                    SwarmTarget,
                    BotTargetMarker,
                    ColliderInfo {
                        collider_type: ColliderType::Sphere,
                        collider: Arc::new(RwLock::new(SphericalCollider {
                            radius: 0.3,
                            center: Vec3::ZERO,
                        })),
                        immune_to: None,
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
                    // parent.spawn((
                    //     SceneRoot(scene_assets.map_marker.clone()),
                    //     Name::new("marker"),
                    // ));
                })
                .id(),
        );
        let homing_launcher = Some(
            commands
                .spawn((
                    SceneRoot(scene_assets.missile2.clone()),
                    HomingMissileLauncher {
                        source: entities.player,
                        ..Default::default()
                    },
                    BOT_MISSILE_OFFSET,
                ))
                .id(),
        );

        let swarm_launcher_right = Some(
            commands
                .spawn((
                    SceneRoot(scene_assets.missile.clone()),
                    SwarmMissileLauncher {
                        source: entities.player,
                    },
                    SWARM_MISSILE_OFFSET_RIGHT,
                ))
                .id(),
        );
        let swarm_launcher_left = Some(
            commands
                .spawn((
                    SceneRoot(scene_assets.missile.clone()),
                    SwarmMissileLauncher {
                        source: entities.player,
                    },
                    SWARM_MISSILE_OFFSET_LEFT,
                ))
                .id(),
        );

        launchers = SpaceShipMissileLauncher {
            swarm_launcher_left,
            swarm_launcher_right,
            homing_launcher,
        };
        commands.entity(entities.player.unwrap()).add_children(&[
            swarm_launcher_left.unwrap(),
            swarm_launcher_right.unwrap(),
            homing_launcher.unwrap(),
        ]);
        commands
            .entity(entities.player.unwrap())
            .insert(CollisionDamage {
                damage: 10.,
                from: entities.player,
            });
        commands.insert_resource(launchers);
    } else {
        info!("Asset not loaded!")
    }
}
