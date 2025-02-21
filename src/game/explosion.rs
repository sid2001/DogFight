use super::spaceship::SpaceShip;
use bevy::math::bounding::Aabb3d;
use bevy::render::primitives::Aabb;
use bevy::render::view::PostProcessWrite;
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
    max_size: f32,
    curr_size: f32,
    color: Color,
    spread_rate: f32,
    shrink_rate: f32,
    shrink: bool,
}

impl Default for Explosion {
    fn default() -> Self {
        Self {
            max_size: 2.,
            color: Color::srgba(0.255, 0.255, 0., 0.9),
            spread_rate: 1.,
            curr_size: 1.,
            shrink_rate: 2.,
            shrink: false,
        }
    }
}

#[derive(Resource, PartialEq)]
pub struct RunOnce(bool);

pub struct ExplosionPlugin;
impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RunOnce(false))
            .add_systems(Update, setup.run_if(resource_equals(RunOnce(false))))
            .add_systems(Update, animate_explosion);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ab_query: Query<(&Aabb, &Name)>,
    player_query: Query<Entity, With<SpaceShip>>,
    mut run_once: ResMut<RunOnce>,
) {
    let mut sphere: Handle<Mesh>;
    let mut half_length_player: f32 = 0.;
    for (aabb, name) in ab_query.iter() {
        if name.as_str() == "SpaceShipMesh" {
            half_length_player = (2. * aabb.half_extents).length();
            run_once.0 = true;
        }
    }
    if run_once.0 == false {
        return;
    }
    for player in player_query.iter() {
        let explosion = Explosion { ..default() };
        // info!("so {:?}", aabb_parent.half_extents.length());
        sphere = meshes.add(
            Sphere {
                radius: half_length_player,
            }
            .mesh()
            .uv(32, 18),
        );
        commands.entity(player).with_child((
            Mesh3d(sphere.clone()),
            ExplosionMarker,
            explosion,
            Transform::from_scale(Vec3::new(1., 1., 1.)),
            MeshMaterial3d(materials.add(StandardMaterial {
                emissive: LinearRgba::rgb(5.32, 2.0, 13.99),
                ..default()
            })),
        ));
    }
    info!("falg1");
}

fn explode(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ab_query: Query<(&Aabb, &Name)>,
    player_query: Query<Entity, With<SpaceShip>>,
    mut run_once: ResMut<RunOnce>,
) {
    let mut sphere: Handle<Mesh>;
    let mut half_length_player: f32 = 0.;
    for (aabb, name) in ab_query.iter() {
        if name.as_str() == "SpaceShipMesh" {
            half_length_player = (2. * aabb.half_extents).length();
            // run_once.0 = true;
            for player in player_query.iter() {
                let explosion = Explosion { ..default() };
                // info!("so {:?}", aabb_parent.half_extents.length());
                sphere = meshes.add(
                    Sphere {
                        radius: half_length_player,
                    }
                    .mesh()
                    .uv(32, 18),
                );
                commands.entity(player).with_child((
                    Mesh3d(sphere.clone()),
                    ExplosionMarker,
                    explosion,
                    Transform::from_scale(Vec3::new(1., 1., 1.)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        emissive: LinearRgba::rgb(5.32, 2.0, 13.99),
                        ..default()
                    })),
                ));
            }
        }
    }
}

fn animate_explosion(
    mut query: Query<(&mut Transform, &mut Explosion), With<ExplosionMarker>>,
    mut mesh: ResMut<Assets<Mesh>>,
    mut ab_query: Query<(&Aabb, &Name)>,
    time: Res<Time>,
) {
    for (mut trans, mut explosion) in query.iter_mut() {
        if explosion.shrink == false {
            let rate = explosion.spread_rate;
            explosion.curr_size += rate * time.delta_secs();
            trans.scale = trans.scale + rate * time.delta_secs();
            info!("spreading {}", explosion.curr_size);
            if explosion.curr_size > explosion.max_size {
                explosion.shrink = true;
            }
        } else {
            let rate = explosion.shrink_rate;
            explosion.curr_size -= rate * time.delta_secs();
            if explosion.curr_size <= 0. {
                explosion.curr_size = 0.1;
            }
            info!("shriking {}", explosion.curr_size);
            trans.scale = trans.scale - rate * time.delta_secs();
            if explosion.curr_size == 0. {
                todo!()
            }
        }
    }
}
