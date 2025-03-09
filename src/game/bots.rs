use super::camera::REAR_VIEW_LAYERS;
use super::missile::HomingMissileTarget;
use bevy::utils::info;
use bevy::{math::VectorSpace, prelude::*};

use super::{turret::*, GameObjectMarker};
use crate::asset_loader::*;
use crate::sets::*;

use crate::asset_loader::SceneAssets;

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
pub struct TargetMarker;

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
            angular_steer: 60.,
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

#[derive(Component)]
pub struct BotTurret;

#[derive(Component)]
pub struct Bot {
    pub health: f32,
    pub level: u32,
}

#[derive(Resource)]
pub struct BotCount(u32);

//*implement later */
// #[derive(Resource)]
// pub struct BotAssets {
//     pub spaceship: Handle<Scene>,
//     pub turret: Handle<Scene>,
// }

// #[derive(Bundle)]
// pub struct BotBundle {
// }

pub struct BotPlugin;
impl Plugin for BotPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BotCount(0))
            // .add_systems(Startup, setup)
            .add_systems(
                Update,
                (thrust_control, chase_target)
                    .chain()
                    .in_set(UpdateSet::InGame),
            )
            .add_systems(
                Update,
                (shoot_target, shoot_turret::<BotTurret>)
                    .chain()
                    .in_set(UpdateSet::InGame),
            );
    }
}

fn chase_target(
    target_query: Query<&Transform, With<TargetMarker>>,
    mut bot_query: Query<
        (&mut Transform, &mut BotState, &mut BotMotion),
        (With<BotMarker>, Without<TargetMarker>),
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
                if bm.acceleration < 6. {
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

fn aim_target(
    target_query: Query<&Transform, With<TargetMarker>>,
    mut bot_query: Query<
        (&mut Transform, &mut BotMotion, &BotState),
        (With<BotMarker>, Without<TargetMarker>),
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
    target_query: Query<&Transform, With<TargetMarker>>,
    bot_query: Query<(&Transform, &Children, &BotMotion), (With<BotMarker>, Without<TargetMarker>)>,
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
            Bot {
                health: 100.,
                level: 1,
            },
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
            Bot {
                health: 100.,
                level: 1,
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
            Bot {
                health: 100.,
                level: 3,
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
            Bot {
                health: 100.,
                level: 2,
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
