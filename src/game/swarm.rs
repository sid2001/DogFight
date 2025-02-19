use crate::asset_loader::SceneAssets;
use bevy::{math::VectorSpace, prelude::*, state::commands};
use core::slice;
use rand::Rng;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
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

#[derive(Clone)]
pub struct SwarmData {
    leader: Option<Entity>,
    followers: u32,
    followers_limit: u32,
    state: SwarmBotState,
    last_state: SwarmBotState,
    swarm_id: u32,
}

impl Default for SwarmData {
    fn default() -> Self {
        Self {
            leader: None,
            followers: 0,
            followers_limit: 5,
            state: SwarmBotState::Solo,
            last_state: SwarmBotState::Solo,
            swarm_id: 0,
        }
    }
}

#[derive(Clone)]
pub struct SwarmIdPool {
    front: i32,
    back: i32,
    limit: u32,
    id_pool: Vec<u32>,
}

impl Default for SwarmIdPool {
    fn default() -> Self {
        let limit: u32 = 20;
        let mut id_pool = Vec::new();
        for i in 0..limit {
            id_pool.push(i + 1);
        }
        Self {
            front: 0,
            back: limit as i32 - 1,
            limit,
            id_pool,
        }
    }
}

impl SwarmIdPool {
    fn get_id(&mut self) -> u32 {
        if self.front != -1 {
            let x = self.id_pool[self.front as usize];
            if self.front == self.back {
                self.front = -1;
                self.back = -1;
            } else {
                self.front = (self.front + 1) % self.limit as i32;
            }
            x
        } else {
            0
        }
    }

    fn free_id(&mut self, id: u32) {
        self.back = (self.back + 1) % self.limit as i32;
        self.id_pool[self.back as usize] = id;
        if self.front == -1 {
            self.front = self.back;
        }
    }
}

#[derive(Resource, Clone)]
pub struct SwarmTracker(HashMap<Entity, SwarmData>, SwarmIdPool);

#[derive(Clone, PartialEq)]
pub enum SwarmBotState {
    Swarming,
    InSwarm,
    Solo,
    Avoid,
}
#[derive(Component, Clone)]
#[require(SceneRoot)]
pub struct SwarmBot {
    dir: Dir3,
    target_dir: Dir3,
    avoid_dir_vector: Dir3,
    velocity: Vec3,
    thrust: f32,
    thrust_limit: f32,
    angular_velocity: f32,
    drag: Vec3,
    health: f32,
    in_swarm: bool,
    target_distance: f32,
    swarm_data: SwarmData,
    state: SwarmBotState,
    swarm_up_distance: f32,
    repel_thrust: f32,
    swarm_spacing_min: f32,
    swarm_spacing_max: f32,
}

impl Default for SwarmBot {
    fn default() -> Self {
        Self {
            health: 10.,
            dir: Dir3::Y,
            target_dir: Dir3::Y,
            avoid_dir_vector: Dir3::Y,
            thrust: 0.,
            thrust_limit: 10.,
            angular_velocity: 200.,
            drag: Vec3::ZERO,
            velocity: Vec3::ZERO,
            in_swarm: false,
            target_distance: 9999.,
            swarm_data: SwarmData::default(),
            state: SwarmBotState::Solo,
            swarm_up_distance: 4.,
            repel_thrust: 0.5,
            swarm_spacing_min: 0.25,
            swarm_spacing_max: 0.5,
        }
    }
}

impl Default for SwarmPoint {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            burst_count: 5,
            cooldown: Duration::new(2, 0),
            last_burst: Duration::new(0, 0),
            limit: 20,
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
        app.insert_resource(SwarmTracker(HashMap::new(), SwarmIdPool::default()))
            .add_systems(Startup, setup)
            .add_systems(Update, release_bots)
            .add_systems(
                Update,
                (
                    // detect_target,
                    swarm_up, avoidance, steer, accelerate, move_bots,
                )
                    .chain(),
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
    mut swarm_tracker: ResMut<SwarmTracker>,
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
                rng.random_range(-1.0..=1.0),
                rng.random_range(-1.0..=1.0),
                rng.random_range(-1.0..=1.0),
            );
            let dir = Vec3::new(x, y, z).normalize_or(Vec3::Y);
            let rad = swarm_point.radius;

