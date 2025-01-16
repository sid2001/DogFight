use bevy::prelude::*;
#[derive(Resource, Debug, Default)]
pub struct SceneAssets {
    pub spaceship: Handle<Scene>,
    pub asteroid: Handle<Scene>,
    pub missiles: Handle<Scene>,
}

#[derive(Resource, Debug, Default)]
pub struct AudioAssets {
    pub throttle_up: Handle<AudioSource>,
    pub engine_humming: Handle<AudioSource>,
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
        spaceship: asset_server.load("Spaceship.glb#Scene0"),
        missiles: asset_server.load("Bullet.glb#Scene0"),
    }
}

fn load_audio_assets(mut audio_assets: ResMut<AudioAssets>, asset_server: Res<AssetServer>) {
    *audio_assets = AudioAssets {
        throttle_up: asset_server.load("sounds/thrusters.ogg"),
        engine_humming: asset_server.load("sounds/ambient-spacecraft-hum-33119.ogg"),
    }
}
