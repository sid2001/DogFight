use crate::asset_loader::SceneAssets;
use crate::camera::setup_camera;
use crate::movement::{Direction, Drag, Inertia, Position, Velocity};
use bevy::audio::PlaybackMode::*;
use bevy::prelude::*;

const DEFAULT_HEALTH: f32 = 100.0;
const DEFAULT_THRUST: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const DEFAULT_SPAWN: Vec3 = Vec3::ZERO;
const DEFAULT_ANGULAR_CHANGE: f32 = 30.0;
const DEFAULT_DIRECTION: (Vec3, Vec3) = (Vec3::Y, Vec3::X);
const DEFAULT_DRAG: Vec3 = Vec3::new(0.1, 0.1, 0.1);
const DEFAULT_SPEED_LIMIT: f32 = 1.5;
// const STARTING_TRANSLATION: Vec3 = Vec3::new(0.0, 20.0, 0.0);

#[derive(Resource, Default)]
pub struct Entities {
    pub player: Option<Entity>,
    pub camera: Option<Entity>,
}

#[derive(Component)]
pub struct SpaceShip;

#[derive(Component)]
pub struct Health(f32);

#[derive(Bundle)]
pub struct SpaceShipBundle {
    pub health: Health,
    pub marker: SpaceShip,
    pub position: Position,
    pub inertia: Inertia,
    pub direction: Direction,
    pub model: SceneBundle,
    pub audio: AudioBundle,
    pub drag: Drag,
}
pub struct SpaceShipPlugin;

impl Plugin for SpaceShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_spaceship.before(setup_camera))
            .add_systems(
                Update,
                (
                    spaceship_controls,
                    accelerate_spaceship,
                    move_spaceship,
                    spaceship_orientation,
                )
                    .chain(),
            );
    }
}

fn spaceship_controls(
    keys: Res<Input<KeyCode>>,
    mut spaceship_query: Query<(&mut Inertia, &mut Direction), With<SpaceShip>>,
    time: Res<Time>,
    entity: Res<Entities>,
) {
    let (ref mut inertia, ref mut dir) = spaceship_query
        .get_mut(entity.player.unwrap())
        .expect("Can't get entity!");
    if keys.just_pressed(KeyCode::J) {
        if inertia.thrust != 6.0 {
            inertia.thrust += 2.0;
        }
    }
    if keys.just_pressed(KeyCode::K) {
        if inertia.thrust != -6.0 {
            inertia.thrust -= 2.0;
        }
    }

    if keys.pressed(KeyCode::W) {
        // let target = dir.0.cross(dir.1);
        let rotation = Quat::from_axis_angle(
            dir.1,
            DEFAULT_ANGULAR_CHANGE.to_radians() * time.delta_seconds(),
        );
        dir.0 = rotation.mul_vec3(dir.0);
    }

    if keys.pressed(KeyCode::S) {
        let rotation = Quat::from_axis_angle(
            -dir.1,
            DEFAULT_ANGULAR_CHANGE.to_radians() * time.delta_seconds(),
        );
        dir.0 = rotation.mul_vec3(dir.0);
    }
    if keys.pressed(KeyCode::A) {
        let rotation = Quat::from_axis_angle(
            -dir.0,
            DEFAULT_ANGULAR_CHANGE.to_radians() * time.delta_seconds(),
        );
        dir.1 = rotation.mul_vec3(dir.1);
    }

    if keys.pressed(KeyCode::D) {
        let rotation = Quat::from_axis_angle(
            dir.0,
            DEFAULT_ANGULAR_CHANGE.to_radians() * time.delta_seconds(),
        );
        dir.1 = rotation.mul_vec3(dir.1);
    }

    if keys.pressed(KeyCode::Space) {
        let Vec3 { x, y, z } = inertia.velocity.0;
        inertia.velocity.0.x = if x.abs() < 0.1 {
            0.0
        } else {
            x / (1.0 + 1. * time.delta_seconds())
        };
        inertia.velocity.0.y = if y.abs() < 0.1 {
            0.0
        } else {
            y / (1.0 + 1. * time.delta_seconds())
        };
        inertia.velocity.0.z = if z.abs() < 0.1 {
            0.0
        } else {
            z / (1.0 + 1. * time.delta_seconds())
        };
    }
}

