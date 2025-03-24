use super::camera::REAR_VIEW_LAYERS;
use super::collider::*;
use super::collider::{self, CollisionDamage};
use super::explosion::{ExplosibleObjectMarker, ExplosionEvent};
use super::missile::HomingMissileTarget;
use super::spaceship::Health;
use crate::states::*;
use bevy::text::cosmic_text::BorrowedWithFontSystem;
use bevy::{math::VectorSpace, prelude::*};
use rand::Rng;
use std::sync::{Arc, RwLock};

use super::{turret::*, GameObjectMarker};
use crate::asset_loader::*;
use crate::sets::*;

use crate::asset_loader::SceneAssets;
use crate::states::GameState;

const SHOOT_VICINITY_ANGLE: f32 = 15.;
const SHOOT_VICINITY_DISTANCE: f32 = 20.;

// marks bot entities
#[derive(Component)]
pub struct BotMarker;

#[derive(Clone, PartialEq)]
pub enum BotTargetVicinity {
    Far,
    Around,
    Near,
}

// marks target entities
#[derive(Component)]
pub struct BotTargetMarker;

// marks projectile from target entities
#[derive(Component)]
pub struct TargetProjectileMarker;

// marks projectile from bot entities
#[derive(Component)]
pub struct BotProjectileMarker;

#[derive(Component)]
pub enum BotState {
    Ideal,
    Chasing,
    Dead,
    Searching,
    Evading,
    Dodge(Dir3),
}

#[derive(Component)]
pub struct BotMotion {
    pub acceleration: f32,
    pub drag: Vec3,
    pub angular_steer: f32,
    pub velocity: Vec3,
    pub direction: Vec3,
    pub nearest_obstacle: (f32, Dir3),
    pub last_dir: Option<Dir3>,
    pub target_vicinity: BotTargetVicinity,
}

impl Default for BotMotion {
    fn default() -> Self {
        Self {
            acceleration: 5.,
            drag: Vec3::ZERO,
            angular_steer: 40.,
            velocity: Vec3::ZERO,
            direction: Vec3::Z,
            nearest_obstacle: (f32::INFINITY, Dir3::Y),
            last_dir: None,
            target_vicinity: BotTargetVicinity::Around,
        }
    }
}

impl BotMotion {
    fn estimate_vicintiy(dist: f32) -> BotTargetVicinity {
        // let dist = (p1 - p2).length();
        if dist <= 3. {
            BotTargetVicinity::Near
        } else if dist <= 6. {
            BotTargetVicinity::Around
        } else {
            BotTargetVicinity::Far
        }
    }
}

#[derive(Resource)]
pub struct BotSpawner {
    pub active_bots: u32,
    pub capacity: u32,
    pub next_bot: u32,
    pub spawn_distance: f32,
}

impl Default for BotSpawner {
    fn default() -> Self {
        Self {
            active_bots: 0,
            capacity: 3,
            next_bot: 0,
            spawn_distance: 30.,
        }
    }
}

#[derive(Component)]
pub struct BotTurret;

#[derive(Component)]
pub struct Bot {
    pub level: u32,
    pub is_alive: bool,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            level: 1,
            is_alive: true,
        }
    }
}

// #[derive(Component)]
// pub struct Health(f32);

pub struct BotPlugin;
impl Plugin for BotPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BotSpawner::default())
            // .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    spawn_bots,
                    thrust_control,
                    chase_target,
                    avoid_crash,
                    collider::collision_response::<BotMarker>,
                    despawn_dead_bots,
                )
                    .chain()
                    .in_set(UpdateSet::InGame)
                    .run_if(in_state(GameState::Game)),
            )
            .add_systems(
                Update,
                (shoot_target, shoot_turret::<BotTurret>)
                    .chain()
                    .in_set(UpdateSet::InGame)
                    .run_if(in_state(GameState::Game)),
            );
    }
}

fn despawn_dead_bots(
    query: Query<(Entity, &Health), With<BotMarker>>,
    mut commands: Commands,
    mut bot_spawner: ResMut<BotSpawner>,
) {
    for (ent, health) in query.iter() {
        if health.0 <= 0. {
            bot_spawner.active_bots -= 1;
            commands.entity(ent).despawn_recursive();
        }
    }
}

