use super::movement::{Direction, Inertia};
use super::spaceship::{Entities, SpaceShip};
use super::GameObjectMarker;
use crate::controls::Controls;
use crate::sets::*;
use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::{core_3d::ScreenSpaceTransmissionQuality, tonemapping::Tonemapping};
use bevy::image::ImageSampler;
use bevy::render::camera::Viewport;
use bevy::render::camera::{RenderTarget, ScalingMode};
use bevy::render::view::RenderLayers;
// use bevy::render::render_resource::{
//     Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
// };
use bevy::{pbr::*, prelude::*};
use core::ops::Range;
use std::f32::consts::PI;

// use bevy_core_pipeline::core_3d::graph::Core3d;

const DEFAULT_ANGULAR_SPEED: f32 = 100.;
pub const MAIN_CAMERA_LAYER: RenderLayers = RenderLayers::layer(0);
pub const REAR_VIEW_LAYERS: RenderLayers = RenderLayers::layer(2);
pub const NEBULA_LAYER: RenderLayers = RenderLayers::layer(3);

#[derive(Component)]
pub struct MyCameraMarker;

#[derive(Component)]
pub struct NebulaCamMarker;

pub enum ViewMode {
    FirstPerson((f32, f32)),
    ThirdPerson1((f32, f32)),
    ThirdPerson2((f32, f32)),
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::ThirdPerson1((0.5, 1.))
    }
}

#[derive(Component, Default)]
pub struct CameraMode {
    pub freelook: bool,
    pub view_mode: ViewMode,
    pub toggle_rear_view: bool,
}

#[derive(Component)]
pub struct RearCameraMarker;

#[derive(Bundle)]
pub struct MyCameraBundle {
    marker: MyCameraMarker,
    camera: Camera3d,
    transform: Transform,
    projection: Projection,
    mode: CameraMode,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FrameCounter { frame: 0 })
            // .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    follow_spaceship,
                    camera_view,
                    insert_render_layer,
                    sync_with_cam::<NebulaCamMarker>,
                )
                    .chain()
                    .in_set(UpdateSet::InGame),
            );
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut entities: ResMut<Entities>,
    mut images: ResMut<Assets<Image>>,
) {
    entities.camera = Some(
        commands
            .spawn((
                MyCameraBundle {
                    camera: Camera3d { ..default() },
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: 70.0_f32.to_radians(),
                        ..Default::default()
                    }),
                    transform: Transform::from_xyz(25., 25., 25.).looking_at(Vec3::ZERO, Vec3::Y),
                    marker: MyCameraMarker,
                    mode: CameraMode {
                        freelook: false,
                        ..default()
                    },
                },
                Camera {
                    order: 2,
                    hdr: true,
                    ..default()
                },
                GameObjectMarker,
                Tonemapping::TonyMcMapface,
                Bloom { ..Bloom::NATURAL },
                RenderLayers::from_layers(&[0, 1, 2]),
                // MAIN_CAMERA_LAYER,
                DistanceFog {
                    color: Color::srgba(0.06452, 0.01285, 0.12332, 0.9),
                    directional_light_color: Color::srgba(1.0, 0.95, 0.85, 1.),
                    directional_light_exponent: 60.0,
                    falloff: FogFalloff::from_visibility_colors(
                        1., // distance in world units up to which objects retain visibility (>= 5% contrast)
                        Color::srgb(0.92, 0.91, 0.92), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                        Color::srgb(0.246, 0.245, 0.251), // atmospheric inscattering color (light gained due to scattering from the sun)
                    ),
                    // falloff: FogFalloff::ExponentialSquared { density: (0.1) },
                },
            ))
            .id(),
    );

    commands.spawn((
        Camera3d { ..default() },
        Projection::Perspective(PerspectiveProjection {
            fov: 70.0_f32.to_radians(),
            ..Default::default()
        }),
        Transform::from_xyz(25., 25., 25.).looking_at(Vec3::ZERO, Vec3::Y),
        NebulaCamMarker,
        Camera {
            order: 1,
            hdr: true,
            ..default()
        },
        GameObjectMarker,
        Tonemapping::TonyMcMapface,
        Bloom {
            // intensity: 0.5,
            ..Bloom::SCREEN_BLUR
        },
        // RenderLayers::layer(2).without(0).without(1),
        NEBULA_LAYER,
        DistanceFog {
            color: Color::srgba(0.06452, 0.01285, 0.12332, 0.9),
            directional_light_color: Color::srgba(1.0, 0.95, 0.85, 1.),
            directional_light_exponent: 20.0,
            falloff: FogFalloff::from_visibility_colors(
                1., // distance in world units up to which objects retain visibility (>= 5% contrast)
                Color::srgb(0.92, 0.91, 0.92), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                Color::srgb(0.246, 0.245, 0.251), // atmospheric inscattering color (light gained due to scattering from the sun)
            ),
            // falloff: FogFalloff::ExponentialSquared { density: (0.1) },
        },
    ));

    let rear_camera = commands
        .spawn((
            Camera3d {
                screen_space_specular_transmission_quality: ScreenSpaceTransmissionQuality::Low,
                screen_space_specular_transmission_steps: 0,
                ..default()
            },
            Projection::Perspective(PerspectiveProjection {
                fov: PI / 1.5,
                far: 30.,
                ..Default::default()
            }),
            GameObjectMarker,
            Transform::from_translation(Vec3::ZERO).looking_at(Vec3::ZERO, Vec3::Y),
            Camera {
                // Renders cameras with different priorities to prevent ambiguities
                order: 3,
                // target: RenderTarget::Image(image_handle),
                viewport: Some(Viewport {
                    physical_position: UVec2::new(0, 0),
                    physical_size: UVec2::new(400, 200),
                    // depth: Range {
                    //     start: 0.0,
                    //     end: 0.5,
                    // },
                    ..default()
                }),
                is_active: false,
                ..default()
            },
            RearCameraMarker,
            REAR_VIEW_LAYERS,
        ))
        .id();

    // commands.spawn(image);
    // commands.spawn((
    //     TargetCamera(rear_camera),
    //     Node {
    //         width: Val::Percent(100.),
    //         height: Val::Percent(100.),
    //         position_type: PositionType::Absolute,
    //         border: UiRect::all(Val::Px(2.)),
    //         ..default()
    //     },
    //     BackgroundColor(Color::LinearRgba(LinearRgba {
    //         red: 0.,
    //         green: 1.,
    //         blue: 0.,
    //         alpha: 0.1,
    //     })),
    //     BorderRadius::all(Val::Percent(5.)),
    // ));
}

