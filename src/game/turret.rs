use std::sync::{Arc, RwLock};
use std::time::Duration;

use super::collider;
use super::spaceship::Health;
use super::{collider::*, GameObjectMarker};
use crate::sets::*;
use crate::states::*;
use bevy::prelude::*;

const DEFAULT_BULLET_RANGE: f32 = 20.;

//* Add code for input */
// #[derive(Component)]
// pub enum Turret {
//     PlayerTurret(u32),
//     EnemyTurret(u32),
// }

#[derive(Component, Default)]
pub struct TurretMarker;
#[derive(Component)]
pub struct BulletMarker;

#[derive(Component)]
#[require(SceneRoot, Transform, TurretMarker)]
pub struct Turret(pub TurretBundle);

#[derive(Component)]
pub struct TurretShooting(bool);
// #[derive(Bundle)]
pub struct TurretBundle {
    pub shooting: bool,
    pub speed: f32,
    pub direction: Dir3,
    pub bullet_size: f32,
    pub shooter: Option<Entity>,
    pub cooldown_time: f32,
    pub cooldown: f32,
    pub overheat_limit: f32,
    pub overheat: bool,
    pub bullet_inertial_velocity: Vec3,
}

#[derive(Event)]
pub struct ShootTurretEventOn(pub Entity);
#[derive(Event)]
pub struct ShootTurretEventOff(pub Entity);
pub trait TurretSoundEventPlugin: Plugin {
    fn build(&self, _: &mut App);
}

// pub struct TurretSoundEventPlugin;

impl Default for TurretBundle {
    fn default() -> Self {
        Self {
            shooting: false,
            speed: 10.,
            direction: Dir3::Y,
            bullet_size: 1.,
            bullet_inertial_velocity: Vec3::ZERO,
            shooter: None,
            overheat_limit: 2.,
            cooldown: 0.,
            cooldown_time: 3.,
            overheat: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct TurretBullet {
    pub handle: Handle<Scene>,
}

#[derive(Resource, Default)]
pub struct BulletScenePath(String);

#[derive(Component)]
pub struct Bullet {
    pub speed: f32,
    pub direction: Dir3,
    pub velocity: Vec3,
    pub distance_covered: f32,
}

#[derive(Resource, Default)]
pub struct FireRateTimer(Timer);
pub struct TurretPlugin {
    pub bullet_scene_path: String,
}
impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TurretBullet>()
            .insert_resource(FireRateTimer(Timer::new(
                Duration::from_secs_f32(0.08),
                TimerMode::Repeating,
            )))
            .insert_resource(BulletScenePath(self.bullet_scene_path.clone()))
            .add_systems(OnEnter(InGameStates::Setup), setup.in_set(SetupSet::InGame))
            // .add_systems(Update, shoot_turret)
            .add_systems(
                Update,
                (
                    bullet_travel,
                    collider::collision_response::<BulletMarker>,
                    despawn_bullet,
                )
                    .in_set(UpdateSet::InGame),
            );
    }
}

fn bullet_travel(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet, &mut Transform), With<BulletMarker>>,
    time: Res<Time>,
) {
    for (entity, mut bullet, mut trans) in query.iter_mut() {
        // if bullet.distance_covered > DEFAULT_BULLET_RANGE {
        //     commands.entity(entity).despawn_recursive();
        //     // info!("despawned");
        // } else {
        trans.translation += (bullet.velocity + bullet.direction.as_vec3().clone() * bullet.speed)
            * time.delta_secs();
        bullet.distance_covered += ((bullet.velocity
            + bullet.direction.as_vec3().clone() * bullet.speed)
            * time.delta_secs())
        .length();
        // }
    }
}

fn despawn_bullet(
    mut commands: Commands,
    query: Query<(Entity, &Bullet, &Health), With<BulletMarker>>,
) {
    for (entity, bullet, health) in query.iter() {
        if bullet.distance_covered > DEFAULT_BULLET_RANGE || health.0 <= 0. {
            info!("bulllet");
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn setup(
    asset_server: Res<AssetServer>,
    path: Res<BulletScenePath>,
    mut bullet: ResMut<TurretBullet>,
) {
    *bullet = TurretBullet {
        handle: asset_server.load(path.0.clone()),
    };
}

pub fn shoot_turret<T: Component>(
    mut commands: Commands,
    mut query: Query<(&mut Turret, &GlobalTransform), (With<TurretMarker>, With<T>)>,
    bullet: Res<TurretBullet>,
    mut timer: ResMut<FireRateTimer>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut tur, gt) in query.iter_mut() {
        match tur.0.shooting {
            true => {
                if timer.0.tick(time.delta()).just_finished() {
                    if tur.0.overheat {
                        continue;
                    }
                    tur.0.cooldown += 0.08;
                    commands.spawn((
                        GameObjectMarker,
                        SceneRoot(bullet.handle.clone()),
                        Transform::from_translation(gt.translation().clone())
                            .with_scale(Vec3::ONE * tur.0.bullet_size.clone())
                            .with_rotation(gt.rotation().clone()),
                        BulletMarker,
                        Bullet {
                            speed: tur.0.speed.clone(),
                            direction: gt.forward(),
                            velocity: tur.0.bullet_inertial_velocity,
                            distance_covered: 0.,
                        },
                        Health(5.),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            emissive: LinearRgba::rgb(5.32, 2.0, 13.99),
                            ..default()
                        })),
                        ColliderMarker,
                        ColliderInfo {
                            collider_type: ColliderType::Point,
                            collider: Arc::new(RwLock::new(PointCollider { center: Vec3::ZERO })),
                            // ikik not checking for none value cause it won't happen;source: trust me bro
                            immune_to: Some(Vec::from([tur.0.shooter.unwrap()])),
                        },
                        CollisionDamage {
                            damage: 20.,
                            from: tur.0.shooter,
                        },
                    ));
                    if tur.0.cooldown >= tur.0.overheat_limit {
                        tur.0.overheat = true;
                        tur.0.cooldown = tur.0.cooldown_time;
                    }
                };
            }
            false => {
                if tur.0.overheat {
                    tur.0.cooldown -= time.delta_secs();
                    if tur.0.cooldown <= 0. {
                        tur.0.overheat = false;
                        tur.0.cooldown = 0.;
                    }
                } else if tur.0.cooldown > 0. {
                    tur.0.cooldown -= time.delta_secs();
                    if tur.0.cooldown <= 0. {
                        tur.0.cooldown = 0.;
                    }
                }
            }
        }
    }
}

pub fn turret_sound_on(
    mut ev_turret_on: EventReader<ShootTurretEventOn>,
    mut query: Query<&SpatialAudioSink, With<TurretMarker>>,
) {
    for entity in ev_turret_on.read() {
        if let Ok(bun) = query.get_mut(entity.0) {
            if bun.is_paused() {
                bun.play();
            }
        } else {
            // error!("Entity not present");
        }
    }
}

pub fn turret_sound_off(
    mut ev_turret_off: EventReader<ShootTurretEventOff>,
    mut query: Query<&SpatialAudioSink, With<TurretMarker>>,
) {
    for entity in ev_turret_off.read() {
        if let Ok(bun) = query.get_mut(entity.0) {
            error!("recv audio down sound {}", entity.0.to_bits());
            if !bun.is_paused() {
                bun.pause();
            }
        } else {
            // error!("Entity not present {}", entity.0.to_bits());
        }
    }
}
