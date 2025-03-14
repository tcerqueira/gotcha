use bevy::prelude::*;

use crate::{cup::TargetsLeft, game::AppState};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui);
        app.add_systems(Update, update_targets_text);
        app.add_systems(OnEnter(AppState::Welcome), setup_welcome_ui);
        app.add_systems(OnEnter(AppState::Gameplay), setup_gameplay_ui);
        app.add_systems(OnEnter(AppState::GameOver), setup_gameover_ui);
    }
}

#[derive(Component)]
struct TargetsLeftText;

fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            parent.spawn((Text("6".into()), TargetsLeftText));
        });
}

fn update_targets_text(
    targets_left: Res<TargetsLeft>,
    mut query: Query<&mut Text, With<TargetsLeftText>>,
) {
    if targets_left.is_changed() {
        for mut text in &mut query {
            text.0 = format!("Targets left: {}", targets_left.0);
        }
    }
}

fn setup_welcome_ui() {}
fn setup_gameplay_ui() {}
fn setup_gameover_ui() {}
