use std::mem::transmute_copy;
use std::{default, time::Duration};

use bevy::audio::{SpatialScale, Volume};
use bevy::{prelude::*, state::commands};

use super::collider::{
    collision_response, ColliderInfo, ColliderMarker, ColliderType, CollisionDamage,
    SphericalCollider,
};
use super::explosion::ExplosibleObjectMarker;
use super::spaceship::Health;
use super::GameObjectMarker;
use crate::asset_loader::{AudioAssets, SceneAssets};
use crate::sets::UpdateSet;
use std::f32::consts::PI;
use std::sync::{Arc, RwLock};

const HOMING_MISSILE_DAMAGE: f32 = 100.;
const SWARM_MISSILE_DAMAGE: f32 = 20.;
const MISSILE_DESTRUCT_TIME: f32 = 5.;

const MISSILE_OFFSET: Transform = Transform::from_xyz(0., 0., 0.);

#[derive(Default)]
pub enum LauncherState {
    Aiming,
    Locked,
    #[default]
    Ideal,
}

#[derive(Component)]
pub struct SwarmMissileMarker;
#[derive(Component)]
pub struct HomingMissileMarker;

#[derive(Component)]
pub struct MissileMarker;

#[derive(Component)]
pub struct HomingMissileTarget;
#[derive(Component)]
pub struct SwarmMissileTarget;

#[derive(Component, Default)]
pub struct HomingMissileLauncher {
    pub state: LauncherState,
    pub target: Option<Entity>,
    pub source: Option<Entity>,
}

#[derive(Component, Default)]
pub struct SwarmMissileLauncher {
    pub source: Option<Entity>,
}

pub enum MissileType {
    SwarmMissile,
    HomingMissile,
}

#[derive(Event)]
pub struct HomingMissileShootEvent {
    pub launcher: Entity,
    pub missile: Missile,
}

#[derive(Clone)]
pub enum SwarmMissileStage {
    Stage1(Dir3),
    Stage2(Option<Entity>, Vec3),
}

#[derive(Component, Clone)]
pub struct SwarmMissile {
    pub source: Entity,
    pub stage: SwarmMissileStage,
    pub initial_speed: f32,
    pub speed: f32,
    pub timer: Duration,
    pub target: Option<Entity>,
    pub converge_point: Vec3,
    pub angluar_speed: f32,
}

#[derive(Event, Clone)]
pub struct SwarmMissileShootEvent {
    pub launcher: Entity,
    pub missile: SwarmMissile,
}

#[derive(Component, Clone)]
pub struct Missile {
    pub source: Entity,
    pub is_locked: bool,
    pub initial_speed: f32,
    pub target: Option<Entity>,
    pub thrust: f32,
    pub timer: Duration,
    pub damage: f32,
    pub velocity: Vec3,
    pub drag: Vec3,
    pub angular_speed: f32,
}

type HomingMissile = Missile;
// type SwarmMissile = Missile;

pub struct MissilePlugin;
impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HomingMissileShootEvent>()
            .add_event::<SwarmMissileShootEvent>()
            .add_systems(
                Update,
                (
                    launch_homing_missile,
                    launch_swarm_missile,
                    move_swarm_missile,
                    move_missile,
                )
                    .in_set(UpdateSet::InGame),
            )
            .add_systems(
                Update,
                (
                    collision_response::<HomingMissileMarker>,
                    collision_response::<SwarmMissileMarker>,
                    despawn_swarm_missile,
                )
                    .chain()
                    .in_set(UpdateSet::InGame),
            );
    }
}