fn sync_with_cam<T: Component>(
    mut query: Query<&mut Transform, With<T>>,
    mc_query: Query<&Transform, (With<MyCameraMarker>, Without<T>)>,
) {
    if let Ok(mc_trans) = mc_query.get_single() {
        for mut trans in query.iter_mut() {
            *trans = mc_trans.clone();
        }
    }
}

fn follow_spaceship(
    mut cam_query: Query<(&mut Transform, &CameraMode), With<MyCameraMarker>>,
    mut sp_query: Query<
        (&Transform, &mut Direction, &Inertia),
        (With<SpaceShip>, Without<MyCameraMarker>),
    >,
    mut rc_query: Query<
        &mut Transform,
        (
            With<RearCameraMarker>,
            Without<MyCameraMarker>,
            Without<SpaceShip>,
        ),
    >,
    entity: Res<Entities>,
    time: Res<Time>,
) {
    let (trans, sp_dir, iner) = sp_query
        .get_mut(entity.player.unwrap())
        .expect("Error while player!");
    let v = trans.translation.clone();

    let (mut camera, camera_mode) = cam_query
        .get_mut(entity.camera.unwrap())
        .expect("Can't get entitiy camera");

    let (d, s) = match camera_mode.view_mode {
        ViewMode::FirstPerson((x, y)) => (x, y),
        ViewMode::ThirdPerson1((x, y)) => (x, y),
        ViewMode::ThirdPerson2((x, y)) => (x, y),
    };
    //* find a way to calcuate a factor so that the camera speed changes with spacecraft velocity
    // let factor = iner.velocity.0.clone()
    //     - Vec3::new(1., 1., 1.) * iner.velocity.0.clone().normalize_or_zero();

    let cam = camera.translation.clone();
    camera.translation += (v - sp_dir.0.normalize().clone() * d * 2. - cam)
        * if s == 0. {
            1.
        } else {
            time.delta_secs() * (iner.velocity.0.length() * 2. + 2.)
        };

    camera.translation += if s == 0. {
        sp_dir.0.clone().normalize() * 0.2
            + sp_dir.1.clone().cross(sp_dir.0.clone()).normalize() * 0.2
    } else {
        sp_dir.1.clone().cross(sp_dir.0.clone()).normalize() * 0.01
    };

    let rotation = Quat::from_rotation_arc(camera.forward().normalize_or_zero(), sp_dir.0);
    if !camera_mode.freelook {
        camera.rotate(rotation);
    }
    if s == 0. {
        let rot = Quat::from_rotation_arc(
            camera.right().clone().as_vec3().normalize(),
            sp_dir.1.clone().normalize(),
        );
        camera.rotate(rot);
    }
    if let Ok(mut rc_trans) = rc_query.get_single_mut() {
        *rc_trans = Transform::from_translation(camera.translation)
            .with_rotation(camera.rotation)
            .looking_to(-camera.forward(), Vec3::Y)
        // .with_rotation(camera.rotation);
    }
}

