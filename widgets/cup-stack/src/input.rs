use bevy::input::touch::TouchPhase;
use bevy::{input::InputSystem, prelude::*, window::PrimaryWindow};
use gotcha_plugin::GotchaState;

pub struct ThrowInputPlugin;

impl Plugin for ThrowInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ThrowAction>();
        app.init_resource::<DragState>();
        app.add_systems(
            PreUpdate,
            (throw_input_system, touch_input_system)
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
    active_pointer: Option<PointerId>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PointerId {
    Mouse,
    Touch(u64),
}

fn throw_input_system(
    mut drag_state: ResMut<DragState>,
    mut throw_events: EventWriter<ThrowAction>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    // Skip if touch is active
    if matches!(drag_state.active_pointer, Some(PointerId::Touch(_))) {
        return;
    }

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
        drag_state.active_pointer = Some(PointerId::Mouse);
        return;
    }

    // Update drag
    if mouse_button_input.pressed(MouseButton::Left)
        && matches!(drag_state.active_pointer, Some(PointerId::Mouse))
    {
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
    if mouse_button_input.just_released(MouseButton::Left)
        && matches!(drag_state.active_pointer, Some(PointerId::Mouse))
    {
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
        drag_state.active_pointer = None;
    }
}

fn touch_input_system(
    mut drag_state: ResMut<DragState>,
    mut touch_events: EventReader<TouchInput>,
    mut throw_events: EventWriter<ThrowAction>,
) {
    for touch in touch_events.read() {
        let position = touch.position;

        match touch.phase {
            TouchPhase::Started => {
                // Only handle first touch
                if drag_state.active_pointer.is_none() {
                    drag_state.start_position = Some(position);
                    drag_state.current_position = Some(position);
                    drag_state.active_pointer = Some(PointerId::Touch(touch.id));
                }
            }
            TouchPhase::Moved => {
                // Update only if this is the active touch
                if matches!(drag_state.active_pointer, Some(PointerId::Touch(id)) if id == touch.id)
                {
                    if let Some(start_pos) = drag_state.start_position {
                        drag_state.current_position = Some(position);
                        let drag_vector = start_pos - position;
                        let throw_params = compute_throw_params(drag_vector);
                        throw_events.send(ThrowAction::Holding(throw_params));
                    }
                }
            }
            TouchPhase::Ended => {
                // Handle touch release only for active touch
                if matches!(drag_state.active_pointer, Some(PointerId::Touch(id)) if id == touch.id)
                {
                    if let (Some(start_pos), Some(end_pos)) =
                        (drag_state.start_position, drag_state.current_position)
                    {
                        let drag_vector = start_pos - end_pos;
                        let throw_params = compute_throw_params(drag_vector);
                        throw_events.send(ThrowAction::Throw(throw_params));
                    }
                    // Reset drag state
                    drag_state.start_position = None;
                    drag_state.current_position = None;
                    drag_state.active_pointer = None;
                }
            }
            TouchPhase::Canceled => {
                drag_state.start_position = None;
                drag_state.current_position = None;
                drag_state.active_pointer = None;
            }
        }
    }
}

// Constants for throw mechanics
const MAX_DRAG_DISTANCE: f32 = 300.0; // Maximum drag distance for full power
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
