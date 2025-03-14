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
    pub planet1: Handle<Scene>,
    pub terrain: Handle<Scene>,
    pub mes: Handle<Mesh>,
    pub swarm_point: Handle<Scene>,
    pub missile: Handle<Scene>,
    pub missile2: Handle<Scene>,
}

#[derive(Resource, Debug, Default)]
pub struct AudioAssets {
    pub throttle_up: Handle<AudioSource>,
    pub engine_humming: Handle<AudioSource>,
    pub laser_turret: Handle<AudioSource>,
    pub homing_launch: Handle<AudioSource>,
    pub homing_cruise: Handle<AudioSource>,
}

#[derive(Resource, Debug, Default)]
pub struct MenuAssets {
    pub start: Handle<Image>,
    pub settings: Handle<Image>,
    pub exit: Handle<Image>,
    pub screen_background: Handle<Image>,
    pub menu_background: Handle<Image>,
    pub resume: Handle<Image>,
    // pub falling_star: Handle<Image>
}

#[derive(Resource, Debug, Default)]
pub struct MapOneAssets {
    pub sun: Handle<Scene>,
}

#[derive(Resource, Debug, Default)]
pub struct AssetsLoading(pub Vec<UntypedHandle>);

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneAssets>()
            .init_resource::<AudioAssets>()
            .init_resource::<MenuAssets>()
            .init_resource::<AssetsLoading>()
            .init_resource::<MapOneAssets>()
            .add_systems(PreStartup, load_scene_assets)
            .add_systems(PreStartup, load_audio_assets)
            .add_systems(PreStartup, load_menu_assets)
            .add_systems(PreStartup, load_map_one_assets);
    }
}

fn load_map_one_assets(mut map_assets: ResMut<MapOneAssets>, asset_server: Res<AssetServer>) {
    *map_assets = MapOneAssets {
        sun: asset_server.load("Sun3.gltf#Scene0"),
    };
}

fn load_scene_assets(mut scene_assets: ResMut<SceneAssets>, asset_server: Res<AssetServer>) {
    *scene_assets = SceneAssets {
        asteroid: asset_server.load("Planet.glb#Scene0"),
        spaceship: asset_server.load(format!("Spaceship2.gltf#Scene0")),
        missiles: asset_server.load("Bullet.glb#Scene0"),
        player_turret: asset_server.load("lazer_bullet.glb#Scene0"),
        enemy_turret: asset_server.load("lazer_bullet2.glb#Scene0"),
        bot_spaceship: asset_server.load("SpaceshipBot/SpaceshipBot.gltf#Scene0"),
        map_marker: asset_server.load("map_marker.glb#Scene0"),
        bot_spaceship2: asset_server.load("Spaceship3/Spaceship3.gltf#Scene0"),
        bot_spaceship3: asset_server.load("Spaceship4/Spaceship4.gltf#Scene0"),
        planet1: asset_server.load("Planet1_hollow.glb#Scene0"),
        terrain: asset_server.load("terrain/lowpolylandscape.glb#Scene0"),
        swarm_point: asset_server.load("Planet-18Uxrb2dIc.glb#Scene0"),
        mes: asset_server.load(GltfAssetLabel::Scene(0).from_asset("Spaceship.gltf")),
        missile: asset_server.load("missile.glb#Scene0"),
        missile2: asset_server.load("missile2.glb#Scene0"),
    };
}

fn load_audio_assets(mut audio_assets: ResMut<AudioAssets>, asset_server: Res<AssetServer>) {
    *audio_assets = AudioAssets {
        throttle_up: asset_server.load("sounds/thrusters.ogg"),
        engine_humming: asset_server.load("sounds/ambient-spacecraft-hum-33119.ogg"),
        laser_turret: asset_server.load("sounds/laserturret.ogg"),
        homing_cruise: asset_server.load("sounds/homing_cruise.ogg"),
        homing_launch: asset_server.load("sounds/homing_launch.ogg"),
    }
}

fn load_menu_assets(
    mut menu_assets: ResMut<MenuAssets>,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    let asset_dir = String::from("menu/Menu Buttons/Large Buttons/Large Buttons/");
    *menu_assets = MenuAssets {
        start: asset_server.load("menu/Menu Buttons/Large Buttons/Large Buttons/StartButton.png"),
        exit: asset_server.load(asset_dir.clone() + "ExitButton.png"),
        settings: asset_server.load(asset_dir.clone() + "SettingsButton.png"),
        screen_background: asset_server.load("menu/Stars_Background2.png"),
        menu_background: asset_server.load("menu/Stars_Background_Black.png"),
        resume: asset_server.load(asset_dir.clone() + "ResumeButton.png"),
    };

    loading.0.push(menu_assets.start.clone().untyped());
    loading.0.push(menu_assets.settings.clone().untyped());
    loading.0.push(menu_assets.exit.clone().untyped());
    loading
        .0
        .push(menu_assets.screen_background.clone().untyped());
    loading
        .0
        .push(menu_assets.menu_background.clone().untyped());
    loading.0.push(menu_assets.resume.clone().untyped());
}
