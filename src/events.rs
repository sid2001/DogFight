use crate::game::spaceship::SpaceShip;
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

fn throttle_sound_on(
    mut ev_throttle_up: EventReader<ThrottleUpEvent>,
    mut query: Query<&AudioSink, With<SpaceShip>>,
) {
    for entity in ev_throttle_up.read() {
        if let Ok(bun) = query.get_mut(entity.0) {
            error!("up recv");
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
            error!("up dv");
            bun.pause();
        } else {
            error!("Entity not present");
        }
    }
}
