use std::ptr::null;

use bevy::{math::VectorSpace, prelude::*, state::commands};

const DEFAULT_BULLET_RANGE: f32 = 50.;

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
        }
    }
}

#[derive(Resource, Default)]
pub struct TurretBullet {
    pub handle: Handle<Scene>,
}

#[derive(Resource)]
pub struct BulletScenePath(String);

#[derive(Component)]
pub struct Bullet {
    pub speed: f32,
    pub direction: Dir3,
    pub velocity: Vec3,
    pub distance_covered: f32,
}

pub struct TurretPlugin {
    pub bullet_scene_path: String,
}
impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TurretBullet>()
            .insert_resource(BulletScenePath(self.bullet_scene_path.clone()))
            .add_systems(Startup, load_bullet)
            .add_systems(Update, shoot_turret)
            .add_systems(Update, bullet_travel);
    }
}

fn bullet_travel(
    mut commands: Commands,
    mut query: Query<(Entity, &Bullet, &mut Transform), With<BulletMarker>>,
    time: Res<Time>,
) {
    for (entity, bullet, mut trans) in query.iter_mut() {
        if bullet.distance_covered > DEFAULT_BULLET_RANGE {
            commands.entity(entity).despawn_recursive();
        } else {
            trans.translation += (bullet.velocity
                + bullet.direction.as_vec3().clone() * bullet.speed)
                * time.delta_secs();
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

fn shoot_turret(
    mut commands: Commands,
    query: Query<(Entity, &Turret, &Transform, &GlobalTransform), With<TurretMarker>>,
    bullet: Res<TurretBullet>,
) {
    for (entity, tur, trans, gt) in query.iter() {
        match &tur.0.shooting {
            true => {
                commands.spawn((
                    SceneRoot(bullet.handle.clone()),
                    Transform::from_translation(gt.translation().clone())
                        .with_scale(Vec3::ONE * tur.0.bullet_size.clone())
                        .with_rotation(trans.rotation.clone()),
                    BulletMarker,
                    Bullet {
                        speed: tur.0.speed.clone(),
                        direction: gt.forward(),
                        velocity: tur.0.bullet_inertial_velocity,
                        distance_covered: 0.,
                    },
                ));
            }
            _ => (),
        }
    }
}

pub fn turret_sound_on(
    mut ev_turret_on: EventReader<ShootTurretEventOn>,
    mut query: Query<&AudioSink, With<TurretMarker>>,
) {
    error!("turret sound1");
    for entity in ev_turret_on.read() {
        error!("turret sound");
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
    mut query: Query<&AudioSink, With<TurretMarker>>,
) {
    for entity in ev_turret_off.read() {
        if let Ok(ref mut bun) = query.get_mut(entity.0) {
            // error!("up dv");
            if !bun.is_paused() {
                bun.pause();
            }
        } else {
            error!("Entity not present");
        }
    }
}
