use std::{default, time::Duration};

use bevy::audio::Volume;
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

pub enum MissileType {
    SwarmMissile,
    HomingMissile,
}

#[derive(Event)]
pub struct HomingMissileShootEvent {
    pub launcher: Entity,
    pub missile: Missile,
}

#[derive(Event)]
pub struct SwarmMissileShootEvent(pub Entity);

#[derive(Component)]
pub struct SwarmMissileLauncher;

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
type SwarmMissile = Missile;

pub struct MissilePlugin;
impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HomingMissileShootEvent>().add_systems(
            Update,
            (
                launch_homing_missile,
                move_missile,
                collision_response::<HomingMissileMarker>,
            )
                .in_set(UpdateSet::InGame),
        );
    }
}

fn launch_homing_missile(
    mut ev_homing: EventReader<HomingMissileShootEvent>,
    query: Query<(&GlobalTransform, &HomingMissileLauncher), With<HomingMissileLauncher>>,
    // target_query: Query<&Transform, With<HomingMissileTarget>>,
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
