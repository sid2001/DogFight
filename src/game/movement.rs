use bevy::prelude::*;
#[derive(Component, Default)]
pub struct Velocity(pub Vec3);

#[derive(Component, Default)]
pub struct Thrust(pub Vec3);

#[derive(Component)]
pub struct Drag(pub Vec3);

#[derive(Component, Default)]
pub struct Inertia {
    pub velocity: Velocity,
    pub thrust: f32,
    pub angular_velocity: f32,
}

#[derive(Component, Default)]
pub struct Direction(pub Vec3, pub Vec3);

#[derive(Component)]
pub struct Position(pub Vec3);
