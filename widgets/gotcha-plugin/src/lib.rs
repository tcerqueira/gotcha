use bevy::{
    input::{InputSystem, touch::TouchPhase},
    prelude::*,
};

use ui::*;

#[cfg(target_arch = "wasm32")]
mod gotcha_lib;
mod ui;

pub struct GotchaPlugin;

impl Plugin for GotchaPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GotchaState>();
        app.add_sub_state::<GameOverState>();
        app.insert_resource(AttemptCount(0));
        app.add_event::<GameplayAttempt>();
        app.add_plugins(UiPlugin);
        app.add_systems(Startup, set_up_gotcha);
        app.add_systems(
            PreUpdate,
            start_gameplay
                .run_if(in_state(GotchaState::Welcome))
                .after(InputSystem),
        );
        app.add_systems(
            PostUpdate,
            handle_gameplay_attempt_event.run_if(in_state(GotchaState::Gameplay)),
        );
        app.add_systems(
            Update,
            tick_debounce_timer.run_if(resource_exists::<GameplayDebounceTimer>),
        );
        app.add_systems(OnEnter(GotchaState::Gameplay), remove_debounce_timer);
        app.add_systems(OnEnter(GotchaState::GameOver), handle_gameover);
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Default, Hash)]
pub enum GotchaState {
    #[default]
    Welcome,
    Gameplay,
    TryAgain,
    GameOver,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(GotchaState = GotchaState::GameOver)]
enum GameOverState {
    #[default]
    Success,
    Fail,
}

#[derive(
    Resource, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deref, DerefMut,
)]
pub struct AttemptCount(pub u8);

impl AttemptCount {
    pub fn as_result(&self) -> Result<u8, u8> {
        match **self {
            count @ 0..=3 => Ok(count),
            count => Err(count),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameplayAttempt {
    Success,
    Failure,
}

#[derive(Resource)]
struct GameplayDebounceTimer(Timer);

fn start_gameplay_timer(mut commands: Commands) {
    commands.insert_resource(GameplayDebounceTimer(Timer::from_seconds(
        0.2,
        TimerMode::Once,
    )));
}

fn tick_debounce_timer(mut timer: ResMut<GameplayDebounceTimer>, time: Res<Time>) {
    timer.0.tick(time.delta());
}

fn remove_debounce_timer(mut commands: Commands) {
    commands.remove_resource::<GameplayDebounceTimer>();
}

fn set_up_gotcha() {
    #[cfg(target_arch = "wasm32")]
    let _ = gotcha_lib::init();
}

fn start_gameplay(
    commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut touch_events: EventReader<TouchInput>,
    debounce_gameplay_timer: Option<Res<GameplayDebounceTimer>>,
    mut gotcha_state: ResMut<NextState<GotchaState>>,
) {
    if debounce_gameplay_timer.is_some_and(|timer| timer.0.finished()) {
        gotcha_state.set(GotchaState::Gameplay);
    }
    if mouse_input.just_pressed(MouseButton::Left) {
        start_gameplay_timer(commands);
        return;
    }
    for touch in touch_events.read() {
        if matches!(touch, TouchInput { phase: TouchPhase::Ended, .. }) {
            // next_state.set(GotchaState::Gameplay);
            start_gameplay_timer(commands);
            return;
        }
    }
}

fn handle_gameplay_attempt_event(
    mut attempt_count: ResMut<AttemptCount>,
    mut event_r: EventReader<GameplayAttempt>,
    mut gotcha_state: ResMut<NextState<GotchaState>>,
    mut game_over_state: ResMut<NextState<GameOverState>>,
) {
    for evt in event_r.read() {
        **attempt_count += 1;
        match evt {
            GameplayAttempt::Success => {
                gotcha_state.set(GotchaState::GameOver);
                game_over_state.set(GameOverState::Success);
            }
            GameplayAttempt::Failure => match **attempt_count {
                0..3 => gotcha_state.set(GotchaState::TryAgain),
                _ => {
                    gotcha_state.set(GotchaState::GameOver);
                    game_over_state.set(GameOverState::Fail);
                }
            },
        };
    }
}

fn handle_gameover(game_over_state: Res<State<GameOverState>>) {
    #[cfg(target_arch = "wasm32")]
    use bevy::tasks::AsyncComputeTaskPool;

    match game_over_state.get() {
        GameOverState::Success => {
            info!("success");
            #[cfg(target_arch = "wasm32")]
            AsyncComputeTaskPool::get().spawn(async {
                gotcha_lib::send_challenge_result(true).await;
            });
        }
        GameOverState::Fail => {
            info!("failure");
            #[cfg(target_arch = "wasm32")]
            AsyncComputeTaskPool::get().spawn(async {
                gotcha_lib::send_challenge_result(false).await;
            });
        }
    }
}