fn spawn_bots(
    mut commands: Commands,
    mut bot_spawner: ResMut<BotSpawner>,
    query: Query<&Transform, With<BotTargetMarker>>,
    scene_assets: Res<SceneAssets>,
    audio_assets: Res<AudioAssets>,
) {
    while bot_spawner.active_bots < bot_spawner.capacity {
        bot_spawner.active_bots += 1;
        let mut bot_scene = scene_assets.bot_spaceship.clone();
        match bot_spawner.next_bot {
            1 => {
                bot_scene = scene_assets.bot_spaceship2.clone();
            }
            2 => {
                bot_scene = scene_assets.bot_spaceship3.clone();
            }
            _ => (),
        }
        bot_spawner.next_bot = (bot_spawner.next_bot + 1) % 3;

        // todo: this code line is not safe, it's only for temporary use
        let target = query.single().translation;
        let mut rng = rand::rng();
        let (x, y, z) = (
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        );
        let dir = Vec3::new(x, y, z).normalize_or(Vec3::Y);
        let spwan_point = target + (bot_spawner.spawn_distance * dir);

        let transform = Transform::from_translation(spwan_point)
            .looking_at(-dir, Vec3::Y)
            .with_scale(Vec3::splat(0.5));
        let bot = commands
            .spawn((
                SceneRoot(bot_scene),
                BotMotion::default(),
                BotState::Chasing,
                BotMarker,
                Bot::default(),
                REAR_VIEW_LAYERS,
                GameObjectMarker,
                Health(1000.0),
                HomingMissileTarget,
                ColliderMarker,
                ColliderInfo {
                    collider_type: ColliderType::Sphere,
                    collider: Arc::new(RwLock::new(SphericalCollider {
                        radius: 0.3,
                        center: Vec3::ZERO,
                    })),
                    immune_to: None,
                },
                ExplosibleObjectMarker,
                (
                    AudioPlayer(audio_assets.throttle_up.clone()),
                    PlaybackSettings::LOOP.with_spatial(true),
                ),
                transform,
            ))
            .id();
        commands.entity(bot).insert(CollisionDamage {
            damage: 100.,
            from: Some(bot),
        });
        commands.entity(bot).with_children(|parent| {
            parent.spawn((
                Transform::from_xyz(0., 0., 0.),
                Turret(TurretBundle {
                    shooting: false,
                    speed: 20.,
                    bullet_size: 0.0002,
                    shooter: Some(parent.parent_entity()),
                    ..default()
                }),
                GameObjectMarker,
                AudioPlayer(audio_assets.laser_turret.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    paused: true,
                    spatial: true,
                    ..default()
                },
                BotTurret,
                TurretMarker,
            ));
        });
    }
}

fn chase_target(
    target_query: Query<&Transform, With<BotTargetMarker>>,
    mut bot_query: Query<
        (&mut Transform, &mut BotState, &mut BotMotion),
        (With<BotMarker>, Without<BotTargetMarker>),
    >,
    time: Res<Time>,
) {
    let t_trans = target_query.single();

    for (mut trans, state, mut motion) in bot_query.iter_mut() {
        // let target_distance = (t_trans.translation.clone() - trans.translation.clone()).length();
        motion.target_vicinity =
            BotMotion::estimate_vicintiy((t_trans.translation - trans.translation).length());
        let t = time.delta_secs();
        match *state {
            BotState::Chasing => {
                let drag = motion.drag.clone();
                let velocity =
                    motion.direction.clone().normalize_or_zero() * motion.acceleration.clone() * t
                        + motion.velocity.clone()
                        + drag * t;
                motion.velocity = velocity;
                trans.translation += motion.velocity * t;
                motion.drag = -motion.velocity * 2.;
                // info!("Velocityy bot {}", motion.velocity.length());
            }
            _ => (),
        }
    }
}

