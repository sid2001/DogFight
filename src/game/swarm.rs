use crate::asset_loader::SceneAssets;
use bevy::{math::VectorSpace, prelude::*, state::commands};
use rand::Rng;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::time::Duration;

use super::spaceship::Health;

#[derive(Component)]
pub struct SwarmTarget;

#[derive(Component)]
pub struct SwarmPointMarker;

#[derive(Component)]
pub struct SwarmBotMarker;

#[derive(Component)]
pub struct SwarmPoint {
    origin: Vec3,
    burst_count: u32,
    cooldown: Duration,
    last_burst: Duration,
    limit: u32,
    live_bots: u32,
    radius: f32,
}

#[derive(Component)]
#[require(SceneRoot)]
pub struct SwarmBot {
    dir: Dir3,
    target_dir: Dir3,
    velocity: Vec3,
    thrust: f32,
    thrust_limit: f32,
    angular_velocity: f32,
    drag: Vec3,
    health: f32,
    in_swarm: bool,
    target_distance: f32,
}

impl Default for SwarmBot {
    fn default() -> Self {
        Self {
            health: 10.,
            dir: Dir3::Y,
            target_dir: Dir3::Y,
            thrust: 0.,
            thrust_limit: 10.,
            angular_velocity: 80.,
            drag: Vec3::ZERO,
            velocity: Vec3::ZERO,
            in_swarm: false,
            target_distance: 9999.,
        }
    }
}

impl Default for SwarmPoint {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            burst_count: 20,
            cooldown: Duration::new(10, 0),
            last_burst: Duration::new(0, 0),
            limit: 40,
            live_bots: 0,
            radius: 2.,
        }
    }
}

impl SwarmPoint {
    pub fn xyz(&self) -> Vec3 {
        self.origin.clone()
    }
}

pub struct SwarmPlugin;
impl Plugin for SwarmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, release_bots)
            .add_systems(
                Update,
                (detect_target, avoidance, steer, accelerate, move_bots).chain(),
            );
    }
}

fn setup(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    let swarm = SwarmPoint::default();
    let origin = swarm.xyz();
    let transform =
        Transform::from_xyz(origin.x, origin.y, origin.z).with_scale(Vec3::new(0.5, 0.5, 0.5));
    let scene = SceneRoot(scene_assets.swarm_point.clone());
    commands.spawn((swarm, transform, scene, SwarmPointMarker));
    commands.spawn((
        SceneRoot(scene_assets.map_marker.clone()),
        Transform::from_xyz(-4., 4., -6.).with_scale(Vec3::new(0.05, 0.05, 0.05)),
        SwarmTarget,
    ));
    commands.spawn((
        SceneRoot(scene_assets.map_marker.clone()),
        Transform::from_xyz(4., 4., -6.).with_scale(Vec3::new(0.05, 0.05, 0.05)),
        SwarmTarget,
    ));
}

fn release_bots(
    mut commands: Commands,
    mut query_swarm_point: Query<(&Transform, &mut SwarmPoint), With<SwarmPointMarker>>,
    scene_assets: Res<SceneAssets>,
    time: Res<Time>,
) {
    let time_delta = time.delta();
    for (trans, mut swarm_point) in query_swarm_point.iter_mut() {
        swarm_point.last_burst += time_delta;
        if swarm_point.last_burst < swarm_point.cooldown {
            continue;
        }
        swarm_point.last_burst = Duration::ZERO;
        let mut count = swarm_point.limit - swarm_point.live_bots;
        count = if count > swarm_point.burst_count {
            swarm_point.burst_count
        } else {
            count
        };
        swarm_point.live_bots += count;
        for _ in 0..count {
            let mut rng = rand::rng();
            let (x, y, z) = (
                rng.random_range(0.0..=0.1),
                rng.random_range(0.0..=0.1),
                rng.random_range(0.0..=0.1),
            );
            let dir = Vec3::new(x, y, z).normalize_or(Vec3::Y);
            let rad = swarm_point.radius;

            let bot = SwarmBot {
                dir: Dir3::new(dir).unwrap_or(Dir3::Y),
                velocity: dir * 0.0,
                thrust: 2.,
                drag: Vec3::ZERO,
                ..default()
            };
            // info!("{}", bot.dir.clone().length());
            let pos = bot.dir.clone() * rad;
            // info!("Pos: {pos:?}");
            let transform = Transform::from_translation(trans.translation.clone() + pos)
                .with_scale(Vec3::new(0.2, 0.2, 0.2))
                .looking_to(bot.dir.clone(), Dir3::Y);
            let scene = SceneRoot(scene_assets.bot_spaceship.clone());
            commands.spawn((bot, transform, scene, SwarmBotMarker));
        }
    }
}

fn move_bots(
    mut query_bots: Query<(&mut Transform, &SwarmBot), With<SwarmBotMarker>>,
    time: Res<Time>,
) {
    query_bots
        .par_iter_mut()
        .for_each(|(mut trans, swarm_bot)| {
            trans.translation += swarm_bot.velocity * time.delta_secs();
        });

    // for (mut trans, swarm_bot) in query_bots.iter_mut() {
    //     trans.translation += swarm_bot.velocity * time.delta_secs();
    // }
}

