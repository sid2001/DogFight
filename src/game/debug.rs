use super::bots::{Bot, BotMarker, BotMotion, BotState, BotTurret};
use super::camera::REAR_VIEW_LAYERS;
use super::turret::*;
use super::{spaceship::*, GameObjectMarker};
use crate::asset_loader::*;
use crate::sets::*;
use crate::states::*;
use bevy::audio::{PlaybackMode::*, Volume};
use bevy::math::VectorSpace;
use bevy::prelude::*;
use std::f32::INFINITY;
use std::time::Duration;

#[derive(Component, Clone, Copy)]
pub struct ObstacleMarker;

#[derive(Component, Clone, Copy)]
pub struct ObstacleInfo {
    pub radius: f32,
}

#[derive(Resource)]
pub struct MyTimer(Timer);

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MyTimer(Timer::new(
            Duration::from_secs_f32(1.),
            TimerMode::Repeating,
        )))
        // .add_systems(Startup, setup)
        .add_systems(
            Update,
            (detect_obstacle, avoid_obstacle)
                .chain()
                .in_set(UpdateSet::InGame), // .run_if(in_state(GameState::Game)),
        );
        // .add_systems(PostStartup, mark_spaceship);
    }
}

fn mark_spaceship(
    mut commands: Commands,
    query: Query<(&Transform, &SceneRoot), With<SpaceShip>>,
    scene_asset: Res<SceneAssets>,
) {
    for (trans, scene) in query.iter() {
        // info!("spawned marker {}",scene.);
        commands.spawn((
            SceneRoot(scene_asset.map_marker.clone()),
            Transform::from_translation(trans.translation.clone())
                .with_scale(Vec3::new(0.05, 0.05, 0.05))
                .with_rotation(trans.rotation.clone()),
            GameObjectMarker,
        ));
    }
}

// fn steer(
//     mut bot_query: Query<
//         (&mut Transform, &mut BotState, &mut BotMotion),
//         (With<BotMarker>, Without<PlanetMarker>),
//     >,
//     planet_query: Query<&Transform, With<PlanetMarker>>,
//     time: Res<Time>,
// ) {
// }

fn avoid_obstacle(
    mut bot_query: Query<
        (&mut Transform, &mut BotState, &mut BotMotion),
        (With<BotMarker>, Without<SpaceShip>),
    >,
    planet_query: Query<&Transform, With<SpaceShip>>,
    time: Res<Time>,
    mut timer: ResMut<MyTimer>,
) {
    for (mut trans, state, mut motion) in bot_query.iter_mut() {
        let p_trans = planet_query.single();
        let t = time.delta_secs();
        match &*state {
            BotState::Chasing => {
                motion.last_dir = None;
                let t_vec = p_trans.translation.clone() - trans.translation.clone();
                let rot_axis: Vec3;
                // info!(
                //     "nearest obs {} tv {}",
                //     motion.nearest_obstacle.0,
                //     t_vec.clone().length()
                // );
                if motion.nearest_obstacle.0 >= t_vec.clone().length() {
                    rot_axis = motion
                        .direction
                        .clone()
                        .normalize_or(Vec3::Y)
                        .cross(t_vec.normalize_or_zero());
                } else {
                    match &mut motion.last_dir {
                        Some(_) => (),
                        None => {
                            motion.last_dir = Some(Dir3::new(motion.direction.clone()).unwrap());
                        }
                    }
                    rot_axis = motion
                        .direction
                        .clone()
                        .normalize_or(Vec3::Y)
                        .cross(motion.nearest_obstacle.1.normalize_or_zero());
                }
                let rotation = Quat::from_axis_angle(
                    rot_axis.normalize_or(Vec3::Y),
                    motion.angular_steer.to_radians() * t,
                );
                trans.rotate(rotation);
                motion.direction = trans.forward().as_vec3().normalize();
                // let drag = motion.drag.clone();
                // let velocity = motion.direction.clone().normalize_or_zero() * motion.acceleration * t
                //     + motion.velocity.clone()
                //     + drag * t;
                // motion.velocity = velocity.clone();
                // trans.translation += motion.velocity.clone() * t;
                // motion.drag = -velocity.clone() * 2.;
                if timer.0.tick(time.delta()).just_finished() {
                    // info!("Velocityy bot {}", motion.velocity.length());
                }
            }
            BotState::Ideal => {
                let rot_axis = motion
                    .direction
                    .clone()
                    .normalize_or_zero()
                    .cross(trans.right().as_vec3())
                    .normalize_or_zero();
                let rotation = Quat::from_axis_angle(
                    rot_axis.normalize_or(Vec3::Y),
                    motion.angular_steer.to_radians() * t,
                );
                trans.rotate(rotation);
                motion.direction = trans.forward().as_vec3().normalize_or_zero();
                trans.translation +=
                    motion.velocity.clone().length() * motion.direction.clone() * t;
            }
            BotState::Dodge(dir) => {
                info!("dodging ");
                let acc = motion.acceleration * dir.as_vec3();
                motion.velocity += acc.clone() * t;
                *trans = trans.looking_to(
                    Dir3::new(motion.velocity.clone().normalize_or_zero()).unwrap(),
                    motion.velocity.clone().cross(dir.clone().as_vec3()),
                );
                trans.translation += motion.velocity.clone() * t;
            }
            _ => (),
        }
    }
}

