use bevy::{
    color::palettes::css::{BLUE, GREEN, RED},
    prelude::*,
};
use bevy_rapier3d::prelude::*;

use crate::{GameResult, cup::*, throwable::ThrowablesLeftCount};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(AppState::Welcome);
        app.insert_resource(AttemptCount(1));
        app.add_systems(Startup, (setup_lighting, setup_entities));
        app.add_systems(
            PreUpdate,
            check_game_over.run_if(not(in_state(AppState::GameOver(GameResult::Failure))
                .or(in_state(AppState::GameOver(GameResult::Success))))),
        );
        app.add_systems(
            OnEnter(AppState::Gameplay),
            (
                setup_entities.run_if(not(is_first_attempt)),
                setup_throwables_left,
            ),
        );
        app.add_systems(
            OnExit(AppState::GameOver(GameResult::Failure)),
            (despawn_entities, increment_attempt_count),
        );
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Default, Hash)]
pub enum AppState {
    #[default]
    Welcome,
    Gameplay,
    GameOver(GameResult),
}

fn check_game_over(
    targets_left: Res<TargetsLeft>,
    throwables_left: Res<ThrowablesLeftCount>,
    attempts: Res<AttemptCount>,
    mut next_state: ResMut<NextState<AppState>>,
    mut event_w: EventWriter<GameResult>,
) {
    if targets_left.0 == 0 {
        event_w.send(GameResult::Success);
        next_state.set(AppState::GameOver(GameResult::Success));
        return;
    }
    if throwables_left.0 == 0 {
        if let 3.. = attempts.0 {
            event_w.send(GameResult::Failure);
        };
        next_state.set(AppState::GameOver(GameResult::Failure));
    }
}

#[derive(Resource)]
pub struct AttemptCount(pub u8);

pub fn is_first_attempt(attempts: Res<AttemptCount>) -> bool {
    attempts.0 == 1
}

fn increment_attempt_count(mut attempts: ResMut<AttemptCount>) {
    attempts.0 += 1;
}

fn setup_throwables_left(mut throwables_left: ResMut<ThrowablesLeftCount>) {
    throwables_left.0 = 3;
}

fn setup_lighting(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Add ambient light to the scene
    commands.insert_resource(AmbientLight { color: Color::srgb(0.65, 0.7, 0.9), brightness: 1. });
    // Add a directional light
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.85),
            illuminance: 10000.0,
            shadows_enabled: true,
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 0.6,
        },
        Transform::from_xyz(4.0, 8.0, 4.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 4.)),
    ));
    // Add skybox
    // commands.spawn(Skybox { image: asset_server.load("Ryfjallet_cubemap.png"), ..default() });
}

const GROUND_DIM: Vec3 = Vec3::new(200., 0.2, 200.);
pub const WALL_DIM: Vec3 = Vec3::new(5., 5., 0.5);
pub const TABLE_POS: Vec3 = Vec3::new(0., 1.5, 0.5);
pub const TABLE_DIM: Vec3 = Vec3::new(WALL_DIM.x, 0.1, 0.1);

fn setup_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(GROUND_DIM.x, GROUND_DIM.y, GROUND_DIM.z))),
        MeshMaterial3d(
            materials.add(StandardMaterial { base_color: GREEN.into(), ..Default::default() }),
        ),
        Transform::from_xyz(0., -2., 0.),
        RigidBody::Fixed,
        Collider::cuboid(GROUND_DIM.x / 2., GROUND_DIM.y / 2., GROUND_DIM.z / 2.),
    ));
    // Wall
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(WALL_DIM.x, WALL_DIM.y, WALL_DIM.z))),
        MeshMaterial3d(
            materials.add(StandardMaterial { base_color: BLUE.into(), ..Default::default() }),
        ),
        Transform::from_xyz(0., 0., 0.),
        RigidBody::Fixed,
        Collider::cuboid(WALL_DIM.x / 2., WALL_DIM.y / 2., WALL_DIM.z / 2.),
    ));
    // Table
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(TABLE_DIM.x, TABLE_DIM.y, TABLE_DIM.z))),
        MeshMaterial3d(
            materials.add(StandardMaterial { base_color: BLUE.into(), ..Default::default() }),
        ),
        Transform::from_xyz(TABLE_POS.x, TABLE_POS.y, TABLE_POS.z),
        RigidBody::Fixed,
        Collider::cuboid(TABLE_DIM.x / 2., TABLE_DIM.y / 2., TABLE_DIM.z / 2.),
    ));
    // Cups
    let mesh = meshes.add(Cylinder::new(CUP_RADIUS, CUP_HEIGHT));
    let material = materials.add(StandardMaterial { base_color: RED.into(), ..Default::default() });
    let cup_builder = |pos_x: f32, pos_y: f32, pos_z: f32| -> CupBundle {
        CupBundle {
            mesh: Mesh3d(mesh.clone()),
            material: MeshMaterial3d(material.clone()),
            transform: Transform::from_xyz(pos_x, pos_y, pos_z),
            ..default()
        }
    };
    // Pyramide of cups
    for level in 0..10 {
        let x_start_pad = CUP_RADIUS * level as f32;
        let y = CUP_HEIGHT * level as f32 + TABLE_POS.y + TABLE_DIM.y;
        for i in 0..(3 - level) {
            const GAP: f32 = CUP_RADIUS * 2. + 0.01;
            let x = GAP * i as f32 + x_start_pad;
            // shift everything left to center
            commands.spawn(cup_builder(x - (GAP * 3. / 2.), y, TABLE_POS.z));
        }
    }
}

fn despawn_entities(mut commands: Commands, rigid_bodies: Query<Entity, With<RigidBody>>) {
    for entity in &rigid_bodies {
        commands.entity(entity).despawn_recursive();
    }
}
