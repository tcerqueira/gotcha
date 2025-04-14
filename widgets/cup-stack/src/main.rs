use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use camera::*;
use cup::*;
use game::*;
use gotcha_plugin::GotchaPlugin;
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
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // resolution: (720., 1000.).into(),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            // RapierDebugRenderPlugin::default(),
        ))
        .add_plugins(GotchaPlugin)
        .add_plugins((
            UiPlugin,
            GamePlugin,
            CameraPlugin,
            CupsPlugin,
            ThrowInputPlugin,
            ThrowablePlugin,
        ))
        .run();
}
