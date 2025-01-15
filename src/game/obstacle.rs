use crate::asset_loader::SceneAssets;
use bevy::prelude::*;

#[derive(Component)]
pub enum ObstacleMarker {
    Obstacle(u32),
    None,
}

#[derive(Component)]
pub struct Obstacle {
    visibility: Visibility,
    // scene: SceneBundle,
}

#[derive(Bundle)]
pub struct ObstacleBundle {
    pub obstacle: Obstacle,
    pub marker: ObstacleMarker,
    scene: SceneBundle, // pub visibility: Visibility,
}

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_obstacles);
    }
}

fn spawn_obstacles(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    // let scene = scene_assets.asteroid.clone();

    commands.spawn((
        ObstacleBundle {
            obstacle: Obstacle {
                visibility: Visibility::Visible,
                // scene: SceneBundle {
                //     visibility: Visibility::Visible,
                //     scene: scene_assets.asteroid.clone(),
                //     transform: Transform::from_xyz(1., 1., 1.).with_scale(Vec3::new(2., 2., 2.)),
                //     ..default()
                // },
            },
            marker: ObstacleMarker::Obstacle(1),
            scene: SceneBundle {
                // visibility: Visibility::Visible,
                scene: scene_assets.asteroid.clone(),
                transform: Transform::from_xyz(1., 5., 5.).with_scale(Vec3::new(1., 1., 1.)),
                ..default()
            },
        },
        // SceneBundle {
        //     // visibility: Visibility::Visible,
        //     scene: scene_assets.asteroid.clone(),
        //     transform: Transform::from_xyz(1., 5., 5.).with_scale(Vec3::new(2., 2., 2.)),
        //     ..default()
        // },
    ));

    commands.spawn((
        ObstacleBundle {
            // visibility: Visibility::Visible,
            // scale: Vec3::new(2., 2., 2.),
            obstacle: Obstacle {
                visibility: Visibility::Visible,
                // scene: SceneBundle {
                //     visibility: Visibility::Visible,
                //     scene: scene_assets.asteroid.clone(),
                //     transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::new(1., 1., 1.)),
                //     ..default()
                // },
            },
            marker: ObstacleMarker::Obstacle(2),
            scene: SceneBundle {
                // visibility: Visibility::Visible,
                scene: scene_assets.asteroid.clone(),
                transform: Transform::from_xyz(1., 1., -5.).with_scale(Vec3::new(1., 1., 1.)),
                ..default()
            },
        },
        // SceneBundle {
        //     // visibility: Visibility::Visible,
        //     // scene: scene_assets.asteroid.clone(),
        //     // transform: Transform::from_xyz(1., 1., -5.).with_scale(Vec3::new(1., 1., 1.)),
        //     ..default()
        // },
    ));
}
