use bevy::{prelude::*, ui::UiSystem};

use super::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            try_again_action
                .after(UiSystem::Focus)
                .run_if(in_state(GotchaState::TryAgain)),
        );
        app.add_systems(OnEnter(GotchaState::Welcome), setup_welcome_ui);
        app.add_systems(OnExit(GotchaState::Welcome), destroy_welcome_ui);
        app.add_systems(OnEnter(GotchaState::TryAgain), setup_try_again_ui);
        app.add_systems(OnExit(GotchaState::TryAgain), destroy_try_again_ui);
        app.add_systems(OnEnter(GotchaState::GameOver), setup_gameover_ui);
    }
}

#[derive(Component)]
struct WelcomeUi;

fn setup_welcome_ui(mut commands: Commands) {
    commands
        .spawn((
            WelcomeUi,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.5)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Click to start!"),
                TextFont { font_size: 30., ..default() },
                TextColor(Color::WHITE),
                Node { margin: UiRect::all(Val::Auto), ..default() },
            ));
        });
}

fn destroy_welcome_ui(mut commands: Commands, query: Query<Entity, With<WelcomeUi>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
pub struct TryAgainUi;

#[derive(Component)]
pub struct TryAgainButton;

fn setup_try_again_ui(mut commands: Commands) {
    commands
        .spawn((
            TryAgainUi,
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
                    parent.spawn((
                        TryAgainButton,
                        Button,
                        Text::new("Try again"),
                        TextFont { font_size: 30., ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

#[allow(clippy::type_complexity)]
fn try_again_action(
    commands: Commands,
    mut button: Option<
        Single<(&Interaction, &mut TextColor), (With<TryAgainButton>, Changed<Interaction>)>,
    >,
    mut gotcha_state: ResMut<NextState<GotchaState>>,
    debounce_gameplay_timer: Option<Res<GameplayDebounceTimer>>,
) {
    if debounce_gameplay_timer.is_some_and(|timer| timer.0.finished()) {
        gotcha_state.set(GotchaState::Gameplay);
    }
    let Some((interaction, text_color)) = button.as_deref_mut() else {
        return;
    };

    match interaction {
        Interaction::Pressed => {
            start_gameplay_timer(commands);
        }
        Interaction::Hovered => {
            text_color.0 = Color::srgb(0., 0.8, 0.);
        }
        Interaction::None => {
            text_color.0 = Color::WHITE;
        }
    };
}

fn destroy_try_again_ui(mut commands: Commands, query: Query<Entity, With<TryAgainUi>>) {
    for ui in &query {
        commands.entity(ui).despawn_recursive();
    }
}

#[derive(Component)]
struct GameOverUi;

fn setup_gameover_ui(
    mut commands: Commands,
    game_over_state: Res<State<GameOverState>>,
    attempt_count: Res<AttemptCount>,
) {
    let bg_color = match game_over_state.get() {
        GameOverState::Success => Color::srgba(0., 0.8, 0., 0.3),
        GameOverState::Fail => Color::srgba(0.8, 0., 0., 0.3),
    };
    commands
        .spawn((
            GameOverUi,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            BackgroundColor(bg_color),
        ))
        .with_children(|parent| match game_over_state.get() {
            GameOverState::Success => setup_gameover_success_ui(parent, *attempt_count),
            GameOverState::Fail => setup_gameover_failure_ui(parent),
        });
}

fn setup_gameover_failure_ui(parent: &mut ChildBuilder<'_>) {
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
        });
}

fn setup_gameover_success_ui(parent: &mut ChildBuilder<'_>, attempt_count: AttemptCount) {
    parent
        .spawn((
            Node {
                margin: UiRect::horizontal(Val::Auto),
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
                Text::new(match *attempt_count {
                    1 => "Perfect!",
                    2 => "Good job.",
                    3 => "Close enough...",
                    _ => "Hmmm 🤨",
                }),
                TextFont { font_size: 40., ..default() },
                TextColor(Color::WHITE),
            ));
        });
}
