use bevy::{prelude::*, ui::UiSystem};

use crate::{
    GameResult,
    cup::TargetsLeft,
    game::{AppState, AttemptCount},
    throwable::ThrowablesLeftCount,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui);
        app.add_systems(PreUpdate, try_again_button.after(UiSystem::Focus));
        app.add_systems(Update, (update_targets_text, update_throwables_text));
        app.add_systems(OnEnter(AppState::Welcome), setup_welcome_ui);
        app.add_systems(OnEnter(AppState::Gameplay), setup_gameplay_ui);
        app.add_systems(
            OnEnter(AppState::GameOver(GameResult::Failure)),
            setup_gameover_ui,
        );
        app.add_systems(
            OnExit(AppState::GameOver(GameResult::Failure)),
            destroy_gameover_ui,
        );
        app.add_systems(
            OnEnter(AppState::GameOver(GameResult::Success)),
            setup_success_ui,
        );
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
                Text::new("Knock over all the targets!"),
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

fn setup_welcome_ui() {}
fn setup_gameplay_ui() {}

#[derive(Component)]
struct GameOverUi;

#[derive(Component)]
pub struct TryAgainButton;

fn setup_gameover_ui(mut commands: Commands, attempts: Res<AttemptCount>) {
    commands
        .spawn((
            GameOverUi,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            BackgroundColor(Color::srgba(0.8, 0., 0., 0.3)),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        margin: UiRect::horizontal(Val::Auto),
                        width: Val::Px(300.),
                        padding: UiRect::all(Val::Px(20.)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(20.),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0., 0., 0., 0.8)),
                    Transform::from_xyz(-150., -100., 0.),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Game Over"),
                        TextFont { font_size: 40., ..default() },
                        TextColor(Color::WHITE),
                    ));
                    if attempts.0 < 3 {
                        parent.spawn((
                            TryAgainButton,
                            Button,
                            Text::new("Try again"),
                            TextFont { font_size: 30., ..default() },
                            TextColor(Color::WHITE),
                        ));
                    }
                });
        });
}

#[allow(clippy::type_complexity)]
fn try_again_button(
    mut button: Option<
        Single<(&Interaction, &mut TextColor), (With<TryAgainButton>, Changed<Interaction>)>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some((interaction, text_color)) = button.as_deref_mut() else {
        return;
    };

    match interaction {
        Interaction::Pressed => {
            next_state.set(AppState::Gameplay);
        }
        Interaction::Hovered => {
            text_color.0 = Color::srgb(0., 0.8, 0.);
        }
        Interaction::None => {
            text_color.0 = Color::WHITE;
        }
    };
}

fn destroy_gameover_ui(mut commands: Commands, query: Query<Entity, With<GameOverUi>>) {
    for ui in &query {
        commands.entity(ui).despawn_recursive();
    }
}

fn setup_success_ui(mut commands: Commands, attempts: Res<AttemptCount>) {
    commands
        .spawn((
            GameOverUi,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0.8, 0., 0.3)),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        margin: UiRect::horizontal(Val::Auto),
                        width: Val::Px(300.),
                        padding: UiRect::all(Val::Px(20.)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(20.),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0., 0., 0., 0.8)),
                    Transform::from_xyz(-150., -100., 0.),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(match attempts.0 {
                            1 => "Perfect! 🚀",
                            2 => "Good job. 👍",
                            3 => "Close enough... 👏",
                            _ => "Hmmm 🤨",
                        }),
                        TextFont { font_size: 40., ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}