fn camera_view(
    mut query: Query<(&mut Transform, &mut CameraMode), (With<MyCameraMarker>, Without<SpaceShip>)>,
    mut sp_query: Query<(&Transform, &mut Visibility), With<SpaceShip>>,
    mut rc_query: Query<&mut Camera, With<RearCameraMarker>>,
    keys: Res<ButtonInput<KeyCode>>,
    controls: Res<Controls>,
    time: Res<Time>,
) {
    for (mut trans, mut camera_mode) in query.iter_mut() {
        let (sp_trans, mut sp_visibility) = sp_query.single_mut();

        if keys.just_pressed(controls.camera_view.unwrap()) {
            camera_mode.view_mode = match camera_mode.view_mode {
                ViewMode::FirstPerson(_) => {
                    *sp_visibility = Visibility::Inherited;
                    ViewMode::ThirdPerson1((0.5, 1.))
                }
                ViewMode::ThirdPerson1(_) => ViewMode::ThirdPerson2((1., 1.)),
                ViewMode::ThirdPerson2(_) => {
                    *sp_visibility = Visibility::Hidden;
                    ViewMode::FirstPerson((0.2, 0.))
                }
            }
        }

        if keys.just_pressed(controls.toggle_rear_view.unwrap()) {
            camera_mode.toggle_rear_view = !camera_mode.toggle_rear_view;
            if camera_mode.toggle_rear_view {
                if let Ok(mut rear_cam) = rc_query.get_single_mut() {
                    rear_cam.is_active = true;
                }
            } else {
                if let Ok(mut rear_cam) = rc_query.get_single_mut() {
                    rear_cam.is_active = false;
                }
            }
        }

        if keys.just_released(controls.toggle_freelook.unwrap()) {
            camera_mode.freelook = false;
        }
        if keys.just_pressed(controls.align_camera.unwrap()) {
            trans.look_to(sp_trans.forward(), sp_trans.up());
        }
        if keys.pressed(controls.toggle_freelook.unwrap()) {
            camera_mode.freelook = true;
            if keys.pressed(controls.camera_up.unwrap()) {
                let axis = sp_trans.right().clone();
                trans.rotate_axis(
                    axis,
                    DEFAULT_ANGULAR_SPEED.to_radians().clone() * time.delta_secs(),
                );
            }

            if keys.pressed(controls.camera_down.unwrap()) {
                let axis = sp_trans.left().clone();
                trans.rotate_axis(
                    axis,
                    DEFAULT_ANGULAR_SPEED.to_radians().clone() * time.delta_secs(),
                );
            }

            if keys.pressed(controls.camera_l.unwrap()) {
                let axis = sp_trans
                    .forward()
                    .as_vec3()
                    .cross(sp_trans.right().as_vec3())
                    .normalize();
                trans.rotate_axis(
                    Dir3::new(-axis).unwrap(),
                    DEFAULT_ANGULAR_SPEED.to_radians().clone() * time.delta_secs(),
                );
            }

            if keys.pressed(controls.camera_r.unwrap()) {
                let axis = sp_trans
                    .forward()
                    .as_vec3()
                    .cross(sp_trans.right().as_vec3())
                    .normalize();
                trans.rotate_axis(
                    Dir3::new(axis).unwrap(),
                    DEFAULT_ANGULAR_SPEED.to_radians().clone() * time.delta_secs(),
                );
            }
        }
    }
}

#[derive(Resource)]
struct FrameCounter {
    frame: usize,
}

fn rear_view_frame_limit(
    mut query: Query<&mut Camera, With<RearCameraMarker>>,
    mut frame_counter: ResMut<FrameCounter>,
) {
    frame_counter.frame += 1;
    let render_now = frame_counter.frame % 3 == 0; // Update every 10 frames

    for mut cam in query.iter_mut() {
        cam.is_active = render_now;
    }
}

fn insert_render_layer(
    query: Query<(Entity, &Name), Without<RenderLayers>>,
    mut commands: Commands,
) {
    for (ent, name) in query.iter() {
        if name.to_string() == "BotMesh" {
            commands.entity(ent).insert(REAR_VIEW_LAYERS);
        }
    }
}
