use crate::asset_loader::SceneAssets;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct SwarmTarget;

#[derive(Component)]
pub struct SwarmPointMarker;

#[derive(Component)]
pub struct SwarmBotMarker;

#[derive(Component)]
pub struct SwarmPoint {
    origin: Vec3,
    burst_count: u32,
    cooldown: Duration,
    last_burst: Duration,
    limit: u32,
    live_bots: u32,
}

#[derive(Component)]
#[require(SceneRoot)]
pub struct SwarmBot {
    dir: Dir3,
    velocity: Vec3,
    thrust: f32,
    angular_velocity: f32,
}

impl Default for SwarmPoint {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            burst_count: 10,
            cooldown: Duration::new(10, 0),
            last_burst: Duration::new(0, 0),
            limit: 20,
            live_bots: 0,
        }
    }
}

impl SwarmPoint {
    pub fn xyz(&self) -> Vec3 {
        self.origin.clone()
    }
}

pub struct SwarmPlugin;
impl Plugin for SwarmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    let swarm = SwarmPoint::default();
    let origin = swarm.xyz();
    let transform =
        Transform::from_xyz(origin.x, origin.y, origin.z).with_scale(Vec3::new(0.5, 0.5, 0.5));
    let scene = SceneRoot(scene_assets.swarm_point.clone());
    commands.spawn((swarm, transform, scene, SwarmPointMarker));
    commands.spawn((
        SceneRoot(scene_assets.map_marker.clone()),
        Transform::from_xyz(1., 1., 1.).with_scale(Vec3::new(0.05, 0.05, 0.05)),
    ));
}

fn release_bots(
    mut query_swarm_point: Query<(&Transform, &mut SwarmPoint), With<SwarmPointMarker>>,
    scene_assets: Res<SceneAssets>,
    time: Res<Time>,
) {
    let time_delta = time.delta();
    for (trans, swarm_point) in query_swarm_point.iter_mut() {
        let mut count = swarm_point.limit - swarm_point.live_bots;
        count = if count > swarm_point.burst_count {
            swarm_point.burst_count
        } else {
            count
        };

        // for
    }
}