fn steer(
    mut query_bots: Query<(&mut Transform, &mut SwarmBot), With<SwarmBotMarker>>,
    time: Res<Time>,
) {
    query_bots.par_iter_mut().for_each(|(mut trans, mut bot)| {
        if !bot.in_swarm {
            let rot_axis = bot.dir.as_vec3().cross(bot.target_dir.as_vec3());
            let rotation = Quat::from_axis_angle(
                rot_axis.normalize_or_zero(),
                bot.angular_velocity.to_radians() * time.delta_secs(),
            );
            trans.rotate(rotation);
            bot.dir = trans.forward();
        } else {
            todo!();
        }
    });

    // for (mut trans, mut bot) in query_bots.iter_mut() {
    //     if !bot.in_swarm {
    //         let rot_axis = bot.dir.as_vec3().cross(bot.target_dir.as_vec3());
    //         let roatation = Quat::from_axis_angle(
    //             rot_axis.normalize_or_zero(),
    //             bot.angular_velocity.to_radians() * time.delta_secs(),
    //         );
    //         trans.rotate(roatation);
    //         bot.dir = trans.forward();
    //     } else {
    //         todo!();
    //     }
    // }
}

fn accelerate(mut query_bots: Query<&mut SwarmBot, With<SwarmBotMarker>>, time: Res<Time>) {
    query_bots.par_iter_mut().for_each(|mut swarm_bot| {
        swarm_bot.velocity = {
            swarm_bot.velocity
                + (swarm_bot.drag + (swarm_bot.thrust * swarm_bot.dir.as_vec3()))
                    * time.delta_secs()
        };
        swarm_bot.drag = -0.5 * swarm_bot.velocity
    });

    // for mut swarm_bot in query_bots.iter_mut() {
    //     swarm_bot.velocity = {
    //         swarm_bot.velocity
    //             + (swarm_bot.drag.clone() + (swarm_bot.thrust * swarm_bot.dir.as_vec3()))
    //                 * time.delta_secs()
    //     };
    //     swarm_bot.drag = -0.3 * swarm_bot.velocity;
    // }
}

fn detect_target(
    mut query_bots: Query<(&Transform, &mut SwarmBot), With<SwarmBotMarker>>,
    query_target: Query<&Transform, (With<SwarmTarget>, Without<SwarmBotMarker>)>,
) {
    query_bots.par_iter_mut().for_each(|(trans_bot, mut bot)| {
        let mut target: Vec3 = Vec3::ZERO;
        let mut dist: f32 = 9999.;

        query_target.iter().for_each(|trans_target| {
            let target_dist = (trans_target.translation - trans_bot.translation).length();
            if dist > target_dist {
                target = trans_target.translation;
                dist = target_dist;
                info!("target detected");
            }
        });

        let dir = (target - trans_bot.translation).normalize_or_zero();
        bot.target_dir = Dir3::new(dir).unwrap_or(Dir3::Y);
        bot.target_distance = dist;
    });
    // for (trans_bot, mut bot) in query_bots.iter_mut() {
    //     let mut target: Vec3 = Vec3::ZERO;
    //     let mut dist: f32 = 9999.; // this can break game if some bot went too far
    //     for trans_target in query_target.iter() {
    //         let target_dist =
    //             (trans_target.translation.clone() - trans_bot.translation.clone()).length();
    //         if dist > target_dist {
    //             target = trans_target.translation;
    //             dist = target_dist;
    //             info!("target detected");
    //         }
    //     }
    //     let dir = (target - trans_bot.translation).normalize_or_zero();
    //     bot.target_dir = Dir3::new(dir).unwrap_or(Dir3::Y);
    //     bot.target_distance = dist;
    // }
}

fn avoidance(mut query_bots: Query<(Entity, &Transform, &mut SwarmBot), With<SwarmBotMarker>>) {
    let bots: Vec<_> = query_bots
        .iter()
        .map(|(e, t, _)| (e.clone(), t.clone()))
        .collect();
    // let mut_bots: Vec<_> = query_bots.iter_mut().collect();
    for (e1, t1, mut b1) in query_bots.iter_mut() {
        let mut dist: f32 = 999.;
        let mut obj = Vec3::ZERO;
        for (e2, t2) in bots.iter() {
            if e1 == *e2 {
                continue;
            }
            let diff = (t2.translation - t1.translation).length();
            if dist > diff {
                dist = diff;
                obj = t2.translation;
            }
        }
        if dist < 0.2 {
            let avoid_dir = (obj - t1.translation).normalize_or_zero();
            let mut dir = avoid_dir.cross(t1.forward().as_vec3());
            if dir == Vec3::ZERO {
                dir = avoid_dir.cross(t1.left().as_vec3());
            }
            b1.target_dir = Dir3::new(dir).unwrap_or(Dir3::Y);
        }
    }
}
