use bevy::prelude::*;

#[derive(Component)]
pub struct BotMarker;

#[derive(Component)]
pub struct BotTargetMarker;

#[derive(Component)]
pub enum BotState {
    Ideal,
    Chasing,
    Dead,
    Searching,
    Evading,
}

#[derive]

pub struct BotPlugin;
impl Plugin for BotPlugin {
    fn build(&self, app: &mut App) {}
}
