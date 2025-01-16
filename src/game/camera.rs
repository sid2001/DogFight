use super::movement::{Direction, Inertia};
use super::spaceship::{Entities, SpaceShip};
use bevy::{pbr::*, prelude::*};

#[derive(Component)]
pub struct MyCameraMarker;

#[derive(Bundle)]
pub struct MyCameraBundle {
    marker: MyCameraMarker,
    camera: Camera3d,
    transform: Transform,
    projection: Projection,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, follow_spaceship);
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
            .spawn(MyCameraBundle {
                camera: Camera3d { ..default() },
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: 70.0_f32.to_radians(),
                    ..Default::default()
                }),
                transform: Transform::from_xyz(0.0, -10.0, 0.0).looking_at(Vec3::Y, Vec3::Z),
                marker: MyCameraMarker,
            })
            .id(),
    );
}

fn follow_spaceship(
    mut cam_query: Query<&mut Transform, With<MyCameraMarker>>,
    sp_query: Query<
        (&Transform, &mut Direction, &Inertia),
        (With<SpaceShip>, Without<MyCameraMarker>),
    >,
    entity: Res<Entities>,
    time: Res<Time>,
) {
    let (trans, sp_dir, _iner) = sp_query
        .get(entity.player.unwrap())
        .expect("Error while player!");
    let v = trans.translation.clone();

    let mut camera = cam_query
        .get_mut(entity.camera.unwrap())
        .expect("Can't get entitiy camera");

    //* find a way to calcuate a factor so that the camera speed changes with spacecraft velocity
    // let factor = iner.velocity.0.clone()
    //     - Vec3::new(1., 1., 1.) * iner.velocity.0.clone().normalize_or_zero();

    let cam = camera.translation.clone();
    camera.translation += (v - sp_dir.0.normalize().clone() - cam) * time.delta_secs() * 5.;

    let rotation = Quat::from_rotation_arc(camera.forward().normalize_or_zero(), sp_dir.0);

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
    camera.rotate(rotation);
}
