#![allow(dead_code)]
use bevy::{prelude::*, window::WindowResized};

const GRID_SIZE: usize = 6;

#[derive(Resource, Default)]
struct GridResolution {
    grid_size: f32,
    cell_size: f32,
}

#[derive(Resource, Default)]
struct GameState {
    grid: Grid,
}

#[derive(Default)]
struct Grid {
    cells: [[CellType; GRID_SIZE]; GRID_SIZE],
}

#[derive(Default)]
enum CellType {
    #[default]
    Empty,
    Sun,
    Moon,
}

#[derive(Component)]
struct GridCell {
    row: usize,
    col: usize,
}

#[derive(Event)]
struct GridCellEvent {
    row: usize,
    col: usize,
    new_type: CellType,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<GridCellEvent>()
        .insert_resource(GridResolution::default())
        .insert_resource(GameState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (on_resize_window, draw_outlines))
        .run();
}

fn setup(
    mut commands: Commands,
    window: Single<&Window>,
    mut state: ResMut<GameState>,
    mut resolution: ResMut<GridResolution>,
) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);

    // Get window dimensions
    let window_width = window.width();
    let window_height = window.height();

    // Calculate grid size (90% of the smallest dimension)
    *resolution = grid_resolution(window_width, window_height);

    // Spawn grid cells
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let (x, y) = cell_pos(row, col, &resolution);

            // Initialize grid state (separate from entities)
            state.grid.cells[row][col] = CellType::Empty;

            // Spawn cell entity for visual representation
            commands.spawn((
                GridCell { row, col },
                Sprite {
                    color: Color::srgba(0., 0., 0., 0.),
                    custom_size: Some(Vec2::new(resolution.cell_size, resolution.cell_size)),
                    ..default()
                },
                Transform::from_xyz(x, y, 0.0),
            ));
        }
    }
}

fn draw_outlines(
    mut gizmos: Gizmos,
    cells: Query<&Transform, With<GridCell>>,
    resolution: Res<GridResolution>,
) {
    for transform in &cells {
        let pos = transform.translation;
        let half_cell = resolution.cell_size / 2.;
        let tl_pos = Vec2::new(pos.x - half_cell, pos.y + half_cell);
        let tr_pos = Vec2::new(pos.x + half_cell, pos.y + half_cell);
        let bl_pos = Vec2::new(pos.x - half_cell, pos.y - half_cell);
        let br_pos = Vec2::new(pos.x + half_cell, pos.y - half_cell);
        gizmos.line_2d(tl_pos, tr_pos, Color::WHITE);
        gizmos.line_2d(tr_pos, br_pos, Color::WHITE);
        gizmos.line_2d(br_pos, bl_pos, Color::WHITE);
        gizmos.line_2d(bl_pos, tl_pos, Color::WHITE);
    }
}

fn on_resize_window(
    mut resize_reader: EventReader<WindowResized>,
    mut cells: Query<(&mut Sprite, &mut Transform, &GridCell), With<GridCell>>,
    mut resolution: ResMut<GridResolution>,
) {
    for e in resize_reader.read() {
        *resolution = grid_resolution(e.width, e.height);
    }
    for (mut sprite, mut transform, cell) in &mut cells {
        if let Some(s) = sprite.custom_size.as_mut() {
            *s = Vec2::new(resolution.cell_size, resolution.cell_size);
        }
        let (x, y) = cell_pos(cell.row, cell.col, &resolution);
        *transform = Transform::from_xyz(x, y, 0.);
    }
}

fn grid_resolution(win_width: f32, win_height: f32) -> GridResolution {
    let grid_size = 0.9 * f32::min(win_width, win_height);
    let cell_size = grid_size / GRID_SIZE as f32;
    GridResolution { grid_size, cell_size }
}

fn cell_pos(row: usize, col: usize, resolution: &GridResolution) -> (f32, f32) {
    let start_x = -resolution.grid_size / 2.0;
    let start_y = resolution.grid_size / 2.0;
    let x = col as f32 * resolution.cell_size;
    let y = -(row as f32 * resolution.cell_size);
    (
        start_x + x + resolution.cell_size / 2.,
        start_y + y - resolution.cell_size / 2.,
    )
}
