use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SetupSet {
    InGame(InGameSet),
    Menu(MenuSet),
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpdateSet {
    InGame(InGameSet),
    Menu(MenuSet),
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputSet {
    InGame(Controls),
    Menu,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Controls {
    InGame(InGameSet),
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InGameSet {
    SpaceShip,
    Camera,
    Obstacle,
    Environment,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MenuSet;
