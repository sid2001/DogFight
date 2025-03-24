use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use rand::Rng;
use std::sync::{Arc, RwLock};

use super::debug::{ObstacleInfo, ObstacleMarker};
use super::GameObjectMarker;
use crate::asset_loader::MapOneAssets;
use crate::game::camera::MAIN_CAMERA_LAYER;
use crate::game::camera::NEBULA_LAYER;
use crate::game::collider::{
    ColliderInfo, ColliderMarker, ColliderType, CollisionDamage, SphericalCollider,
};
use crate::game::obstacle::Obstacle;
use crate::sets::*;
use crate::states::*;
use bevy::pbr::*;

pub struct MapOnePlugin;
impl Plugin for MapOnePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InsertedEmissive(false))
            .add_systems(
                OnEnter(GameState::Game),
                (setup_lights, setup)
                    .chain()
                    .in_set(SetupSet::InGame)
                    .run_if(in_state(GameState::Game)),
            )
            .add_systems(
                Update,
                (insert_emissive_property, revolve_satellites)
                    .in_set(UpdateSet::InGame)
                    .run_if(in_state(GameState::Game)),
            );
    }
}

#[derive(Resource, Default)]
pub struct InsertedEmissive(pub bool);

fn setup_lights(mut commands: Commands) {
    let point_light = PointLight {
        color: Color::WHITE,
        range: 1000.,
        intensity: 10000000.,
        ..Default::default()
    };
    // commands.spawn((
    //     DirectionalLight {
    //         color: Color::srgb(1.0, 1.0, 0.9), // Slight yellowish tint for sunlight
    //         illuminance: 10000.0,              // Brightness of the sunlight
    //         shadows_enabled: true,             // Enable shadows
    //         ..default()
    //     },
    //     GameObjectMarker,
    //     // RenderLayers::layer(0).with(1),
    //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)), // 45° angle
    // ));
    commands.spawn((
        point_light.clone(),
        GameObjectMarker,
        Transform::from_xyz(50., 50., 50.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(-50., 50., 50.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(50., -50., 50.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(50., 50., -50.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(-50., -50., 50.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(-50., 50., -50.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(50., -50., -50.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(-50., -50., -50.),
    ));
}

#[derive(Component)]
pub struct Satellite {
    angular_speed: f32,
    distance: f32,
    gravity: f32,
    axis: Vec3,
}

#[derive(Component)]
pub struct SatelliteMarker;

fn setup(
    map_assets: Res<MapOneAssets>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dot_mesh = meshes.add(Sphere::new(0.02).mesh().ico(1).unwrap());
    let satellite_collider_obstacle = (
        ColliderMarker,
        CollisionDamage {
            damage: 1000.,
            from: None,
        },
        ObstacleMarker,
        ObstacleInfo { radius: 2. },
    );
    commands.spawn((
        Satellite {
            angular_speed: 2.,
            distance: 14.,
            gravity: 0.,
            axis: Vec3::Z,
        },
        ColliderInfo {
            collider_type: ColliderType::Sphere,
            collider: Arc::new(RwLock::new(SphericalCollider {
                radius: 2.,
                center: Vec3::ZERO,
            })),
            immune_to: None,
        },
        satellite_collider_obstacle.clone(),
        SatelliteMarker,
        Transform::from_translation(Vec3::new(0., 14., 0.)),
        SceneRoot(map_assets.planet1.clone()),
    ));
    commands.spawn((
        Satellite {
            angular_speed: -5.0,
            distance: 16.,
            gravity: 0.,
            axis: Vec3::Y,
        },
        ColliderInfo {
            collider_type: ColliderType::Sphere,
            collider: Arc::new(RwLock::new(SphericalCollider {
                radius: 2.,
                center: Vec3::ZERO,
            })),
            immune_to: None,
        },
        satellite_collider_obstacle.clone(),
        SatelliteMarker,
        Transform::from_translation(Vec3::new(-16., 0., 0.)),
        SceneRoot(map_assets.planet2.clone()),
    ));
    commands.spawn((
        Satellite {
            angular_speed: 4.0,
            distance: 26.,
            gravity: 0.,
            axis: Vec3::Z,
        },
        ColliderInfo {
            collider_type: ColliderType::Sphere,
            collider: Arc::new(RwLock::new(SphericalCollider {
                radius: 2.,
                center: Vec3::ZERO,
            })),
            immune_to: None,
        },
        satellite_collider_obstacle.clone(),
        SatelliteMarker,
        Transform::from_translation(Vec3::new(10., 24., 0.)),
        SceneRoot(map_assets.planet3.clone()),
    ));
    commands.spawn((
        Satellite {
            angular_speed: 5.,
            distance: 39.,
            gravity: 0.,
            axis: Vec3::X,
        },
        ColliderInfo {
            collider_type: ColliderType::Sphere,
            collider: Arc::new(RwLock::new(SphericalCollider {
                radius: 2.,
                center: Vec3::ZERO,
            })),
            immune_to: None,
        },
        satellite_collider_obstacle.clone(),
        SatelliteMarker,
        Transform::from_translation(Vec3::new(-36., 15., 0.)),
        SceneRoot(map_assets.planet4.clone()),
    ));
    commands.spawn((
        Satellite {
            angular_speed: -5.,
            distance: 34.,
            gravity: 0.,
            axis: Vec3::X,
        },
        ColliderInfo {
            collider_type: ColliderType::Sphere,
            collider: Arc::new(RwLock::new(SphericalCollider {
                radius: 2.,
                center: Vec3::ZERO,
            })),
            immune_to: None,
        },
        satellite_collider_obstacle.clone(),
        SatelliteMarker,
        Transform::from_translation(Vec3::new(4., 10., 32.)),
        SceneRoot(map_assets.planet5.clone()),
    ));
    commands.spawn((
        Satellite {
            angular_speed: 5.,
            distance: 34.,
            gravity: 0.,
            axis: Vec3::Y,
        },
        ColliderInfo {
            collider_type: ColliderType::Sphere,
            collider: Arc::new(RwLock::new(SphericalCollider {
                radius: 2.,
                center: Vec3::ZERO,
            })),
            immune_to: None,
        },
        satellite_collider_obstacle.clone(),
        SatelliteMarker,
        Transform::from_translation(Vec3::new(10., -4., -32.)),
        SceneRoot(map_assets.planet6.clone()),
    ));
    commands.spawn((
        Satellite {
            angular_speed: 2.,
            distance: 34.,
            gravity: 0.,
            axis: Vec3::Z,
        },
        ColliderInfo {
            collider_type: ColliderType::Sphere,
            collider: Arc::new(RwLock::new(SphericalCollider {
                radius: 2.,
                center: Vec3::ZERO,
            })),
            immune_to: None,
        },
        satellite_collider_obstacle.clone(),
        SatelliteMarker,
        Transform::from_translation(Vec3::new(4., 32., 10.)),
        SceneRoot(map_assets.planet7.clone()),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(100.0, 100.0, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::hex("051119").unwrap().into(),
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        NEBULA_LAYER,
        GameObjectMarker,
        // Transform::from_scale(Vec3::splat(20.0)),
        NotShadowCaster,
    ));
    info!("Spawned sun");
    let mat_handle = materials.add(StandardMaterial {
        emissive: LinearRgba::rgb(13.99, 5.32, 10.0),
        ..default()
    });
    let sun_bundle = (
        Transform {
            translation: Vec3::new(0., 0., 0.),
            scale: Vec3::splat(4.),
            ..default()
        },
        Name::new("Sun"),
        ObstacleMarker,
        ObstacleInfo { radius: 4.5 },
        PointLight {
            color: Color::LinearRgba(LinearRgba {
                red: 255.,
                green: 255.,
                blue: 159.,
                alpha: 0.8,
            }),
            range: 50.,
            radius: 5.0,
            intensity: 100000.,
            ..Default::default()
        },
        RenderLayers::layer(0).with(1).with(2),
        // MAIN_CAMERA_LAYER,
        MeshMaterial3d(mat_handle),
        GameObjectMarker,
        SceneRoot(map_assets.sun.clone()),
    );
    let _sun = commands.spawn(sun_bundle).id();
    let mut rng = rand::rng();
    let mut rng2 = rand::rng();
    for i in (-50..50).step_by(20) {
        // if i > -25 && i < 25 {
        //     continue;
        // }
        for j in (-50..50).step_by(10) {
            // if j > -25 && j < 25 {
            //     continue;
            // }
            for k in (-50..50).step_by(10) {
                if i32::pow(i, 2) + i32::pow(j, 2) + i32::pow(k, 2) <= 1600 {
                    continue;
                }
                let x = rng.random_range(-10.0..=10.0) + i as f32;
                let y = rng.random_range(-10.0..=10.0) + j as f32;
                let z = rng.random_range(-10.0..=10.0) + k as f32;
                let (r, g, b) = (
                    rng2.random_range(-1.0..1.0),
                    rng2.random_range(-1.0..1.0),
                    rng2.random_range(-1.0..1.0),
                );
                let spark = rng.random_range(1.0..3.0);
                let transform = Transform::from_xyz(x, y, z);
                commands.spawn((
                    Mesh3d(dot_mesh.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        alpha_mode: AlphaMode::Add,
                        emissive: LinearRgba::rgb(2. + r, 2. + g, 2. + b) * spark,
                        ..default()
                    })),
                    transform,
                    // NEBULA_LAYER,
                ));
            }
        }
    }
    // commands.entity(sun).add_child(cover);
}

fn revolve_satellites(
    mut query: Query<(&Satellite, &mut Transform), With<SatelliteMarker>>,
    time: Res<Time>,
) {
    for (sat, mut trans) in query.iter_mut() {
        trans.rotate_around(
            Vec3::ZERO,
            Quat::from_axis_angle(sat.axis, sat.angular_speed.to_radians() * time.delta_secs()),
        );
        trans.rotate_local_x(sat.angular_speed.to_radians() * 4. * time.delta_secs());
    }
}

fn insert_emissive_property(
    query: Query<(Entity, &Name)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut cond: ResMut<InsertedEmissive>,
) {
    if cond.0 {
        return;
    }
    for (ent, name) in query.iter() {
        if name.to_string() == "sun_mesh_1" {
            cond.0 = true;
            info!("added emissive");
            commands
                .entity(ent)
                .insert(MeshMaterial3d(materials.add(StandardMaterial {
                    // emissive: Color::srgb(1.04942, 0.85883, -0.39299).into(),
                    emissive: LinearRgba::rgb(20.0, 10.0, 0.0),
                    ..Default::default()
                })));
        } else if name.to_string() == "sun_mesh_0" {
            commands
                .entity(ent)
                .insert(MeshMaterial3d(materials.add(StandardMaterial {
                    // emissive: Color::srgb(1.04942, 0.85883, -0.39299).into(),
                    emissive: LinearRgba::rgb(20.0, 20.0, 0.0),
                    ..Default::default()
                })));
        }
    }
}