            let bot = SwarmBot {
                dir: Dir3::new(dir).unwrap_or(Dir3::Y),
                velocity: dir * 0.0,
                drag: Vec3::ZERO,
                ..default()
            };
            // info!("{}", bot.dir.clone().length());
            let pos = bot.dir.clone() * rad;
            // info!("Pos: {pos:?}");
            let transform = Transform::from_translation(trans.translation.clone() + pos)
                .with_scale(Vec3::new(0.1, 0.1, 0.1))
                .looking_to(bot.dir.clone(), Dir3::Y);
            let scene = SceneRoot(scene_assets.bot_spaceship.clone());
            let entity = commands.spawn((bot, transform, scene, SwarmBotMarker)).id();
            swarm_tracker.0.insert(entity, SwarmData::default());
        }
    }
}

fn move_bots(
    mut query_bots: Query<(&mut Transform, &SwarmBot), With<SwarmBotMarker>>,
    time: Res<Time>,
) {
    // query_bots
    //     .par_iter_mut()
    //     .for_each(|(mut trans, swarm_bot)| {
    //         trans.translation += swarm_bot.velocity * time.delta_secs();
    //     });

    for (mut trans, swarm_bot) in query_bots.iter_mut() {
        trans.translation += swarm_bot.velocity * time.delta_secs();
    }
}

