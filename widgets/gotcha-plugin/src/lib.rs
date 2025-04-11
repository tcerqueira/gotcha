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

fn set_up_gotcha() {
    #[cfg(target_arch = "wasm32")]
    let _ = gotcha_lib::init();
}

fn start_gameplay(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut touch_events: EventReader<TouchInput>,
    mut next_state: ResMut<NextState<GotchaState>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        next_state.set(GotchaState::Gameplay);
        return;
    }
    for touch in touch_events.read() {
        if matches!(touch, TouchInput { phase: TouchPhase::Ended, .. }) {
            next_state.set(GotchaState::Gameplay);
            return;
        }
    }
}

fn handle_gameplay_attempt_event(
    mut attempt_count: ResMut<AttemptCount>,
    mut event_r: EventReader<GameplayAttempt>,
    mut next_state: ResMut<NextState<GotchaState>>,
    mut game_over_state: ResMut<NextState<GameOverState>>,
) {
    for evt in event_r.read() {
        **attempt_count += 1;
        match evt {
            GameplayAttempt::Success => {
                next_state.set(GotchaState::GameOver);
                game_over_state.set(GameOverState::Success);
            }
            GameplayAttempt::Failure => match **attempt_count {
                0..3 => next_state.set(GotchaState::TryAgain),
                _ => {
                    next_state.set(GotchaState::GameOver);
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