fn accelerate_spaceship(
    mut spaceship_query: Query<(&mut Inertia, &Drag, &Direction), With<SpaceShip>>,
    time: Res<Time>,
    entity: Res<Entities>,
) {
    let (mut inertia, drag, dir) = spaceship_query
        .get_mut(entity.player.unwrap())
        .expect("Can't get entitiy!");

    let acc = dir.0.normalize_or_zero() * inertia.thrust;
    info!("acc: {:?}", acc);
    info!("vel: {:?}", inertia.velocity.0);

    let Vec3 { x, y, z } = inertia.velocity.0.clone();
    inertia.velocity.0.y = if y.abs() <= drag.0.y {
        0.0
    } else {
        (y / y.abs()) * (y.abs() - drag.0.y * time.delta_seconds())
    };
    inertia.velocity.0.x = if x.abs() <= drag.0.x {
        0.0
    } else {
        (x / x.abs()) * (x.abs() - drag.0.x * time.delta_seconds())
    };
    inertia.velocity.0.z = if z.abs() <= drag.0.z {
        0.0
    } else {
        (z / z.abs()) * (z.abs() - drag.0.z * time.delta_seconds())
    };
    let next_velocity = inertia.velocity.0 + acc * time.delta_seconds();
    inertia.velocity.0 = if next_velocity.length() > DEFAULT_SPEED_LIMIT {
        next_velocity.normalize() * DEFAULT_SPEED_LIMIT
    } else {
        next_velocity
    };
}

fn move_spaceship(
    mut query: Query<(&mut Transform, &Inertia), With<SpaceShip>>,
    entity: Res<Entities>,
    time: Res<Time>,
) {
    let (mut trans, iner) = query
        .get_mut(entity.player.unwrap())
        .expect("Error getting entity player!");

    trans.translation += iner.velocity.0 * time.delta_seconds();
}

fn spaceship_orientation(
    mut query: Query<(&mut Transform, &Direction, &Inertia), With<SpaceShip>>,
    entities: Res<Entities>,
    // time: Res<Time>,
) {
    let (ref mut trans, dir, iner) = query
        .get_mut(entities.player.unwrap())
        .expect("Cannot get player entity!");

    // let mut curr_dir = Vec3::ZERO;
    let curr_dir = trans.forward();
    let target_dir = dir.0.clone();

    let roll_curr = trans.right();
    let roll_target = dir.1.clone();
    // let target_dir = match iner.velocity.0.length() {
    //     0.0 => curr_dir.normalize_or_zero(),
    //     _ => dir.0.normalize_or_zero(),
    // };
    // let mut target_dir = if iner.velocity.0.length() != 0.0 {
    //     curr_dir += trans.up() + trans.forward() + trans.right();
    //     curr_dir = curr_dir.normalize_or_zero();
    //     -iner.velocity.0.clone().normalize()
    // } else {
    //     trans.forward()
    // };
    // let res_vector = curr_dir + target_dir;
    // target_dir = curr_dir + res_vector.normalize_or_zero() * time.delta_seconds();
    // target_dir.normalize_or_zero();
    info!("{:?}", curr_dir);
    info!("{:?}", target_dir);
    // let rotation = Quat::from_axis_angle(
    //     curr_dir.cross(target_dir).normalize_or_zero(),
    //     DEFAULT_ANGULAR_CHANGE.to_radians() * time.delta_seconds(),
    // );
    let rotation =
        Quat::from_rotation_arc(curr_dir.normalize_or_zero(), target_dir.normalize_or_zero());

    let roll = Quat::from_rotation_arc(
        roll_curr.normalize_or_zero(),
        roll_target.normalize_or_zero(),
    );
    trans.rotate(rotation);
    trans.rotate(roll);
}

fn spawn_spaceship(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    asset_server: Res<AssetServer>,
    mut entities: ResMut<Entities>,
) {
    if let Some(spaceship_scene) = scene_assets.spaceship.clone().into() {
        info!("spawning spacehip");
        entities.player = Some(
            commands
                .spawn((SpaceShipBundle {
                    health: Health(DEFAULT_HEALTH),
                    marker: SpaceShip,
                    position: Position(DEFAULT_SPAWN.clone()),
                    drag: Drag(DEFAULT_DRAG.clone()),
                    inertia: Inertia {
                        thrust: 0.,
                        ..default()
                    },
                    direction: Direction(DEFAULT_DIRECTION.0.clone(), DEFAULT_DIRECTION.1.clone()),
                    model: SceneBundle {
                        scene: spaceship_scene,
                        transform: Transform::from_translation(Vec3::new(0., 10., 0.))
                            // .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
                            .with_translation(Vec3::new(0.0, 0., 0.0))
                            .looking_at(Vec3::Y, Vec3::Z),
                        ..default()
                    },
                    audio: AudioBundle {
                        source: asset_server.load("sounds/ambient-spacecraft-hum-33119.ogg"),
                        settings: PlaybackSettings {
                            mode: Loop,
                            paused: false,
                            ..default()
                        },
                    },
                },))
                .id(),
        );
    } else {
        info!("Asset not loaded!")
    }
}
