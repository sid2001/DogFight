use bevy::prelude::*;

#[derive(Resource)]
pub struct Controls {
    pub thrust: Option<KeyCode>,
    pub back_thrust: Option<KeyCode>,
    pub brake: Option<KeyCode>,
    pub up: Option<KeyCode>,
    pub down: Option<KeyCode>,
    pub roll_l: Option<KeyCode>,
    pub roll_r: Option<KeyCode>,
    pub shoot: Option<KeyCode>,
    pub toggle_freelook: Option<KeyCode>,
    pub camera_up: Option<KeyCode>,
    pub camera_down: Option<KeyCode>,
    pub camera_l: Option<KeyCode>,
    pub camera_r: Option<KeyCode>,
    pub align_camera: Option<KeyCode>,
    pub steer_boost: Option<KeyCode>,
    pub camera_view: Option<KeyCode>,
    pub toggle_rear_view: Option<KeyCode>,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            thrust: Some(KeyCode::KeyJ),
            back_thrust: Some(KeyCode::KeyK),
            brake: Some(KeyCode::Space),
            up: Some(KeyCode::KeyW),
            down: Some(KeyCode::KeyS),
            roll_l: Some(KeyCode::KeyA),
            roll_r: Some(KeyCode::KeyD),
            shoot: Some(KeyCode::KeyL),
            toggle_freelook: Some(KeyCode::ControlLeft),
            camera_up: Some(KeyCode::ArrowUp),
            camera_down: Some(KeyCode::ArrowDown),
            camera_l: Some(KeyCode::ArrowLeft),
            camera_r: Some(KeyCode::ArrowRight),
            align_camera: Some(KeyCode::AltRight),
            steer_boost: Some(KeyCode::ShiftLeft),
            camera_view: Some(KeyCode::KeyV),
            toggle_rear_view: Some(KeyCode::KeyC),
        }
    }
}

// pub enum ControlState {
//     Thrust,
//     BackThrust,
//     Brake,
//     Up,
//     Down,
//     RollL,
//     RollR,
//     Shoot,
//     ToggleFreelook,
//     CameraUp,
//     CameraDown,
//     CameraL,
//     CameraR,
//     AlignCamera
// }

pub struct ControlPlugin;
impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Controls { ..default() });
    }
}
