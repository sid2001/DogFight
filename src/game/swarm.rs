use super::explosion::{ExplosibleObjectMarker, *};
use super::missile::SwarmMissileTarget;
use super::{collider::*, GameObjectMarker};
use crate::asset_loader::{AudioAssets, SceneAssets};
use crate::sets::*;
use crate::states::*;
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

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
            followers_limit: 10,
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
    subs: Vec<u32>,
}

impl Default for SwarmIdPool {
    fn default() -> Self {
        let limit: u32 = 20;
        let mut id_pool = Vec::new();
        let mut subs = Vec::new();
        for i in 0..limit {
            id_pool.push(i + 1);
            subs.push(0);
        }
        Self {
            front: 0,
            back: limit as i32 - 1,
            limit,
            id_pool,
            subs,
        }
    }
}

impl SwarmIdPool {
    fn get_id(&mut self) -> u32 {
        if self.front != -1 {
            let x = self.id_pool[self.front as usize];
            self.unit_inc_subs(x as usize);
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
        // self.unit_dec_subs(id as usize);
        if self.front == -1 {
            self.front = self.back;
        }
    }
    fn unit_inc_subs(&mut self, x: usize) {
        self.subs[x - 1] += 1;
    }
    fn unit_dec_subs(&mut self, x: usize) {
        error!("subs {}", self.subs[x - 1]);
        self.subs[x - 1] -= 1;
        if self.subs[x - 1] == 0 {
            self.free_id(x as u32);
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

#[derive(Clone, PartialEq)]
pub enum TargetVicinity {
    Far,
    Around,
    Near,
}
#[derive(Component, Clone)]
#[require(SceneRoot)]
pub struct SwarmBot {
    swarm_point: Option<Entity>,
    dir: Dir3,
    target_dir: Dir3,
    target_vicinity: TargetVicinity,
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
    is_alive: bool,
}

impl Default for SwarmBot {
    fn default() -> Self {
        Self {
            swarm_point: None,
            health: 40.,
            dir: Dir3::Y,
            target_dir: Dir3::Y,
            target_vicinity: TargetVicinity::Around,
            avoid_dir_vector: Dir3::Y,
            thrust: 1.,
            thrust_limit: 10.,
            angular_velocity: 100.,
            drag: Vec3::ZERO,
            velocity: Vec3::ZERO,
            in_swarm: false,
            target_distance: 9999.,
            swarm_data: SwarmData::default(),
            state: SwarmBotState::Solo,
            swarm_up_distance: 2.,
            repel_thrust: 0.2,
            swarm_spacing_min: 0.1,
            swarm_spacing_max: 0.5,
            is_alive: true,
        }
    }
}

impl SwarmBot {
    fn estimate_vicintiy(dist: f32) -> TargetVicinity {
        // let dist = (p1 - p2).length();
        if dist <= 3. {
            TargetVicinity::Near
        } else if dist <= 6. {
            TargetVicinity::Around
        } else {
            TargetVicinity::Far
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
        app.add_systems(OnEnter(InGameStates::Setup), setup)
            .add_systems(Update, release_bots.in_set(UpdateSet::InGame))
            .add_systems(
                Update,
                (
                    detect_target,
                    thrust_control,
                    swarm_up,
                    coerce,
                    avoidance,
                    steer,
                    accelerate,
                    move_bots,
                    collision_response,
                    despawn_swarm_bots,
                )
                    .chain()
                    .in_set(UpdateSet::InGame),
            )
            .add_systems(OnExit(GameState::Game), clear_resources)
            .add_systems(OnEnter(InGameStates::Over), clear_resources);
    }
}

fn clear_resources(mut commands: Commands) {
    commands.remove_resource::<SwarmTracker>();
}

pub fn setup(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    commands.insert_resource(SwarmTracker(HashMap::new(), SwarmIdPool::default()));
    let swarm = SwarmPoint::default();
    let origin = swarm.xyz();
    let transform =
        Transform::from_xyz(origin.x, origin.y, origin.z).with_scale(Vec3::new(0.5, 0.5, 0.5));
    let scene = SceneRoot(scene_assets.swarm_point.clone());
    commands.spawn((swarm, transform, scene, SwarmPointMarker, GameObjectMarker));
    // commands.spawn((
    //     SceneRoot(scene_assets.map_marker.clone()),
    //     Transform::from_xyz(-4., 4., -6.).with_scale(Vec3::new(0.05, 0.05, 0.05)),
    //     SwarmTarget,
    //     GameObjectMarker,
    // ));
    // commands.spawn((
    //     SceneRoot(scene_assets.map_marker.clone()),
    //     Transform::from_xyz(4., 4., -6.).with_scale(Vec3::new(0.05, 0.05, 0.05)),
    //     SwarmTarget,
    //     GameObjectMarker,
    // ));
}

fn release_bots(
    mut commands: Commands,
    mut query_swarm_point: Query<(Entity, &Transform, &mut SwarmPoint), With<SwarmPointMarker>>,
    scene_assets: Res<SceneAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut swarm_tracker: ResMut<SwarmTracker>,
) {
    let mat_handle = materials.add(StandardMaterial {
        emissive: LinearRgba::rgb(13.99, 5.32, 10.0),
        ..default()
    });
    let time_delta = time.delta();
    for (sp_ent, trans, mut swarm_point) in query_swarm_point.iter_mut() {
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
                swarm_point: Some(sp_ent),
                ..default()
            };
            // info!("{}", bot.dir.clone().length());
            let pos = bot.dir.clone() * rad;
            // info!("Pos: {pos:?}");
            let transform = Transform::from_translation(trans.translation.clone() + pos)
                .with_scale(Vec3::new(0.1, 0.1, 0.1))
                .looking_to(bot.dir.clone(), Dir3::Y);
            let scene = SceneRoot(scene_assets.bot_spaceship.clone());
            let entity = commands
                .spawn((
                    bot,
                    transform,
                    scene,
                    MeshMaterial3d(mat_handle.clone()),
                    SwarmBotMarker,
                    ExplosibleObjectMarker,
                    ColliderMarker,
                    CollisionDamage {
                        damage: 10.,
                        from: None,
                    },
                    ColliderInfo {
                        collider_type: ColliderType::Sphere,
                        collider: Arc::new(RwLock::new(SphericalCollider {
                            radius: 0.05,
                            center: Vec3::ZERO,
                        })),
                        immune_to: None,
                    },
                    SwarmMissileTarget,
                    GameObjectMarker,
                ))
                .id();
            swarm_tracker.0.insert(entity, SwarmData::default());
        }
    }
}

fn move_bots(
    mut query_bots: Query<(&mut Transform, &SwarmBot), With<SwarmBotMarker>>,
    time: Res<Time>,
) {
    for (mut trans, swarm_bot) in query_bots.iter_mut() {
        trans.translation += swarm_bot.velocity * time.delta_secs();
    }
}

fn steer(
    mut query_bots: Query<(&mut Transform, &mut SwarmBot), With<SwarmBotMarker>>,
    time: Res<Time>,
) {
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
    for mut swarm_bot in query_bots.iter_mut() {
        swarm_bot.velocity = {
            swarm_bot.velocity
                + (swarm_bot.drag.clone() + (swarm_bot.thrust * swarm_bot.dir.as_vec3()))
                    * time.delta_secs()
        };
        swarm_bot.drag = -0.5 * swarm_bot.velocity;
    }
}

fn thrust_control(mut query_bots: Query<&mut SwarmBot, With<SwarmBotMarker>>, time: Res<Time>) {
    for mut sb in query_bots.iter_mut() {
        match sb.target_vicinity {
            TargetVicinity::Far => {
                if sb.thrust < 3. {
                    sb.thrust += 0.3 * time.delta_secs();
                }
            }
            TargetVicinity::Around => {
                if sb.thrust > 1. {
                    sb.thrust -= 0.6 * time.delta_secs();
                } else {
                    sb.thrust = 1.
                }
                // sb.thrust = 1.;
            }
            TargetVicinity::Near => {
                // if sb.thrust > 0. {
                //     sb.thrust -= 0.6 * time.delta_secs();
                // } else {
                //     sb.thrust = 0.;
                // }
            }
        }
    }
}

fn detect_target(
    mut query_bots: Query<(Entity, &Transform, &mut SwarmBot), With<SwarmBotMarker>>,
    query_target: Query<&Transform, (With<SwarmTarget>, Without<SwarmBotMarker>)>,
    swarm_tracker: Res<SwarmTracker>,
) {
    for (entity, trans_bot, mut bot) in query_bots.iter_mut() {
        let mut target: Vec3 = Vec3::ZERO;
        let mut dist: f32 = 9999.;
        let sd = swarm_tracker.0.get(&entity).unwrap();
        if sd.state == SwarmBotState::Solo
            || (sd.state == SwarmBotState::InSwarm && sd.leader.unwrap() == entity)
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
            bot.target_vicinity = SwarmBot::estimate_vicintiy(dist);
        }
    }
}

fn avoidance(
    mut query_bots: Query<(Entity, &Transform, &mut SwarmBot), With<SwarmBotMarker>>,
    time: Res<Time>,
) {
    let mut query = query_bots.iter_combinations_mut();
    // let mut_bots: Vec<_> = query_bots.iter_mut().collect();
    while let Some([(_, t1, mut b1), (_, t2, _)]) = query.fetch_next() {
        let diff = (t2.translation - t1.translation).length();
        if diff < b1.swarm_spacing_min && diff > 0. {
            let dir = (t2.translation - t1.translation).normalize();
            b1.velocity = b1.velocity - (b1.velocity.dot(dir) * dir);
        } else if diff < b1.swarm_spacing_max && diff > 0. {
            let dir = (t1.translation - t2.translation).normalize();
            b1.velocity = b1.velocity + (b1.repel_thrust * time.delta_secs() * dir.normalize());
        }
    }
}

fn collision_response(
    mut query: Query<
        (
            Entity,
            &Transform,
            &ColliderInfo,
            &mut SwarmBot,
            Option<&ExplosibleObjectMarker>,
        ),
        With<SwarmBotMarker>,
    >,
    audio_asset: Res<AudioAssets>,
    mut ev_reader: EventReader<CollisionEvents>,
    mut ev_explode: EventWriter<ExplosionEvent>,
) {
    // let health = query.single();
    for msg in ev_reader.read() {
        match msg {
            CollisionEvents::TakeDamage(e, d, _) => {
                if let Ok((ent, trans, c_info, mut s_bot, ex_object)) = query.get_mut(e.clone()) {
                    if d.from.is_some_and(|e| e != ent) || d.from.is_none() {
                        if !s_bot.is_alive {
                            continue;
                        }
                        s_bot.health -= d.damage;
                        if s_bot.health <= 0. {
                            s_bot.is_alive = false;
                            if ex_object.is_some() {
                                info!("explosion sent");
                                ev_explode.send(ExplosionEvent {
                                    transform: trans.clone(),
                                    explosion: Explosion {
                                        half_extent: 0.15,
                                        ..default()
                                    },
                                    sound: Some(audio_asset.small_explosion.clone()),
                                });
                            }
                            // s_bot.health = 20.
                            // commands.entity(ent).despawn_recursive();
                            // todo!();
                        }
                        // info!("Health {}", s_bot.health);
                    }
                }
            }
        }
    }
}

pub fn despawn_swarm_bots(
    query: Query<(Entity, &SwarmBot), With<SwarmBotMarker>>,
    mut query_swarm_point: Query<&mut SwarmPoint, With<SwarmPointMarker>>,
    mut commands: Commands,
    mut swarm_tracker: ResMut<SwarmTracker>,
) {
    // let swarm_map = &mut swarm_tracker.0;
    // let id_pool = &swarm_tracker.1;
    for (ent, s_bot) in query.iter() {
        // info!("despawn {}", s_bot.health);
        if !s_bot.is_alive {
            // error!("despawning");
            if s_bot.swarm_point.is_some() {
                if let Ok(mut sp) = query_swarm_point.get_mut(s_bot.swarm_point.unwrap()) {
                    sp.live_bots -= 1;
                }
            }
            commands.entity(ent).despawn_recursive();
            if let Some(sd) = swarm_tracker.clone().0.get(&ent) {
                // check if it's in a swarm or not
                // if sd.swarm_id
                if let Some(leader) = &sd.leader {
                    error!("sward id {}", sd.swarm_id);
                    swarm_tracker.1.unit_dec_subs(sd.swarm_id as usize);
                    if *leader != ent {
                        if let Some(l_sd) = swarm_tracker.0.get_mut(leader) {
                            l_sd.followers -= 1;
                        }
                    }
                }
            }
            // delete from swarm tracker
            swarm_tracker.0.remove(&ent);
        }
    }
}

fn coerce(
    mut query: Query<(Entity, &Transform, &mut SwarmBot), With<SwarmBotMarker>>,
    swarm_tracker: Res<SwarmTracker>,
    time: Res<Time>,
) {
    let mut bot_query = query.iter_combinations_mut();
    while let Some([(e1, t1, mut sb1), (e2, t2, sb2)]) = bot_query.fetch_next() {
        if let Some(leader) = swarm_tracker.0.get(&e1).unwrap().leader {
            if leader == e2 {
                let dir = t2.translation - t1.translation;
                let diff = dir.length();
                if diff > sb1.swarm_spacing_max {
                    sb1.velocity = sb1.velocity + (0.3 * dir.normalize() * time.delta_secs());
                }
            }
        }
    }
}

fn swarm_up(
    mut query_bots: Query<(Entity, &Transform, &mut SwarmBot), With<SwarmBotMarker>>,
    mut swarm_tracker: ResMut<SwarmTracker>,
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

                        let angle = b1.velocity.normalize().dot(b2.velocity.normalize()).acos();
                        if ((t1.translation - t2.translation).length() <= b1.swarm_up_distance)
                            && (angle <= std::f32::consts::PI / 3. && angle >= 0.)
                            && (swarm_tracker.0.get(&e2).unwrap().followers
                                < swarm_tracker.0.get(&e2).unwrap().followers_limit)
                        {
                            //swarm up only when coming from behind
                            //right now swarming up the first bot it sees, not with the closest one

                            //first swarm up with the first one later move to its leader if it has space like a balanced tree
                            // b1.swarm_data.leader = Some(e2.clone());
                            // let t_swarm_data: SwarmData;

                            //checking if the swarm already exists and assigning swarm id if not
                            let mut swarm_id: u32 = 0;
                            if let Some(sd) = swarm_tracker.0.get(&e2) {
                                if sd.swarm_id != 0 {
                                    swarm_id = sd.swarm_id;
                                } else {
                                    // what if this fails later but the get id increased the sub count
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
                                if d.state == SwarmBotState::Solo {
                                    // d.state = SwarmBotState::InSwarm;
                                    d.swarm_id = swarm_id;
                                    d.leader = Some(e2.clone());
                                }
                                d.state = SwarmBotState::InSwarm;
                            });

                            // info!("{} swarming with {}", e1.to_bits(), e2.to_bits());
                            swarm_tracker.0.entry(e1.clone()).and_modify(|d| {
                                d.state = SwarmBotState::Swarming;
                                d.leader = Some(e2.clone());
                                d.swarm_id = swarm_id;
                            });
                            // update the subs count
                            swarm_tracker.1.unit_inc_subs(swarm_id as usize);
                            // b1 aims for the leader while swarming
                            b1.target_dir =
                                Dir3::new((t2.translation - t1.translation).normalize_or(Vec3::Y))
                                    .unwrap();
                            break;
                        }
                    }
                }
                SwarmBotState::Swarming => {
                    let leader = swarm_tracker.0.get(&e1).unwrap().leader.unwrap();
                    let leader_transform = bots
                        .iter()
                        .find(|(e, _, _)| *e == leader)
                        .map(|(e2, t, b2)| (e2.clone(), t.clone(), b2.clone()));
                    if let Some((_, l_trans, b2)) = leader_transform {
                        // verify if this is required
                        if leader == e1 {
                            continue;
                        }
                        // Perform operations with leader_transform
                        if (l_trans.translation - t1.translation).length() <= b1.swarm_spacing_max {
                            swarm_tracker.0.entry(e1.clone()).and_modify(|d| {
                                d.state = SwarmBotState::InSwarm;
                            });

                            b1.target_dir = l_trans.forward();
                            // info!("{} in swarm with", e1.to_bits());
                            info!("b1 thrust {} {} b2 thrust", b1.thrust, b2.thrust);

                            // b1.thrust = b2.thrust;
                        } else {
                            let diff = l_trans.translation - t1.translation;
                            b1.target_dir = Dir3::new(diff.normalize_or(Vec3::Y)).unwrap();
                            // info!("{} swarming with {}", e1.to_bits(), e2.to_bits());

                            // b1.thrust = b2.thrust + 0.2;
                            // this should work since the swarm up distance is less than 10
                            // the idea is to accelerate faster when farther
                        }
                    } else {
                        swarm_tracker.0.entry(e1.clone()).and_modify(|d| {
                            d.state = SwarmBotState::Solo;
                            d.swarm_id = 0;
                            d.leader = None;
                        });
                        // error!("leader not present");
                    }
                }
                SwarmBotState::InSwarm => {
                    // error prone code block
                    // checking leader position wrt follower
                    if let Some((e2, (t2, b2))) =
                        bots_map.get_key_value(&swarm_tracker.0.get(&e1).unwrap().leader.unwrap())
                    {
                        if *e2 == e1 {
                            continue;
                        }
                        // replace this with coerce logic
                        if (t2.translation - t1.translation).length() > b1.swarm_spacing_max {
                            if let Some(sd) = swarm_tracker.0.clone().get_mut(&e1) {
                                if sd.followers == 0 {
                                    swarm_tracker.0.get_mut(&e1).unwrap().state =
                                        SwarmBotState::Solo;
                                    swarm_tracker.0.get_mut(&e1).unwrap().leader = None;
                                    swarm_tracker.1.unit_dec_subs(sd.swarm_id as usize);
                                } else {
                                    swarm_tracker.0.get_mut(&e1).unwrap().leader = Some(e1);
                                }
                            }
                            continue;
                        }
                        // b1.thrust = b2.thrust;
                        b1.target_dir = b2.dir;
                    } else {
                        if let Some(sd) = swarm_tracker.0.clone().get_mut(&e1) {
                            if sd.followers == 0 {
                                swarm_tracker.0.get_mut(&e1).unwrap().state = SwarmBotState::Solo;
                                swarm_tracker.0.get_mut(&e1).unwrap().leader = None;
                                swarm_tracker.1.unit_dec_subs(sd.swarm_id as usize);
                            } else {
                                swarm_tracker.0.get_mut(&e1).unwrap().leader = Some(e1);
                            }
                        }
                    }
                    // change the above block in future

                    // to check if the bot is getting too close to swarm bots
                    for (e2, t2, _) in bots.iter() {
                        if e1 == *e2 {
                            continue;
                        }
                        let separation_vector = t2.translation - t1.translation;
                        if separation_vector.length()
                            < (b1.swarm_spacing_min + b1.swarm_spacing_max) / 2.
                        {
                            // b1.velocity = {
                            //     b1.velocity
                            //         + (-separation_vector.normalize_or_zero() * b1.repel_thrust)
                            //             * time.delta_secs()
                            // };
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
