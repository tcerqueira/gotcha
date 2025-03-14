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
        .run();
}
