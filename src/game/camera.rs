use super::movement::{Direction, Inertia};
use super::spaceship::{Entities, SpaceShip};
use bevy::{pbr::*, prelude::*};

const DEFAULT_ANGULAR_SPEED: f32 = 100.;

#[derive(Component)]
pub struct MyCameraMarker;

#[derive(Component, Default)]
pub struct CameraMode {
    pub freelook: bool,
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
            .add_systems(Update, (follow_spaceship, free_look));
    }
}

pub fn setup_camera(mut commands: Commands, mut entities: ResMut<Entities>) {
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 1.0, 0.9), // Slight yellowish tint for sunlight
            illuminance: 100000.0,             // Brightness of the sunlight
            shadows_enabled: true,             // Enable shadows
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)), // 45° angle
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
                    transform: Transform::from_xyz(0.0, -10.0, 0.0).looking_at(Vec3::Y, Vec3::Z),
                    marker: MyCameraMarker,
                    mode: CameraMode { freelook: false },
                },
                // DistanceFog {
                //     color: Color::srgba(0.35, 0.48, 0.66, 1.0),
                //     directional_light_color: Color::srgba(1.0, 0.95, 0.85, 0.5),
                //     directional_light_exponent: 30.0,
                //     falloff: FogFalloff::from_visibility_colors(
                //         15.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                //         Color::srgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                //         Color::srgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
                //     ),
                // },
            ))
            .id(),
    );
}

fn follow_spaceship(
    mut cam_query: Query<(&mut Transform, &CameraMode), With<MyCameraMarker>>,
    sp_query: Query<
        (&Transform, &mut Direction, &Inertia),
        (With<SpaceShip>, Without<MyCameraMarker>),
    >,
    entity: Res<Entities>,
    time: Res<Time>,
) {
    let (trans, sp_dir, iner) = sp_query
        .get(entity.player.unwrap())
        .expect("Error while player!");
    let v = trans.translation.clone();

    let (mut camera, camera_mode) = cam_query
        .get_mut(entity.camera.unwrap())
        .expect("Can't get entitiy camera");

    //* find a way to calcuate a factor so that the camera speed changes with spacecraft velocity
    // let factor = iner.velocity.0.clone()
    //     - Vec3::new(1., 1., 1.) * iner.velocity.0.clone().normalize_or_zero();

    let cam = camera.translation.clone();
    camera.translation += (v - sp_dir.0.normalize().clone() - cam)
        * time.delta_secs()
        * (iner.velocity.0.length() * 2. + 2.);
    camera.translation += sp_dir.1.clone().cross(sp_dir.0.clone()).normalize() * 0.005;

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
}

fn free_look(
    mut query: Query<(&mut Transform, &mut CameraMode), (With<MyCameraMarker>, Without<SpaceShip>)>,
    sp_query: Query<&Transform, With<SpaceShip>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut trans, mut camera_mode) in query.iter_mut() {
        let sp_trans = sp_query.single();
        if keys.just_released(KeyCode::ControlLeft) {
            camera_mode.freelook = false;
        }
        if keys.just_pressed(KeyCode::AltRight) {
            trans.look_to(sp_trans.forward(), sp_trans.up());
        }
        if keys.pressed(KeyCode::ControlLeft) {
            camera_mode.freelook = true;
            if keys.pressed(KeyCode::ArrowUp) {
                let axis = sp_trans.right().clone();
                trans.rotate_axis(
                    axis,
                    DEFAULT_ANGULAR_SPEED.to_radians().clone() * time.delta_secs(),
                );
            }

            if keys.pressed(KeyCode::ArrowDown) {
                let axis = sp_trans.left().clone();
                trans.rotate_axis(
                    axis,
                    DEFAULT_ANGULAR_SPEED.to_radians().clone() * time.delta_secs(),
                );
            }

            if keys.pressed(KeyCode::ArrowLeft) {
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

            if keys.pressed(KeyCode::ArrowRight) {
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
