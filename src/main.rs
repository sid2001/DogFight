mod asset_loader;
mod events;
mod game;
mod sets;
mod states;

use asset_loader::AssetLoaderPlugin;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use events::EventPlugin;
use game::spaceship::Entities;
use game::GamePlugin;
use states::StatePlugin;

use bevy::{
    color::palettes::css::*,
    pbr::wireframe::{NoWireframe, Wireframe, WireframeColor, WireframeConfig, WireframePlugin},
    render::{
        render_resource::WgpuFeatures,
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugins((
        //     DefaultPlugins.set(RenderPlugin {
        //         render_creation: RenderCreation::Automatic(WgpuSettings {
        //             // WARN this is a native only feature. It will not work with webgl or webgpu
        //             features: WgpuFeatures::POLYGON_MODE_LINE,
        //             ..default()
        //         }),
        //         ..default()
        //     }),
        //     // You need to add this plugin to enable wireframe rendering
        //     WireframePlugin,
        // ))
        .add_plugins(AssetLoaderPlugin)
        .init_resource::<Entities>()
        .add_plugins(EventPlugin)
        .add_plugins(StatePlugin)
        .add_plugins(GamePlugin)
        // .add_plugins(PanOrbitCameraPlugin)
        // .add_systems(Startup, setup)
        // Wireframes can be configured with this resource. This can be changed at runtime.
        // .insert_resource(WireframeConfig {
        //     // The global wireframe config enables drawing of wireframes on every mesh,
        //     // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
        //     // regardless of the global configuration.
        //     global: true,
        //     // Controls the default color of all wireframes. Used as the default color for global wireframes.
        //     // Can be changed per mesh using the `WireframeColor` component.
        //     default_color: WHITE.into(),
        // })
        // .add_plugins(MenuPlugin)
        .run();
}

#[derive(Bundle)]
pub struct PanOrbitBundle {
    transform: Transform,
    camera: PanOrbitCamera,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(PanOrbitBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        camera: PanOrbitCamera::default(),
    });
}
