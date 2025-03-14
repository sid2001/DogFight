mod asset_loader;
mod controls;
mod events;
mod game;
mod menu;
mod sets;
mod states;

use asset_loader::AssetLoaderPlugin;
use asset_loader::MenuAssets;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use bevy_dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use controls::ControlPlugin;
use events::EventPlugin;
use game::spaceship::Entities;
use game::GamePlugin;
use menu::MenuPlugin;
use states::StatePlugin;

use states::GameState;

use bevy::{
    color::palettes::css::*,
    pbr::wireframe::{NoWireframe, Wireframe, WireframeColor, WireframeConfig, WireframePlugin},
    render::{
        render_resource::WgpuFeatures,
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};

struct OverlayColor;

impl OverlayColor {
    const RED: Color = Color::srgb(1.0, 0.0, 0.0);
    const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}

fn main() {
    App::new()
        // .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    // WARN this is a native only feature. It will not work with webgl or webgpu
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            }),
            // You need to add this plugin to enable wireframe rendering
            // WireframePlugin,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 42.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                    },
                    // We can also change color of the overlay
                    text_color: OverlayColor::GREEN,
                    enabled: true,
                },
            },
        ))
        .init_resource::<Entities>()
        .init_state::<GameState>()
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(EventPlugin)
        .add_plugins(StatePlugin)
        .add_plugins((GamePlugin, MenuPlugin))
        .add_plugins(ControlPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        // .add_plugins(PanOrbitCameraPlugin)
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

fn add_game_plugin(app: &mut App) {
    app.add_plugins(GamePlugin);
}
