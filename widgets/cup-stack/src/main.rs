use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use camera::*;
use cup::*;
use game::*;
use input::*;
use throwable::*;
use ui::*;

mod camera;
mod cup;
mod game;
mod input;
mod throwable;
mod ui;

fn main() {
    #[cfg(target_arch = "wasm32")]
    let _ = gotcha_lib::init();
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (360., 500.).into(),
                prevent_default_event_handling: false,
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            // RapierDebugRenderPlugin::default(),
        ))
        .add_plugins((
            UiPlugin,
            GamePlugin,
            CameraPlugin,
            CupsPlugin,
            ThrowInputPlugin,
            ThrowablePlugin,
        ))
        .add_event::<GameResult>()
        .add_systems(PostUpdate, check_game_result)
        .run();
}

#[derive(Debug, Event, Clone, PartialEq, Eq, Hash)]
enum GameResult {
    Success,
    Failure,
}

fn check_game_result(mut event_r: EventReader<GameResult>) {
    #[cfg(target_arch = "wasm32")]
    use bevy::tasks::AsyncComputeTaskPool;

    for res in event_r.read() {
        match res {
            GameResult::Success => {
                info!("success");
                #[cfg(target_arch = "wasm32")]
                AsyncComputeTaskPool::get().spawn(async {
                    gotcha_lib::send_challenge_result(true).await;
                });
            }
            GameResult::Failure => {
                info!("failure");
                #[cfg(target_arch = "wasm32")]
                AsyncComputeTaskPool::get().spawn(async {
                    gotcha_lib::send_challenge_result(false).await;
                });
            }
        }
    }
}
