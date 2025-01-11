use bevy::prelude::*;
#[derive(Component, Default)]
pub struct Velocity(pub Vec3);

#[derive(Component, Default)]
pub struct Thrust(pub Vec3);

#[derive(Component, Default)]
pub struct Inertia {
    pub velocity: Velocity,
    pub thrust: Thrust,
}

#[derive(Component)]
pub struct Position(pub Vec3);
