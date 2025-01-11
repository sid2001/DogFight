use bevy::prelude::*;
use bevy::render::color::Color::*;
use std::f32::consts::PI;
use crate::spaceship::SpaceShip;

#[derive(Component)]
struct MyCameraMarker;

#[derive(Bundle)]
pub struct MyCameraBundle {
    marker: MyCameraMarker,
    camera: Camera3dBundle
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        .add_systems(Update, follow_spaceship);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        SpotLight {
            color: Rgba {
                red: 0.255,
                blue: 0.255,
                green: 0.186,
                alpha: 0.5,
            },
            intensity: 40_000.0, // lumens
            inner_angle: PI / 4.0 * 0.85,
            outer_angle: PI / 4.0,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::Y, Vec3::Y),
    ));
    commands.spawn(
        MyCameraBundle {
            camera: Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 80.0).looking_at(Vec3::NEG_Z, Vec3::Y),
            ..default()
        },
        marker: MyCameraMarker,
        }
    );
}

fn follow_spaceship(
    mut spaceship_query: Query<&Transform, With<SpaceShip>>,
    mut camera_query: Query<&Transform, With<MyCameraMarker>>
) {

}
