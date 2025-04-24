use bevy::prelude::*;

use crate::{cup::TargetsLeft, throwable::ThrowablesLeftCount};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui);
        app.add_systems(Update, (update_targets_text, update_throwables_text));
    }
}

#[derive(Component)]
struct TargetsLeftText;

#[derive(Component)]
struct ThrowablesLeftText;

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
            parent.spawn((
                Text::new("Swipe to aim, release to throw"),
                TextFont::from_font_size(20.),
                Node { margin: UiRect::horizontal(Val::Auto), ..default() },
            ));
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                bottom: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("Targets left: 6"), TargetsLeftText));
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                bottom: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("Balls: 3"), ThrowablesLeftText));
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

fn update_throwables_text(
    throwables_left: Res<ThrowablesLeftCount>,
    mut query: Query<&mut Text, With<ThrowablesLeftText>>,
) {
    if throwables_left.is_changed() {
        for mut text in &mut query {
            text.0 = format!("Balls: {}", throwables_left.0);
        }
    }
}
