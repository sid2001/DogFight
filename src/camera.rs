use crate::movement::Direction;
use crate::spaceship::{Entities, SpaceShip};
use bevy::{pbr::*, prelude::*, render::color::Color::*};
use std::f32::consts::PI;
#[derive(Component)]
pub struct MyCameraMarker;

#[derive(Bundle)]
pub struct MyCameraBundle {
    marker: MyCameraMarker,
    camera: Camera3dBundle,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(PostUpdate, follow_spaceship);
    }
}

pub fn setup_camera(mut commands: Commands, mut entities: ResMut<Entities>) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(1.0, 1.0, 0.9), // Slight yellowish tint for sunlight
            illuminance: 100000.0,            // Brightness of the sunlight
            shadows_enabled: true,            // Enable shadows
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)), // 45° angle
        ..default()
    });
    entities.camera = Some(
        commands
            .spawn(MyCameraBundle {
                camera: Camera3dBundle {
                    projection: PerspectiveProjection {
                        fov: 60.0_f32.to_radians(),
                        ..default()
                    }
                    .into(),
                    transform: Transform::from_xyz(0.0, -10.0, 0.0).looking_at(Vec3::Y, Vec3::Z),
                    ..default()
                },
                marker: MyCameraMarker,
            })
            .id(),
    );
}

fn follow_spaceship(
    mut cam_query: Query<&mut Transform, With<MyCameraMarker>>,
    entity: Res<Entities>,
    mut sp_query: Query<(&Transform, &mut Direction), (With<SpaceShip>, Without<MyCameraMarker>)>,
) {
    let (trans, sp_dir) = sp_query
        .get(entity.player.unwrap())
        .expect("Error while player!");
    let v = trans.translation.clone();
    // let sp = sp_queryy.get(entity.player.unwrap()).expect("Errorsf");
    let mut camera = cam_query
        .get_mut(entity.camera.unwrap())
        .expect("Can't get entitiy camera");
    // let unit_v = v.normalize();

    camera.translation = v - sp_dir.0.normalize().clone() * 1.5;
    // camera.rotation = sp.0.clone().normalize();
    let rotation = Quat::from_rotation_arc(
        camera.forward().normalize_or_zero(),
        sp_dir.0,
        // ((sp.0.clone().normalize() - camera.forward().clone()).normalize() / 2.
        //     + camera.forward().clone())
        // .normalize(),
    );
    camera.rotate(rotation);
    // camera.rotate_around(v, rotation);
    // camera.as_deref_mut() = camera.looking_to(sp.0.clone(), sp.1.cross(sp.0).normalize());
}