fn detect_obstacle(
    query: Query<(&Transform, &ObstacleInfo), With<ObstacleMarker>>,
    mut b_query: Query<
        (Entity, &Transform, &mut BotMotion, &mut BotState),
        (With<BotMarker>, Without<ObstacleMarker>),
    >,
) {
    for (entity, b_trans, mut motion, mut state) in b_query.iter_mut() {
        // store obstacle which is nearest on the collision path
        let mut obstacles: (f32, Dir3) = (f32::INFINITY, Dir3::Z); // placeholder value
        for (p_trans, obstacle) in query.iter() {
            let rad = obstacle.radius;
            let acc = motion.acceleration.clone();
            let vel = motion.velocity.clone();
            let p_pos = p_trans.translation.clone();
            let b_pos = b_trans.translation.clone();
            let b_dir = motion.direction.clone().normalize_or_zero();

            // calculating perpendicular distance from centre of the planet
            // |(r1 - r2) X n^|
            let per_dist = ((p_pos.clone() - b_pos.clone()).cross(b_dir.clone())).length();
            // info!("perd {} rad {}", per_dist, rad);
            if per_dist >= rad {
                continue;
            } else {
                let r1 = (b_pos - p_pos).normalize_or_zero();
                let costh = r1.clone().dot(vel.clone().normalize_or_zero());
                // when bot is moving away from the planet
                // info!("costh {}", costh.clone());
                if costh >= 0. {
                    continue;
                } else {
                    let dist = (p_pos.clone() - b_pos.clone()).dot(vel.clone().normalize_or_zero())
                        - (rad.powf(2.) - per_dist.powf(2.)).sqrt();
                    if obstacles.0 > dist {
                        let r1 =
                            (p_pos - b_pos).dot(vel.clone().normalize_or_zero()) + p_pos.clone();
                        obstacles = (dist, Dir3::new((r1 - p_pos).normalize_or_zero()).unwrap());
                    }
                    // let t = (2. * dist / motion.acceleration).sqrt();
                    // let x = motion.velocity.clone().length() * t;
                    // error!("x {} dist {}", x, dist);
                    // if x < dist {
                    //     continue;
                    // }
                    // *state = BotState::Dodge(Dir3::new((r1 - p_pos).normalize_or_zero()).unwrap());
                }
            }
        }
        motion.nearest_obstacle = obstacles;
    }
}

// fn spawn_bot(
//     mut commands: Commands,
//     scene_asset: Res<SceneAssets>,
//     audio_assets: Res<AudioAssets>,
// ) {
//     let bot_spaceship = scene_asset.bot_spaceship.clone();

//     commands
//         .spawn((
//             SceneRoot(bot_spaceship.clone()),
//             BotMotion {
//                 acceleration: 5.,
//                 velocity: Vec3::ONE,
//                 ..default()
//             },
//             Bot {
//                 health: 100.,
//                 level: 1,
//             },
//             GameObjectMarker,
//             BotState::Chasing,
//             BotMarker,
//             AudioPlayer(audio_assets.throttle_up.clone()),
//             PlaybackSettings {
//                 mode: Loop,
//                 paused: true,
//                 spatial: true,
//                 ..default()
//             },
//             // PlaybackSettings::LOOP.with_spatial(true).paused(),
//             Transform::from_xyz(0., 20., 0.).with_scale(Vec3::new(0.5, 0.5, 0.5)), // .looking_at(Vec3::Y, Vec3::Z), // .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
//         ))
//         .with_children(|parent| {
//             parent.spawn((
//                 Transform::from_xyz(0., 10., 0.),
//                 Turret(TurretBundle {
//                     shooting: false,
//                     speed: 20.,
//                     bullet_size: 0.0002,
//                     ..default()
//                 }),
//                 AudioPlayer(audio_assets.laser_turret.clone()),
//                 PlaybackSettings::LOOP.with_spatial(true).paused(),
//                 BotTurret,
//                 TurretMarker,
//             ));
//         });
// }

// fn setup(mut commands: Commands, scene_asset: Res<SceneAssets>) {
//     commands.spawn((
//         SceneRoot(scene_asset.planet1.clone()),
//         Transform::from_xyz(0., 0., 0.),
//         PlanetMarker,
//         PlanetRadius(2.),
//         REAR_VIEW_LAYERS,
//         GameObjectMarker,
//     ));
//     commands.spawn((
//         SceneRoot(scene_asset.planet1.clone()),
//         Transform::from_xyz(0., 0., 4.),
//         PlanetMarker,
//         PlanetRadius(2.),
//         GameObjectMarker,
//     ));
//     commands.spawn((
//         SceneRoot(scene_asset.planet1.clone()),
//         Transform::from_xyz(3., 0., 2.),
//         PlanetMarker,
//         PlanetRadius(2.),
//         GameObjectMarker,
//     ));
// }

// fn print_position(
//     query: Query<&Transform, With<SpaceShip>>,
//     mut timer: ResMut<MyTimer>,
//     time: Res<Time>,
//     mut bot_query: Query<&mut BotState, (With<BotMarker>, Without<PlanetMarker>)>,
// ) {
//     let trans = query.single();
//     let mut state = bot_query.single_mut();
//     if timer.0.tick(time.delta()).just_finished() {
//         // info!("{}", trans.translation);
//         // *state = BotState::Chasing;
//     }
// }

// fn search_points(mut commands: Commands, scene_asset: Res<SceneAssets>) {
//     let map_marker = scene_asset.bot_spaceship.clone();

//     commands.spawn((
//         SceneRoot(bot_spaceship),
//         BotMotion { ..default() },
//         Bot {
//             health: 100.,
//             level: 1,
//         },
//         BotState::Ideal,
//         Transform::from_xyz(0., 0., 0.),
//     ));
// }