fn launch_swarm_missile(
    mut ev_swarm_missile: EventReader<SwarmMissileShootEvent>,
    mut commands: Commands,
    query: Query<(&GlobalTransform, &SwarmMissileLauncher), With<SwarmMissileLauncher>>,
    scene_asset: Res<SceneAssets>,
    audio_asset: Res<AudioAssets>,
) {
    for SwarmMissileShootEvent { launcher, missile } in ev_swarm_missile.read() {
        if let Ok((gt, swarm_missile_launcher)) = query.get(*launcher) {
            let transform = gt.compute_transform();
            let bundle = (
                missile.clone(),
                GameObjectMarker,
                SwarmMissileMarker,
                Health(1.),
                AudioPlayer(audio_asset.homing_cruise.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    paused: false,
                    spatial: true,
                    // volume: Volume::new(1.),
                    ..default()
                },
                ColliderMarker,
                ExplosibleObjectMarker,
                ColliderInfo {
                    collider_type: ColliderType::Sphere,
                    collider: Arc::new(RwLock::new(SphericalCollider {
                        center: Vec3::ZERO,
                        radius: 0.02,
                    })),
                    immune_to: Some(Vec::from([swarm_missile_launcher.source.unwrap()])),
                },
                CollisionDamage {
                    damage: 100.,
                    from: swarm_missile_launcher.source,
                },
                transform,
                SceneRoot(scene_asset.missile.clone()),
            );
            let sound_effect = (
                AudioPlayer(audio_asset.swarm_missile_launch.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    paused: false,
                    ..Default::default()
                },
            );
            commands.spawn(sound_effect);
            commands.spawn(bundle);
        }
    }
}

fn launch_homing_missile(
    mut ev_homing: EventReader<HomingMissileShootEvent>,
    query: Query<(&GlobalTransform, &HomingMissileLauncher), With<HomingMissileLauncher>>,
    mut commands: Commands,
    scene_asset: Res<SceneAssets>,
    audio_asset: Res<AudioAssets>,
) {
    for HomingMissileShootEvent { launcher, missile } in ev_homing.read() {
        if let Ok((gt, homing_launcher)) = query.get(*launcher) {
            let transform = gt.compute_transform();
            let bundle = (
                missile.clone(),
                GameObjectMarker,
                HomingMissileMarker,
                MissileMarker,
                Health(1.),
                AudioPlayer(audio_asset.homing_cruise.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    paused: false,
                    spatial: true,
                    spatial_scale: Some(SpatialScale::new(2.)),
                    volume: Volume::new(30.),
                    ..default()
                },
                ColliderMarker,
                ExplosibleObjectMarker,
                ColliderInfo {
                    collider_type: ColliderType::Sphere,
                    collider: Arc::new(RwLock::new(SphericalCollider {
                        center: Vec3::ZERO,
                        radius: 0.05,
                    })),
                    immune_to: Some(Vec::from([homing_launcher.source.unwrap()])),
                },
                CollisionDamage {
                    damage: 1000.,
                    from: homing_launcher.source,
                },
                transform,
                SceneRoot(scene_asset.missile2.clone()),
            );
            let sound_effect = (
                AudioPlayer(audio_asset.homing_launch.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    paused: false,
                    ..Default::default()
                },
            );
            commands.spawn(sound_effect);
            commands.spawn(bundle);
        }
    }
}