fn thrust_control(mut query_bots: Query<&mut BotMotion, With<BotMarker>>, time: Res<Time>) {
    for mut bm in query_bots.iter_mut() {
        match bm.target_vicinity {
            BotTargetVicinity::Far => {
                if bm.acceleration < 10. {
                    bm.acceleration += 0.3 * time.delta_secs();
                }
            }
            BotTargetVicinity::Around => {
                if bm.acceleration > 2. {
                    bm.acceleration -= 0.6 * time.delta_secs();
                } else {
                    bm.acceleration = 2.
                }
                // sb.thrust = 1.;
            }
            BotTargetVicinity::Near => {
                if bm.acceleration > 0. {
                    bm.acceleration -= 0.6 * time.delta_secs();
                } else {
                    bm.acceleration = 0.;
                }
            }
        }
    }
}

fn avoid_crash(mut query: Query<(&mut BotMotion, &Transform), With<BotMarker>>, time: Res<Time>) {
    let mut bot_iter = query.iter_combinations_mut();

    while let Some([(mut bm1, t1), (_, t2)]) = bot_iter.fetch_next() {
        let diff_vec = t2.translation - t1.translation;
        if diff_vec.length_squared() < 0.5 {
            let drag_vec = diff_vec.normalize();
            let drag_mag = bm1.velocity.dot(drag_vec);
            bm1.velocity -= drag_mag * drag_vec * time.delta_secs();
            if drag_mag == 0. {
                bm1.velocity -= drag_vec * time.delta_secs();
            }
        }
    }
}

fn aim_target(
    target_query: Query<&Transform, With<BotTargetMarker>>,
    mut bot_query: Query<
        (&mut Transform, &mut BotMotion, &BotState),
        (With<BotMarker>, Without<BotTargetMarker>),
    >,
    time: Res<Time>,
) {
    let t_trans = target_query.single();
    for (mut trans, mut motion, state) in bot_query.iter_mut() {
        match *state {
            BotState::Chasing => {
                let t_vec = t_trans.translation.clone() - trans.translation.clone();
                let rot_aixs = motion
                    .direction
                    .clone()
                    .normalize_or(Vec3::Y)
                    .cross(t_vec.normalize());
                let rotation = Quat::from_axis_angle(
                    rot_aixs.normalize_or(Vec3::Y),
                    motion.angular_steer.to_radians() * time.delta_secs(),
                );
                trans.rotate(rotation);
                motion.direction = trans.forward().as_vec3().normalize();
            }
            _ => (),
        }
    }
}

fn shoot_target(
    target_query: Query<&Transform, With<BotTargetMarker>>,
    bot_query: Query<
        (&Transform, &Children, &BotMotion),
        (With<BotMarker>, Without<BotTargetMarker>),
    >,
    mut bot_turret: Query<(Entity, &mut Turret), (With<TurretMarker>, With<BotTurret>)>,
    mut ev_turret_off: EventWriter<ShootTurretEventOff>,
    mut ev_turret_on: EventWriter<ShootTurretEventOn>,
) {
    'outer: for (b_trans, children, b_motion) in bot_query.iter() {
        let mut dist: Vec3;
        let mut angle: f32;

        for trans in target_query.iter() {
            dist = trans.translation - b_trans.translation;
            angle = b_motion
                .direction
                .normalize_or_zero()
                .dot(dist.normalize_or_zero());

            if dist.length() <= SHOOT_VICINITY_DISTANCE
                && angle.acos() <= SHOOT_VICINITY_ANGLE.to_radians()
                && angle >= 0.
            {
                for child in children {
                    if let Ok((ent, mut b_turret)) = bot_turret.get_mut(child.clone()) {
                        if !b_turret.0.shooting {
                            ev_turret_on.send(ShootTurretEventOn(ent.clone()));
                        } else if b_turret.0.overheat {
                            b_turret.0.shooting = false;
                            ev_turret_off.send(ShootTurretEventOff(ent.clone()));
                            continue;
                        }
                        b_turret.0.shooting = true;
                    }
                }
                continue 'outer;
            }
            for child in children {
                if let Ok((ent, mut b_turret)) = bot_turret.get_mut(child.clone()) {
                    if b_turret.0.shooting {
                        error!("sent audio down sound {}", ent.to_bits());
                        ev_turret_off.send(ShootTurretEventOff(ent.clone()));
                        b_turret.0.shooting = false;
                    }
                }
            }
        }
    }
}

