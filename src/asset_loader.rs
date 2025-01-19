use bevy::prelude::*;
#[derive(Resource, Debug, Default)]
pub struct SceneAssets {
    pub spaceship: Handle<Scene>,
    pub asteroid: Handle<Scene>,
    pub missiles: Handle<Scene>,
    pub player_turret: Handle<Scene>,
    pub enemy_turret: Handle<Scene>,
    pub bot_spaceship: Handle<Scene>,
    pub bot_spaceship2: Handle<Scene>,
    pub bot_spaceship3: Handle<Scene>,
    pub map_marker: Handle<Scene>,
}

#[derive(Resource, Debug, Default)]
pub struct AudioAssets {
    pub throttle_up: Handle<AudioSource>,
    pub engine_humming: Handle<AudioSource>,
    pub laser_turret: Handle<AudioSource>,
}

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneAssets>()
            .init_resource::<AudioAssets>()
            .add_systems(PreStartup, load_scene_assets)
            .add_systems(PreStartup, load_audio_assets);
    }
}

fn load_scene_assets(mut scene_assets: ResMut<SceneAssets>, asset_server: Res<AssetServer>) {
    *scene_assets = SceneAssets {
        asteroid: asset_server.load("Planet.glb#Scene0"),
        spaceship: asset_server.load("Spaceship2.glb#Scene0"),
        missiles: asset_server.load("Bullet.glb#Scene0"),
        player_turret: asset_server.load("lazer_bullet.glb#Scene0"),
        enemy_turret: asset_server.load("lazer_bullet2.glb#Scene0"),
        bot_spaceship: asset_server.load("SpaceshipBot.glb#Scene0"),
        map_marker: asset_server.load("map_marker.glb#Scene0"),
        bot_spaceship2: asset_server.load("Spaceship3.glb#Scene0"),
        bot_spaceship3: asset_server.load("Spaceship4.glb#Scene0"),
    }
}

fn load_audio_assets(mut audio_assets: ResMut<AudioAssets>, asset_server: Res<AssetServer>) {
    *audio_assets = AudioAssets {
        throttle_up: asset_server.load("sounds/thrusters.ogg"),
        engine_humming: asset_server.load("sounds/ambient-spacecraft-hum-33119.ogg"),
        laser_turret: asset_server.load("sounds/laserturret.ogg"),
    }
}