fn move_missile(
    mut query: Query<(Entity, &mut Transform, &mut Missile, &Health), With<MissileMarker>>,
    t_query: Query<&Transform, (With<HomingMissileTarget>, Without<MissileMarker>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (ent, mut trans, mut missile, heatlh) in query.iter_mut() {
        if heatlh.0 <= 0. {
            commands.entity(ent).despawn_recursive();
        }
        if missile.is_locked {
            if let Ok(t_trans) = t_query.get(missile.target.unwrap()) {
                let dir_vec = t_trans.translation - trans.translation;
                let rot_axis = trans
                    .forward()
                    .cross(dir_vec.normalize_or(Vec3::Y))
                    .normalize_or(Vec3::Y);
                let rotation = Quat::from_axis_angle(
                    rot_axis.normalize_or(Vec3::Y),
                    missile.angular_speed.to_radians() * time.delta_secs(),
                );
                trans.rotate(rotation);
            }
        }

        let spin_axis = trans.forward();
        // trans.rotate_axis(spin_axis, PI * 2. * time.delta_secs());
        missile.velocity = missile.velocity
            + ((missile.thrust * trans.forward().as_vec3()) + missile.drag) * time.delta_secs();
        missile.drag = -missile.velocity;
        trans.translation += missile.velocity * time.delta_secs();
        missile.timer += time.delta();
        if missile.timer.as_secs_f32() > MISSILE_DESTRUCT_TIME {
            commands.entity(ent).despawn();
        }
    }
}

fn move_swarm_missile(
    mut query: Query<
        (Entity, &mut Transform, &mut SwarmMissile, &mut Health),
        With<SwarmMissileMarker>,
    >,
    t_query: Query<&Transform, (With<SwarmMissileTarget>, Without<SwarmMissileMarker>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (ent, mut s_trans, mut missile, mut health) in query.iter_mut() {
        let stage = missile.stage.clone();
        missile.timer = missile.timer + time.delta();
        'block1: {
            match stage {
                SwarmMissileStage::Stage1(dir) => {
                    let axis = s_trans.forward().cross(dir.as_vec3());
                    let fallback = s_trans.local_z();
                    s_trans.rotate_axis(
                        Dir3::new(axis.normalize_or(fallback.as_vec3())).unwrap(),
                        (missile.angluar_speed * time.delta_secs()).to_radians(),
                    );

                    // todo this may not work
                    if missile.timer.as_secs_f32() >= 0.5 {
                        missile.stage =
                            SwarmMissileStage::Stage2(missile.target, missile.converge_point);
                    }
                    missile.speed =
                        -16.0 * (2.0 * missile.timer.as_secs_f32() - 0.5).powf(2.0) + 4.0;
                }
                SwarmMissileStage::Stage2(Some(target), _) => {
                    if let Ok(trans) = t_query.get(target) {
                        // excute explosion before
                        let dir_vec = (trans.translation - s_trans.translation).normalize_or_zero();
                        let axis = s_trans.forward().cross(dir_vec);
                        if axis.length_squared() == 0. {
                            break 'block1;
                        }
                        let th = dir_vec.dot(s_trans.forward().as_vec3()).acos();
                        let angle = (missile.angluar_speed * 5. * time.delta_secs()).to_radians();
                        s_trans.rotate_axis(
                            Dir3::new(axis.normalize()).unwrap(),
                            if angle > th { th } else { angle },
                        );
                    } else {
                        missile.target = None;
                    }
                    missile.speed = 5.0_f32.powf(2.0 * missile.timer.as_secs_f32() - 1.0);
                    missile.speed = if missile.speed > 12.0 {
                        12.0
                    } else {
                        missile.speed
                    };
                }
                SwarmMissileStage::Stage2(None, point) => {
                    let vec_dir = point - s_trans.translation;
                    if vec_dir.length_squared() < 0.01 {
                        health.0 = 0.;
                    }
                    let axis = s_trans
                        .forward()
                        .cross(vec_dir.normalize())
                        .normalize_or_zero();
                    if axis.length_squared() == 0. {
                        break 'block1;
                    }
                    s_trans.rotate_axis(
                        Dir3::new(axis).unwrap(),
                        missile.angluar_speed * 2.0 * time.delta_secs(),
                    );
                }
            }
        }
        s_trans.rotate_local_z(PI * 4.0 * time.delta_secs());
        s_trans.translation = s_trans.translation
            + (missile.speed + missile.initial_speed)
                * s_trans.forward().as_vec3()
                * time.delta_secs();
        if missile.timer.as_secs_f32() > 2.0 {
            health.0 = 0.0;
        }
    }
}

fn despawn_swarm_missile(
    mut commands: Commands,
    query: Query<(Entity, &Health), With<SwarmMissileMarker>>,
) {
    for (ent, health) in query.iter() {
        if health.0 <= 0.0 {
            commands.entity(ent).despawn_recursive();
        }
    }
}