pub fn setup(
    mut commands: Commands,
    scene_asset: Res<SceneAssets>,
    audio_assets: Res<AudioAssets>,
) {
    let bot_spaceship = scene_asset.bot_spaceship.clone();

    commands
        .spawn((
            SceneRoot(bot_spaceship.clone()),
            BotMotion { ..default() },
            BotState::Chasing,
            BotMarker,
            REAR_VIEW_LAYERS,
            GameObjectMarker,
            HomingMissileTarget,
            AudioPlayer(audio_assets.throttle_up.clone()),
            PlaybackSettings::LOOP.with_spatial(true),
            Transform::from_xyz(0., 20., 20.).with_scale(Vec3::new(0.5, 0.5, 0.5)), // .looking_at(Vec3::Y, Vec3::Z), // .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Transform::from_xyz(0., 0., 0.),
                Turret(TurretBundle {
                    shooting: false,
                    speed: 20.,
                    bullet_size: 0.0002,
                    shooter: Some(parent.parent_entity()),
                    ..default()
                }),
                AudioPlayer(audio_assets.laser_turret.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    paused: true,
                    spatial: true,
                    ..default()
                },
                BotTurret,
                TurretMarker,
            ));
        });
    commands
        .spawn((
            SceneRoot(bot_spaceship.clone()),
            BotMotion {
                acceleration: 6.,
                angular_steer: 80.,
                ..default()
            },
            AudioPlayer(audio_assets.throttle_up.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                paused: false,
                spatial: true,
                ..default()
            },
            REAR_VIEW_LAYERS,
            GameObjectMarker,
            HomingMissileTarget,
            BotState::Chasing,
            BotMarker,
            Transform::from_xyz(20., 30., 40.).with_scale(Vec3::new(0.5, 0.5, 0.5)), // .looking_at(Vec3::Y, Vec3::Z), // .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Transform::from_xyz(0., 0., 0.),
                Turret(TurretBundle {
                    shooting: false,
                    speed: 20.,
                    bullet_size: 0.0002,
                    shooter: Some(parent.parent_entity()),
                    ..default()
                }),
                AudioPlayer(audio_assets.laser_turret.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    paused: true,
                    spatial: true,
                    ..default()
                },
                BotTurret,
                TurretMarker,
            ));
        });
    commands
        .spawn((
            SceneRoot(scene_asset.bot_spaceship3.clone()),
            BotMotion {
                acceleration: 4.,
                angular_steer: 40.,
                ..default()
            },
            AudioPlayer(audio_assets.throttle_up.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                paused: false,
                spatial: true,
                ..default()
            },
            REAR_VIEW_LAYERS,
            GameObjectMarker,
            BotState::Chasing,
            HomingMissileTarget,
            BotMarker,
            Transform::from_xyz(0., 0., 0.).with_scale(Vec3::new(0.5, 0.5, 0.5)), // .looking_at(Vec3::Y, Vec3::Z), // .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Transform::from_xyz(0., 0., 0.),
                Turret(TurretBundle {
                    shooting: false,
                    speed: 20.,
                    bullet_size: 0.0002,
                    shooter: Some(parent.parent_entity()),
                    ..default()
                }),
                AudioPlayer(audio_assets.laser_turret.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    paused: true,
                    spatial: true,
                    ..default()
                },
                BotTurret,
                TurretMarker,
            ));
        });
    commands
        .spawn((
            SceneRoot(scene_asset.bot_spaceship2.clone()),
            BotMotion {
                acceleration: 10.,
                angular_steer: 90.,
                ..default()
            },
            AudioPlayer(audio_assets.throttle_up.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                paused: false,
                spatial: true,
                ..default()
            },
            HomingMissileTarget,
            BotState::Chasing,
            BotMarker,
            REAR_VIEW_LAYERS,
            GameObjectMarker,
            Transform::from_xyz(30., 70., 0.).with_scale(Vec3::new(0.5, 0.5, 0.5)), // .looking_at(Vec3::Y, Vec3::Z), // .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Transform::from_xyz(0., 0., 0.),
                Turret(TurretBundle {
                    shooting: false,
                    speed: 20.,
                    bullet_size: 0.0002,
                    shooter: Some(parent.parent_entity()),
                    ..default()
                }),
                AudioPlayer(audio_assets.laser_turret.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    paused: true,
                    spatial: true,
                    ..default()
                },
                BotTurret,
                TurretMarker,
            ));
        });
}
