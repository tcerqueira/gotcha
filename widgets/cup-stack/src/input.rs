use std::f32::consts::PI;

use bevy::{input::InputSystem, prelude::*};

use crate::game::AppState;

pub struct ThrowInputPlugin;

impl Plugin for ThrowInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ThrowAction>();
        app.add_systems(
            PreUpdate,
            (
                map_gameplay_input.run_if(in_state(AppState::Gameplay)),
                map_welcome_input.run_if(in_state(AppState::Welcome)),
            )
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

pub const IMPULSE_MAGNITUDE: f32 = 0.05;

fn map_gameplay_input(input: Res<ButtonInput<KeyCode>>, mut event_w: EventWriter<ThrowAction>) {
    if input.just_pressed(KeyCode::KeyF) {
        event_w.send(ThrowAction::Throw { impulse: IMPULSE_MAGNITUDE, angle: PI / 6. });
    }
}

fn map_welcome_input(
    input: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if input.just_pressed(MouseButton::Left) {
        next_state.set(AppState::Gameplay);
    }
}
