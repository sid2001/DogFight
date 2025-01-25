use crate::game::spaceship::SpaceShip;
// use crate::game::turret::TurretMarker;
use crate::game::turret::*;
use crate::sets::*;
use bevy::prelude::*;

#[derive(Event)]
pub struct ThrottleUpEvent(pub Entity);

#[derive(Event)]
pub struct ThrottleDownEvent(pub Entity);

pub struct EventPlugin;
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ThrottleUpEvent>()
            .add_event::<ThrottleDownEvent>()
            .add_systems(Update, (throttle_sound_on, throttle_sound_off));
    }
}

pub struct TurretEventPlugin;
impl Plugin for TurretEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShootTurretEventOn>()
            .add_event::<ShootTurretEventOff>()
            .add_systems(
                Update,
                (turret_sound_on, turret_sound_off)
                    .after(InputSet::InGame(ControlsSet::InGame(InGameSet::SpaceShip))),
            );
    }
}

fn throttle_sound_on(
    mut ev_throttle_up: EventReader<ThrottleUpEvent>,
    mut query: Query<&AudioSink, With<SpaceShip>>,
) {
    for entity in ev_throttle_up.read() {
        if let Ok(bun) = query.get_mut(entity.0) {
            // error!("up recv");
            bun.set_volume(bun.volume() + 0.05);
            bun.play();
        } else {
            error!("Entity not present");
        }
    }
}

fn throttle_sound_off(
    mut ev_throttle_off: EventReader<ThrottleDownEvent>,
    mut query: Query<&AudioSink, With<SpaceShip>>,
) {
    for entity in ev_throttle_off.read() {
        if let Ok(ref mut bun) = query.get_mut(entity.0) {
            // error!("up dv");
            if bun.volume() != 0. {
                bun.set_volume(bun.volume() - 0.05);
            } else {
                bun.pause();
            }
        } else {
            error!("Entity not present");
        }
    }
}
