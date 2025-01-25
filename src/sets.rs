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
    InGame(ControlsSet),
    Menu,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ControlsSet {
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
