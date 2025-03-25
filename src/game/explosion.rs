use super::camera::REAR_VIEW_LAYERS;
use super::spaceship::SpaceShip;
use super::{swarm::*, GameObjectMarker};
use crate::asset_loader::SceneAssets;
use crate::sets::*;
use bevy::audio::Volume;
use bevy::render::primitives::Aabb;
use bevy::{
    core_pipeline::bloom::Bloom, gizmos::aabb, prelude::*, render::texture::FallbackImage,
    utils::info,
};
#[derive(Component)]
pub struct ExplosibleObjectMarker;

#[derive(Component)]
pub struct ExplosionMarker;

#[derive(Component, Clone, Copy)]
pub struct Explosion {
    pub max_size: f32,
    pub curr_size: f32,
    pub color: Color,
    pub spread_rate: f32,
    pub shrink_rate: f32,
    pub shrink: bool,
    pub half_extent: f32,
}

impl Default for Explosion {
    fn default() -> Self {
        Self {
            max_size: 2.,
            color: Color::srgba(0.255, 0.255, 0., 0.9),
            spread_rate: 4.,
            curr_size: 1.,
            shrink_rate: 5.,
            shrink: false,
            half_extent: 0.,
        }
    }
}

#[derive(Resource, PartialEq)]
pub struct RunOnce(bool);

pub struct ExplosionPlugin;
impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RunOnce(false))
            .add_event::<ExplosionEvent>()
            // .add_systems(Update, setup.run_if(resource_equals(RunOnce(false))))
            .add_systems(
                Update,
                (explode, animate_explosion)
                    .chain()
                    .in_set(UpdateSet::InGame),
            );
    }
}

#[derive(Event)]
pub struct ExplosionEvent {
    pub transform: Transform,
    pub explosion: Explosion,
    pub sound: Option<Handle<AudioSource>>,
}

fn explode(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_explode: EventReader<ExplosionEvent>,
    scene_assets: Res<SceneAssets>,
) {
    let mut sphere: Handle<Mesh>;
    for ev in ev_explode.read() {
        sphere = meshes.add(
            Sphere {
                radius: ev.explosion.half_extent,
            }
            .mesh()
            .uv(32, 18),
        );

        let exp = commands
            .spawn((
                Mesh3d(sphere.clone()),
                ExplosionMarker,
                ev.explosion,
                ev.transform.clone(),
                MeshMaterial3d(materials.add(StandardMaterial {
                    emissive: LinearRgba::rgb(20.0, 10.0, 0.0) * 2.,
                    emissive_texture: Some(scene_assets.fireball_texture.clone()),
                    alpha_mode: AlphaMode::Add,
                    ..default()
                })),
                GameObjectMarker,
            ))
            .id();
        if let Some(sound) = &ev.sound {
            commands.spawn((
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    paused: false,
                    spatial: true,
                    volume: Volume::new(50.),
                    ..default()
                },
                ev.transform.clone(),
                AudioPlayer(sound.clone()),
            ));
        }
        // }
    }
}

fn animate_explosion(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Explosion), With<ExplosionMarker>>,
    time: Res<Time>,
) {
    for (ent, mut trans, mut explosion) in query.iter_mut() {
        if explosion.shrink == false {
            let rate = explosion.spread_rate;
            explosion.curr_size += rate * time.delta_secs();
            trans.scale = trans.scale + rate * time.delta_secs();
            // info!("spreading {}", explosion.curr_size);
            if explosion.curr_size > explosion.max_size {
                explosion.shrink = true;
            }
        } else {
            let rate = explosion.shrink_rate;

            let dec = rate * time.delta_secs();
            explosion.curr_size -= dec;
            if explosion.curr_size <= 0. {
                explosion.curr_size = 0.;
            }
            // info!("shriking {}", explosion.curr_size);
            trans.scale = Vec3::splat(explosion.curr_size);
            if explosion.curr_size <= 0. {
                commands.entity(ent).despawn_recursive();
                // change it
                // commands.entity(parent.get()).despawn_recursive();
            }
        }
    }
}
