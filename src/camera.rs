use crate::spaceship::{Entities, SpaceShip};
use bevy::{pbr::*, prelude::*, render::color::Color::*};
use std::f32::consts::PI;
#[derive(Component)]
struct MyCameraMarker;

#[derive(Bundle)]
pub struct MyCameraBundle {
    marker: MyCameraMarker,
    camera: Camera3dBundle,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        // .add_systems(Update, follow_spaceship);
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
                    transform: Transform::from_xyz(0.0, -2.0, 0.0).looking_at(Vec3::Y, Vec3::Z),
                    ..default()
                },
                marker: MyCameraMarker,
            })
            .id(),
    );
}

fn follow_spaceship(
    mut query: Query<&mut Transform>,
    // mut camera_query: Query<&mut Transform, With<MyCameraMarker>>,
    entity: Res<Entities>,
) {
    let v = {
        let spaceship = query
            .get(entity.player.unwrap())
            .expect("Erorr getting player");
        let &Vec3 { x, y, z } = spaceship.get_field("translation").unwrap();
        Vec3::new(x, y, z)
    };
    let mut camera = query
        .get_mut(entity.camera.unwrap())
        .expect("Can't get entitiy camera");
    // let unit_v = v.normalize();
    camera.look_at(v, Vec3::Y);
}
