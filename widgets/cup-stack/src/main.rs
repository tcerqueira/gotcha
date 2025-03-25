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
    App::new()
        .add_plugins(DefaultPlugins)
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

#[derive(Event)]
enum GameResult {
    Success,
    Failure,
}

fn check_game_result(mut event_r: EventReader<GameResult>, mut event_w: EventWriter<AppExit>) {
    for res in event_r.read() {
        match res {
            GameResult::Success => info!("success"),
            GameResult::Failure => info!("failure"),
        }
        event_w.send(AppExit::Success);
    }
}
