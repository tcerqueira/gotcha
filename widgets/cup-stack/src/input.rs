use bevy::input::touch::TouchPhase;
use bevy::{input::InputSystem, prelude::*, window::PrimaryWindow};
use gotcha_plugin::GotchaState;
use rust_fsm::{StateMachine, StateMachineImpl, TransitionImpossibleError};

pub struct ThrowInputPlugin;

impl Plugin for ThrowInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ThrowAction>();
        app.init_resource::<DragStateMachine>();
        app.add_systems(OnEnter(GotchaState::Gameplay), start_debounce_timer);
        app.add_systems(
            PreUpdate,
            (mouse_input_system, touch_input_system)
                .run_if(in_state(GotchaState::Gameplay).and(check_debounce_timer))
                .after(InputSystem),
        );
        app.add_systems(
            Update,
            tick_debounce_timer.run_if(in_state(GotchaState::Gameplay)),
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

#[derive(Resource, Debug, Default)]
struct DragStateMachine(StateMachine<DragInputSystem>);

#[derive(Resource)]
struct DebounceTimer(Timer);

fn start_debounce_timer(mut commands: Commands) {
    commands.insert_resource(DebounceTimer(Timer::from_seconds(0.2, TimerMode::Once)));
}

fn tick_debounce_timer(mut timer: ResMut<DebounceTimer>, time: Res<Time>) {
    timer.0.tick(time.delta());
}

fn check_debounce_timer(timer: Res<DebounceTimer>) -> bool {
    timer.0.finished()
}

fn mouse_input_system(
    mut drag_sm: ResMut<DragStateMachine>,
    mut throw_events: EventWriter<ThrowAction>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let pointer = PointerId::Mouse;
    // Get current mouse position
    let current_mouse_position = if let Some(position) = window.cursor_position() {
        position
    } else {
        return;
    };

    let output = match &*mouse_button_input {
        mouse if mouse.just_pressed(MouseButton::Left) => drag_sm
            .0
            .consume(&DragIn::start(pointer, current_mouse_position)),
        mouse if mouse.pressed(MouseButton::Left) => drag_sm
            .0
            .consume(&DragIn::moved(pointer, current_mouse_position)),
        mouse if mouse.just_released(MouseButton::Left) => drag_sm.0.consume(&DragIn::end(pointer)),
        _ => return,
    };
    DragInputSystem::handle_output(&mut throw_events, output);
}

fn touch_input_system(
    mut drag_sm: ResMut<DragStateMachine>,
    mut touch_events: EventReader<TouchInput>,
    mut throw_events: EventWriter<ThrowAction>,
) {
    for touch in touch_events.read() {
        let pointer = PointerId::Touch(touch.id);
        let position = touch.position;

        let output = match touch.phase {
            TouchPhase::Started => drag_sm.0.consume(&DragIn::start(pointer, position)),
            TouchPhase::Moved => drag_sm.0.consume(&DragIn::moved(pointer, position)),
            TouchPhase::Ended => drag_sm.0.consume(&DragIn::end(pointer)),
            TouchPhase::Canceled => drag_sm.0.consume(&DragIn::cancel(pointer)),
        };
        DragInputSystem::handle_output(&mut throw_events, output);
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
    let dir = Dir2::new(drag_vec).unwrap_or(Dir2::NEG_Y);
    ThrowParams { impulse, dir }
}

#[derive(Debug)]
struct DragInputSystem;

impl StateMachineImpl for DragInputSystem {
    type Input = DragIn;
    type State = DragState;
    type Output = DragOut;

    const INITIAL_STATE: Self::State = DragState::None;

    fn transition(state: &Self::State, input: &Self::Input) -> Option<Self::State> {
        match (state, input) {
            (DragState::None, DragIn { kind: DragInKind::Start(pos), pointer }) => {
                Some(DragState::Dragging {
                    start_position: *pos,
                    current_position: *pos,
                    active_pointer: *pointer,
                })
            }
            (DragState::Dragging { active_pointer, .. }, DragIn { pointer, .. })
                if active_pointer != pointer =>
            {
                None // ignore every input that does not come from the same pointer
            }
            (
                DragState::Dragging { start_position, .. },
                DragIn { kind: DragInKind::Move(pos), pointer },
            ) => Some(DragState::Dragging {
                start_position: *start_position,
                current_position: *pos,
                active_pointer: *pointer,
            }),
            (
                DragState::Dragging { .. },
                DragIn { kind: DragInKind::End | DragInKind::Cancel, .. },
            ) => Some(DragState::None),
            _ => None,
        }
    }

    fn output(state: &Self::State, input: &Self::Input) -> Option<Self::Output> {
        match (state, input) {
            (
                DragState::Dragging { start_position, .. },
                DragIn { kind: DragInKind::Move(current_position), .. },
            ) => Some(DragOut::Holding(compute_throw_params(
                current_position - start_position,
            ))),
            (
                DragState::Dragging { start_position, current_position, .. },
                DragIn { kind: DragInKind::End, .. },
            ) => Some(DragOut::Throw(compute_throw_params(
                current_position - start_position,
            ))),
            _ => None,
        }
    }
}

impl DragInputSystem {
    pub fn handle_output(
        throw_events: &mut EventWriter<ThrowAction>,
        output: Result<Option<DragOut>, TransitionImpossibleError>,
    ) {
        match output {
            Ok(Some(DragOut::Holding(throw_params))) => {
                throw_events.send(ThrowAction::Holding(throw_params));
            }
            Ok(Some(DragOut::Throw(throw_params))) => {
                throw_events.send(ThrowAction::Throw(throw_params));
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
enum DragState {
    None,
    Dragging {
        start_position: Vec2,
        current_position: Vec2,
        active_pointer: PointerId,
    },
}

#[derive(Debug)]
struct DragIn {
    kind: DragInKind,
    pointer: PointerId,
}

#[derive(Debug)]
enum DragInKind {
    Start(Vec2),
    Move(Vec2),
    End,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PointerId {
    Mouse,
    Touch(u64),
}

#[derive(Debug)]
enum DragOut {
    Holding(ThrowParams),
    Throw(ThrowParams),
}

impl DragIn {
    pub fn start(pointer: PointerId, pos: Vec2) -> Self {
        Self { kind: DragInKind::Start(pos), pointer }
    }
    pub fn moved(pointer: PointerId, pos: Vec2) -> Self {
        Self { kind: DragInKind::Move(pos), pointer }
    }
    pub fn end(pointer: PointerId) -> Self {
        Self { kind: DragInKind::End, pointer }
    }
    pub fn cancel(pointer: PointerId) -> Self {
        Self { kind: DragInKind::Cancel, pointer }
    }
}
