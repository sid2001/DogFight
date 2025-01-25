use super::movement::{Direction, Inertia};
use super::spaceship::{Entities, SpaceShip};
use crate::controls::Controls;
use bevy::math::VectorSpace;
use bevy::{pbr::*, prelude::*};

const DEFAULT_ANGULAR_SPEED: f32 = 100.;

#[derive(Component)]
pub struct MyCameraMarker;

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
}

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
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, (follow_spaceship, camera_view));
    }
}

pub fn setup_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut entities: ResMut<Entities>,
) {
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 1.0, 0.9), // Slight yellowish tint for sunlight
            illuminance: 2000.0,               // Brightness of the sunlight
            shadows_enabled: true,             // Enable shadows
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)), // 45° angle
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(100.0, 100.0, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::hex("9da5bd").unwrap().into(),
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_scale(Vec3::splat(20.0)),
        NotShadowCaster,
    ));
    entities.camera = Some(
        commands
            .spawn((
                MyCameraBundle {
                    camera: Camera3d { ..default() },
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: 70.0_f32.to_radians(),
                        ..Default::default()
                    }),
                    transform: Transform::from_xyz(0.0, -6.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
                    marker: MyCameraMarker,
                    mode: CameraMode {
                        freelook: false,
                        ..default()
                    },
                },
                DistanceFog {
                    color: Color::srgba(0.35, 0.48, 0.66, 0.2),
                    directional_light_color: Color::srgba(1.0, 0.95, 0.85, 1.),
                    directional_light_exponent: 60.0,
                    falloff: FogFalloff::from_visibility_colors(
                        0.007, // distance in world units up to which objects retain visibility (>= 5% contrast)
                        Color::srgb(0.92, 0.91, 0.92), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                        Color::srgb(0.246, 0.245, 0.251), // atmospheric inscattering color (light gained due to scattering from the sun)
                    ),
                    // falloff: FogFalloff::ExponentialSquared { density: (0.1) },
                },
            ))
            .id(),
    );
}

fn follow_spaceship(
    mut cam_query: Query<(&mut Transform, &CameraMode), With<MyCameraMarker>>,
    mut sp_query: Query<
        (&Transform, &mut Direction, &Inertia),
        (With<SpaceShip>, Without<MyCameraMarker>),
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
            + sp_dir.1.clone().cross(sp_dir.0.clone()).normalize() * 0.1
    } else {
        sp_dir.1.clone().cross(sp_dir.0.clone()).normalize() * 0.01
    };

    let rotation = Quat::from_rotation_arc(camera.forward().normalize_or_zero(), sp_dir.0);
    // let rot_axis = trans
    //     .forward()
    //     .as_vec3()
    //     .cross(sp_dir.0)
    //     .normalize_or_zero();
    // let angle = camera.forward().as_vec3().clone().angle_between(sp_dir.0);

    // rotation = Quat::from_axis_angle(rot_axis, angle.to_radians() * time.delta_secs() * 10.);
    //* this is a correct method but give camera it's own coordinate plane axes
    //* let angle = (sp_dir
    //*     .0
    //*     .clone()
    //*     .normalize_or_zero()
    //*     .dot(camera.forward().clone().normalize_or_zero()))
    //* .acos();
    //* rotation = Quat::from_axis_angle(
    //*     sp_dir.1.clone().cross(sp_dir.0.clone()),
    //*     angle * time.delta_seconds(),
    //* );
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
}

fn camera_view(
    mut query: Query<(&mut Transform, &mut CameraMode), (With<MyCameraMarker>, Without<SpaceShip>)>,
    mut sp_query: Query<(&Transform, &mut Visibility), With<SpaceShip>>,
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
