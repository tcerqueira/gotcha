use std::f32::consts::PI;

use bevy::{input::InputSystem, prelude::*};
use gotcha_plugin::GotchaState;

pub struct ThrowInputPlugin;

impl Plugin for ThrowInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ThrowAction>();
        app.add_systems(
            PreUpdate,
            map_gameplay_input
                .run_if(in_state(GotchaState::Gameplay))
                .after(InputSystem),
        );
    }
}

#[expect(dead_code)]
#[derive(Event)]
pub enum ThrowAction {
    Holding { impulse: f32, angle: f32 },
    Throw { impulse: f32, angle: f32 },
}

pub const IMPULSE_MAGNITUDE: f32 = 0.03;

fn map_gameplay_input(input: Res<ButtonInput<KeyCode>>, mut event_w: EventWriter<ThrowAction>) {
    if input.just_pressed(KeyCode::KeyF) {
        event_w.send(ThrowAction::Throw { impulse: IMPULSE_MAGNITUDE, angle: PI / 6. });
    }
}