fn steer(
    mut query_bots: Query<(&mut Transform, &mut SwarmBot), With<SwarmBotMarker>>,
    time: Res<Time>,
) {
    // query_bots.par_iter_mut().for_each(|(mut trans, mut bot)| {
    //     // if !bot.in_swarm {
    //     let rot_axis = bot.dir.as_vec3().cross(bot.target_dir.as_vec3());
    //     let rotation = Quat::from_axis_angle(
    //         rot_axis.normalize_or_zero(),
    //         bot.angular_velocity.to_radians() * time.delta_secs(),
    //     );
    //     trans.rotate(rotation);
    //     bot.dir = Dir3::new(trans.forward().as_vec3().normalize_or_zero()).unwrap_or(Dir3::Y);
    //     // } else {
    //     // todo!();
    //     // }
    // });

    for (mut trans, mut bot) in query_bots.iter_mut() {
        //     if !bot.in_swarm {
        let rot_axis = bot.dir.as_vec3().cross(bot.target_dir.as_vec3());
        let rotation = Quat::from_axis_angle(
            rot_axis.normalize_or_zero(),
            bot.angular_velocity.to_radians() * time.delta_secs(),
        );
        trans.rotate(rotation);
        bot.dir = trans.forward();

        //     } else {
        //         todo!();
        //     }
    }
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
    mut query_bots: Query<(Entity, &Transform, &mut SwarmBot), With<SwarmBotMarker>>,
    query_target: Query<&Transform, (With<SwarmTarget>, Without<SwarmBotMarker>)>,
    swarm_tracker: Res<SwarmTracker>,
) {
    query_bots
        .par_iter_mut()
        .for_each(|(entity, trans_bot, mut bot)| {
            let mut target: Vec3 = Vec3::ZERO;
            let mut dist: f32 = 9999.;
            let sd = swarm_tracker.0.get(&entity).unwrap();
            if (sd.state == SwarmBotState::InSwarm && sd.leader.unwrap() == entity)
                || sd.state == SwarmBotState::Solo
            {
                query_target.iter().for_each(|trans_target| {
                    let target_dist = (trans_target.translation - trans_bot.translation).length();
                    if dist > target_dist {
                        target = trans_target.translation;
                        dist = target_dist;
                        // info!("target detected");
                    }
                });

                let dir = (target - trans_bot.translation).normalize_or_zero();
                bot.target_dir = Dir3::new(dir).unwrap_or(Dir3::Y);
                bot.target_distance = dist;
            }
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

fn avoidance(
    mut query_bots: Query<(Entity, &Transform, &mut SwarmBot), With<SwarmBotMarker>>,
    swarm_tracker: Res<SwarmTracker>,
) {
    let bots: Vec<_> = query_bots
        .iter()
        .map(|(e, t, _)| (e.clone(), t.clone()))
        .collect();
    // let mut_bots: Vec<_> = query_bots.iter_mut().collect();
    for (e1, t1, mut b1) in query_bots.iter_mut() {
        let mut dist: f32 = 999.;
        let mut obj = Vec3::ZERO;
        if let Some(swarm_data) = swarm_tracker.0.get(&e1) {
            match swarm_data.state {
                SwarmBotState::Avoid => {}
                SwarmBotState::Solo => {
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
                _ => (),
            }
        } else {
            error!("Entity should be present in hashmap!!");
        }
    }
}

fn swarm_up(
    mut query_bots: Query<(Entity, &Transform, &mut SwarmBot), With<SwarmBotMarker>>,
    mut swarm_tracker: ResMut<SwarmTracker>,
    time: Res<Time>,
) {
    let bots: Vec<_> = query_bots
        .iter()
        .map(|(e, t, b)| (e.clone(), t.clone(), b.clone()))
        .collect();

    let bots_map: HashMap<Entity, _> = bots
        .clone()
        .iter()
        .map(|(e, t, b)| (e.clone(), (t.clone(), b.clone())))
        .collect();

    for (e1, t1, mut b1) in query_bots.iter_mut() {
        // let mut dist = 999.;
        // let mut target_bot = Vec3::ZERO;
        // let swarm_map = swarm_tracker.0;
        if let Some(swarm_data) = swarm_tracker.0.clone().get(&e1) {
            match swarm_data.state {
                SwarmBotState::Solo => {
                    for (e2, t2, b2) in bots.iter() {
                        if e1 == *e2 {
                            continue;
                        }

                        // verify if this is necessary
                        if swarm_tracker.0[&e2].leader == Some(e1) {
                            continue;
                        }
                        if (t1.translation - t2.translation).length() <= b1.swarm_up_distance {
                            //swarm up only when coming from behind
                            //right swarming up the first bot it see, not with the closest one
                            let costh = b1.velocity.normalize().dot(b2.velocity.normalize());
                            if costh < 0. {
                                continue;
                            } else {
                                //first swarm up with the first one later move to its leader if it has space like a balanced tree
                                // b1.swarm_data.leader = Some(e2.clone());
                                // let t_swarm_data: SwarmData;
                                let mut swarm_id: u32 = 0;
                                if let Some(sd) = swarm_tracker.0.get(&e2) {
                                    if sd.swarm_id != 0 {
                                        swarm_id = sd.swarm_id;
                                    } else {
                                        swarm_id = swarm_tracker.1.get_id();
                                        if swarm_id == 0 {
                                            continue;
                                        }
                                    }
                                } else {
                                    error!("entity should have been present!!");
                                }

                                swarm_tracker.0.entry(e2.clone()).and_modify(|d| {
                                    d.followers += 1;
                                    d.state = SwarmBotState::InSwarm;
                                    d.swarm_id = swarm_id;
                                    d.leader = Some(e2.clone());
                                });

                                info!("{} swarming with {}", e1.to_bits(), e2.to_bits());
                                swarm_tracker.0.entry(e1.clone()).and_modify(|d| {
                                    d.state = SwarmBotState::Swarming;
                                    d.leader = Some(e2.clone());
                                    d.swarm_id = swarm_id;
                                });
                                b1.target_dir = Dir3::new(
                                    (t2.translation - t1.translation).normalize_or(Vec3::Y),
                                )
                                .unwrap();
                                break;
                                // b1.state = SwarmBotState::Swarming;
                                // swarm_data.state = SwarmBotState::Swarming;
                                // swarm_data.leader = Some(e2.clone());
                                //think something to update the followers list
                                // b2.swarm_data.followers += 1;
                            }
                        }
                    }
                }
                SwarmBotState::Swarming => {
                    let leader = swarm_tracker.0.get(&e1).unwrap().leader.unwrap();
                    let leader_transform = bots
                        .iter()
                        .find(|(e, _, _)| *e == leader)
                        .map(|(e2, t, b2)| (e2.clone(), t.clone(), b2.clone()));
                    if let Some((e2, l_trans, b2)) = leader_transform {
                        // verify if this is required
                        if swarm_tracker.0[&e2].leader == Some(e1) {
                            continue;
                        }
                        // Perform operations with leader_transform
                        if (l_trans.translation - t1.translation).length() <= b1.swarm_spacing_max {
                            swarm_tracker.0.entry(e1.clone()).and_modify(|d| {
                                d.state = SwarmBotState::InSwarm;
                            });
                            // if swarm_tracker.0[&e2].state == SwarmBotState::Solo {
                            //     swarm_tracker.0.entry(e2.clone()).and_modify(|d| {
                            //         d.state = SwarmBotState::InSwarm;
                            //         d.leader = Some(e2.clone());
                            //     });
                            // }
                            // swarm_tracker.0.entry(e2.clone()).and_modify(|d| {
                            //     // d.state = SwarmBotState::InSwarm;
                            //     d.leader = Some(e2.clone());
                            // });
                            b1.target_dir = l_trans.forward();
                            info!("{} in swarm with", e1.to_bits());
                            b1.thrust = b2.thrust;
                        } else {
                            b1.target_dir = Dir3::new(
                                (l_trans.translation - t1.translation).normalize_or(Vec3::ZERO),
                            )
                            .unwrap();
                            info!("{} swarming with", e1.to_bits());
                            b1.thrust = b2.thrust + 0.1;
                        }
                    } else {
                        swarm_tracker.0.entry(e1.clone()).and_modify(|d| {
                            d.state = SwarmBotState::Solo;
                            d.swarm_id = 0;
                            d.leader = None;
                        });
                    }
                }
                SwarmBotState::InSwarm => {
                    // error prone code block
                    if let Some((e2, (t2, b2))) =
                        bots_map.get_key_value(&swarm_tracker.0.get(&e1).unwrap().leader.unwrap())
                    {
                        if *e2 == e1 {
                            continue;
                        }
                        if (t2.translation - t1.translation).length() > b1.swarm_spacing_max {
                            swarm_tracker.0.get_mut(&e1).unwrap().state = SwarmBotState::Swarming;
                            continue;
                        }
                        b1.target_dir = b2.dir;
                    }
                    // change the above block in future

                    for (e2, t2, _) in bots.iter() {
                        if e1 == *e2 {
                            continue;
                        }
                        let separation_vector = t2.translation - t1.translation;
                        if separation_vector.length() < b1.swarm_spacing_min {
                            b1.velocity = {
                                b1.velocity
                                    + (-separation_vector.normalize_or_zero() * b1.repel_thrust)
                                        * time.delta_secs()
                            };
                        }
                    }
                }
                _ => (),
            }
        } else {
            error!("Entity should be present!!");
        }
    }
}

// fn swarming(mut query_bots: Query<(Entity, &Transform), With<SwarmBotMarker>>) {
//     for
// }

//todo
// get a way to prevent the leader from following it's follower
// while in swarm manitain distance and maintain alignment with neigbours
// try to serialize swarm, communicate swarm members of their global leaders
// make the thrust control more stable and dynamic
// give name to swarms so that avoidance mechanics still works if outer swarm bot tries to cut

// swarm bot is repelling all the bots in vicinity make it specific to only swarm bots
// add another method to coerce bots in swarm repel thrust sometimes is too much

// add alien swarm interaction and swarm merge
// fix swarming bug since the max swarming reamins low bots are
// swarming constantly, change repel thrust logic
