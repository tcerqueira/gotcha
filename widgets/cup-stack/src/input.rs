use bevy::{input::InputSystem, prelude::*};
use gotcha_plugin::GotchaState;

pub struct ThrowInputPlugin;

impl Plugin for ThrowInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ThrowAction>();
        app.init_resource::<DragState>();
        app.add_systems(
            PreUpdate,
            (throw_input_system)
                .run_if(in_state(GotchaState::Gameplay))
                .after(InputSystem),
        );
    }
}

#[derive(Event)]
pub enum ThrowAction {
    Holding(ThrowParams),
    Throw(ThrowParams),
}

#[derive(Debug, Clone, Copy)]
pub struct ThrowParams {
    pub impulse: f32,
    pub dir: Dir2,
}

pub const IMPULSE_MAGNITUDE: f32 = 0.03;

#[derive(Resource, Default)]
struct DragState {
    start_position: Option<Vec2>,
    current_position: Option<Vec2>,
}

fn throw_input_system(
    mut drag_state: ResMut<DragState>,
    mut throw_events: EventWriter<ThrowAction>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    // Get current mouse position
    let current_mouse_position = if let Some(position) = window.cursor_position() {
        position
    } else {
        return;
    };

    // Start dragging
    if mouse_button_input.just_pressed(MouseButton::Left) {
        drag_state.start_position = Some(current_mouse_position);
        drag_state.current_position = Some(current_mouse_position);
        return;
    }

    // Update drag
    if mouse_button_input.pressed(MouseButton::Left) {
        if let Some(start_pos) = drag_state.start_position {
            drag_state.current_position = Some(current_mouse_position);
            // Calculate drag vector
            let drag_vector = start_pos - current_mouse_position;
            let throw_params = compute_throw_params(drag_vector);
            // Send holding event for trajectory prediction
            throw_events.send(ThrowAction::Holding(throw_params));
        }
    }

    // Release throw
    if mouse_button_input.just_released(MouseButton::Left) {
        if let (Some(start_pos), Some(end_pos)) =
            (drag_state.start_position, drag_state.current_position)
        {
            // Calculate final throw parameters
            let drag_vector = start_pos - end_pos;
            let throw_params = compute_throw_params(drag_vector);
            // Send throw event
            throw_events.send(ThrowAction::Throw(throw_params));
        }
        // Reset drag state
        drag_state.start_position = None;
        drag_state.current_position = None;
    }
}

// Constants for throw mechanics
const MAX_DRAG_DISTANCE: f32 = 200.0; // Maximum drag distance for full power
const MAX_IMPULSE: f32 = 0.05; // Maximum throw force
const MIN_IMPULSE: f32 = 0.02; // Minimum throw force

fn compute_throw_params(drag_vec: Vec2) -> ThrowParams {
    // Calculate drag distance (clamped to MAX_DRAG_DISTANCE)
    let drag_distance = drag_vec.length().min(MAX_DRAG_DISTANCE);
    // Calculate impulse (power) based on drag distance
    let impulse = (drag_distance / MAX_DRAG_DISTANCE) * (MAX_IMPULSE - MIN_IMPULSE) + MIN_IMPULSE;
    // Calculate angle (in radians)
    let dir = Dir2::new(-drag_vec).unwrap_or(Dir2::Y);
    debug!("{:?}", dir);
    ThrowParams { impulse, dir }
}
