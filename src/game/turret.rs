use std::sync::{Arc, RwLock};
use std::time::Duration;

use super::collider::*;
use super::spaceship::SpaceShip;
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
            .add_systems(Startup, load_bullet)
            // .add_systems(Update, shoot_turret)
            .add_systems(Update, bullet_travel);
    }
}

fn bullet_travel(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet, &mut Transform), With<BulletMarker>>,
    time: Res<Time>,
) {
    for (entity, mut bullet, mut trans) in query.iter_mut() {
        if bullet.distance_covered > DEFAULT_BULLET_RANGE {
            commands.entity(entity).despawn_recursive();
            // info!("despawned");
        } else {
            trans.translation += (bullet.velocity
                + bullet.direction.as_vec3().clone() * bullet.speed)
                * time.delta_secs();
            bullet.distance_covered += ((bullet.velocity
                + bullet.direction.as_vec3().clone() * bullet.speed)
                * time.delta_secs())
            .length();
        }
    }
}

fn load_bullet(
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
    mut query: Query<(&Turret, &GlobalTransform), (With<TurretMarker>, With<T>)>,
    bullet: Res<TurretBullet>,
    mut timer: ResMut<FireRateTimer>,
    time: Res<Time>,
) {
    for (tur, gt) in query.iter_mut() {
        match tur.0.shooting {
            true => {
                if timer.0.tick(time.delta()).just_finished() {
                    commands.spawn((
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
                        ColliderMarker,
                        ColliderInfo {
                            collider_type: ColliderType::Point,
                            collider: Arc::new(RwLock::new(PointCollider { center: Vec3::ZERO })),
                        },
                        CollisionDamage {
                            damage: 20.,
                            from: if tur.0.shooter.is_none() {
                                None
                            } else {
                                Some(tur.0.shooter.unwrap())
                            },
                        },
                    ));
                };
            }
            _ => (),
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
            error!("Entity not present");
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
            error!("Entity not present {}", entity.0.to_bits());
        }
    }
}
