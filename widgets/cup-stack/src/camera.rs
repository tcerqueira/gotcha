use bevy::prelude::*;

use crate::game::AppState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(
            Update,
            (move_camera, rotate_camera).run_if(in_state(AppState::Gameplay)),
        );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 1.75, 5.0).with_rotation(Quat::from_axis_angle(Vec3::Y, 0.)),
    ));
}

pub fn move_camera(
    mut camera: Single<&mut Transform, With<Camera3d>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut delta_move = Vec3::ZERO;
    if input.pressed(KeyCode::KeyW) {
        delta_move += *camera.forward();
    }
    if input.pressed(KeyCode::KeyA) {
        delta_move += *camera.left();
    }
    if input.pressed(KeyCode::KeyS) {
        delta_move += *camera.back();
    }
    if input.pressed(KeyCode::KeyD) {
        delta_move += *camera.right();
    }
    if input.pressed(KeyCode::Space) {
        delta_move += Vec3::Y;
    }
    if input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight) {
        delta_move += -Vec3::Y;
    }

    if delta_move != Vec3::ZERO {
        delta_move = delta_move.normalize();
        camera.translation += delta_move * 5. * time.delta_secs();
    }
}

fn rotate_camera(
    mut camera: Single<&mut Transform, With<Camera3d>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut angle = 0.;
    if input.pressed(KeyCode::KeyQ) {
        angle += 1.;
    }
    if input.pressed(KeyCode::KeyE) {
        angle += -1.;
    }

    if angle != 0. {
        camera.rotate_y(angle * 2.5 * time.delta_secs());
    }
}
