use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::time::common_conditions::once_after_delay;
use bevy_inspector_egui::egui::Rgba;
use rand::Rng;

use super::GameObjectMarker;
use crate::asset_loader::MapOneAssets;
use crate::game::camera::MAIN_CAMERA_LAYER;
use crate::game::camera::NEBULA_LAYER;
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
                insert_emissive_property
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
        Transform::from_xyz(100., 100., 100.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(-100., 100., 100.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(100., -100., 100.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(100., 100., -100.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(-100., -100., 100.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(-100., 100., -100.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(100., -100., -100.),
    ));
    commands.spawn((
        point_light,
        GameObjectMarker,
        Transform::from_xyz(-100., -100., -100.),
    ));
}

fn setup(
    map_assets: Res<MapOneAssets>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dot_material_emissive = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::rgb(2., 2., 2.),
        ..default()
    });
    let dot_mesh = meshes.add(Sphere::new(0.02).mesh().ico(1).unwrap());

    // let cover = commands
    //     .spawn((
    //         Mesh3d(mesh.clone()),
    //         MeshMaterial3d(material_emissive2),
    //         // Transform::from_xyz(-5., -5., -5.),
    //         // NEBULA_LAYER,
    //     ))
    //     .id();
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
            translation: Vec3::new(5., 5., 5.),
            scale: Vec3::splat(4.),
            ..default()
        },
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
        SceneRoot(map_assets.sun.clone()),
    );
    let _sun = commands.spawn(sun_bundle).id();
    let mut rng = rand::rng();
    let mut rng2 = rand::rng();
    for i in (-20..20).step_by(10) {
        for j in (-20..20).step_by(5) {
            for k in (-20..20).step_by(5) {
                let x = rng.random_range(-10.0..=10.0) + i as f32;
                let y = rng.random_range(-2.0..=5.0) + j as f32;
                let z = rng.random_range(-2.0..=5.0) + k as f32;
                let (r, g, b) = (
                    rng2.random_range(-1.0..1.0),
                    rng2.random_range(-1.0..1.0),
                    rng2.random_range(-1.0..1.0),
                );
                let spark = rng.random_range(1.0..2.0);
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
