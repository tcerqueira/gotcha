use std::f32::consts::PI;

use bevy::{color::palettes::css::PURPLE, prelude::*};
use bevy_rapier3d::prelude::*;
use gotcha_plugin::GotchaState;

use crate::{
    camera::move_camera,
    game::is_first_attempt,
    input::{IMPULSE_MAGNITUDE, ThrowAction, ThrowParams},
};

pub struct ThrowablePlugin;

impl Plugin for ThrowablePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ThrowablesLeftCount(3));
        app.add_event::<ThrownEvent>();
        app.add_systems(Startup, spawn_throwable);
        app.add_systems(
            Update,
            (
                (
                    follow_camera.after(move_camera),
                    draw_trajectory_prediction
                        .after(follow_camera)
                        .run_if(resource_exists::<Aiming>),
                    throwing_object.after(follow_camera),
                )
                    .run_if(any_with_component::<Throwable>),
                handle_throw,
            ),
        );
        app.add_systems(
            OnEnter(GotchaState::Gameplay),
            spawn_throwable.run_if(not(is_first_attempt)),
        );
        // app.add_systems(Update, debug_throwables_left);
    }
}

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Throwable;

#[derive(Resource)]
struct ThrowableHandles {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Bundle)]
struct ThrowableBundle {
    mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
    transform: Transform,
    rigid_body: RigidBody,
    collider: Collider,
    impulse: ExternalImpulse,
    ccd: Ccd,
    restitution: Restitution,
    mass: ColliderMassProperties,
    read_mass: ReadMassProperties,
    read_velocity: Velocity,
    damping: Damping,
    _throwable: Throwable,
    _ball: Ball,
}

pub const THROW_RADIUS: f32 = 0.05;
pub const THROW_MASS_DENSITY: f32 = 10.;
pub const THROW_DAMPING: f32 = 0.;

impl Default for ThrowableBundle {
    fn default() -> Self {
        Self {
            mesh: default(),
            material: default(),
            transform: default(),
            rigid_body: RigidBody::KinematicPositionBased,
            collider: Collider::ball(THROW_RADIUS),
            impulse: ExternalImpulse { impulse: Vec3::new(0., 0., 0.), ..default() },
            ccd: Ccd::enabled(),
            restitution: Restitution::coefficient(0.7),
            mass: ColliderMassProperties::Density(THROW_MASS_DENSITY),
            read_mass: ReadMassProperties::default(),
            read_velocity: Velocity::default(),
            damping: Damping { linear_damping: THROW_DAMPING, angular_damping: THROW_DAMPING },
            _throwable: Throwable,
            _ball: Ball,
        }
    }
}

fn spawn_throwable(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const THROW_DIM: f32 = 0.05;
    let mesh = meshes.add(Sphere::new(THROW_DIM));
    let material = materials.add(StandardMaterial { base_color: PURPLE.into(), ..default() });
    commands.spawn(ThrowableBundle {
        mesh: Mesh3d(mesh.clone()),
        material: MeshMaterial3d(material.clone()),
        ..default()
    });
    commands.insert_resource(ThrowableHandles { mesh, material });
}

fn throwable_to_cam_transform(camera: &Transform) -> Transform {
    Transform::from_rotation(camera.rotation).with_translation(
        camera.translation + camera.forward() * 1. + camera.down() * 0.2 + camera.right() * 0.1,
    )
}

fn follow_camera(
    mut throwable: Single<&mut Transform, With<Throwable>>,
    camera: Single<&Transform, (With<Camera3d>, Without<Throwable>)>,
) {
    **throwable = throwable_to_cam_transform(&camera);
}

fn throwing_object(
    mut commands: Commands,
    mut event_r: EventReader<ThrowAction>,
    mut throw_w: EventWriter<ThrownEvent>,
    mut throwable: Single<(Entity, &mut ExternalImpulse), With<Throwable>>,
    camera: Single<&Transform, With<Camera3d>>,
    mut aiming: Option<ResMut<Aiming>>,
) {
    let (entity, ref mut external_impulse) = *throwable;

    for action in event_r.read() {
        match action {
            ThrowAction::Throw(ThrowParams { impulse, dir }) => {
                commands
                    .entity(entity)
                    .remove::<Throwable>()
                    .remove::<RigidBody>()
                    .insert(RigidBody::Dynamic);

                let throw_direction = throw_dir3(&camera.forward(), &camera.right(), *dir);
                external_impulse.impulse = throw_direction * *impulse;

                throw_w.send(ThrownEvent);
                commands.remove_resource::<Aiming>();
            }
            ThrowAction::Holding(throw_params) => match aiming {
                Some(ref mut aiming) => aiming.0 = *throw_params,
                None => commands.insert_resource(Aiming(*throw_params)),
            },
        }
    }
}

#[derive(Resource)]
pub struct ThrowablesLeftCount(pub u8);

#[derive(Event)]
struct ThrownEvent;

#[derive(Default)]
struct ThrowableTimers {
    spawn: Vec<Timer>,
    decrement: Vec<Timer>,
}

fn handle_throw(
    mut commands: Commands,
    camera: Single<&Transform, With<Camera3d>>,
    mut throw_r: EventReader<ThrownEvent>,
    throwable_handles: Res<ThrowableHandles>,
    mut throwable_timers: Local<ThrowableTimers>,
    time: Res<Time>,
    mut throwables_left: ResMut<ThrowablesLeftCount>,
) {
    let ThrowableTimers { spawn: spawn_timer, decrement: decrement_timer } = &mut *throwable_timers;
    spawn_timer.retain(|t| !t.finished());
    decrement_timer.retain(|t| !t.finished());

    for spawn in spawn_timer {
        if spawn.tick(time.delta()).just_finished() {
            commands.spawn(ThrowableBundle {
                mesh: Mesh3d(throwable_handles.mesh.clone()),
                material: MeshMaterial3d(throwable_handles.material.clone()),
                transform: throwable_to_cam_transform(&camera),
                ..default()
            });
        }
    }
    for decrement in decrement_timer {
        if decrement.tick(time.delta()).just_finished() {
            throwables_left.0 -= 1;
        }
    }

    for _ in throw_r.read() {
        throwable_timers
            .spawn
            .push(Timer::from_seconds(1.5, TimerMode::Once));
        throwable_timers
            .decrement
            .push(Timer::from_seconds(1.5, TimerMode::Once));
    }
}

#[expect(dead_code)]
fn debug_throwables_left(throwables_left: Res<ThrowablesLeftCount>) {
    if throwables_left.is_changed() {
        debug!("throwables left: {}", throwables_left.0);
    }
}

const TRAJECTORY_STEPS: usize = 50;
const TRAJECTORY_TIME_STEP: f32 = 0.025;
const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

#[derive(Resource)]
struct Aiming(ThrowParams);

impl Default for Aiming {
    fn default() -> Self {
        Self(ThrowParams { impulse: IMPULSE_MAGNITUDE, dir: Dir2::Y })
    }
}

#[allow(clippy::type_complexity)]
fn draw_trajectory_prediction(
    camera: Single<&Transform, With<Camera3d>>,
    throwable: Single<(&Transform, &Velocity, &ReadMassProperties, &Damping), With<Throwable>>,
    aiming: Res<Aiming>,
    mut gizmos: Gizmos,
) {
    let (transform, velocity, mass, damping) = *throwable;

    let start_pos = transform.translation;
    let throw_direction = throw_dir3(&camera.forward(), &camera.right(), aiming.0.dir);

    let throw_strength = aiming.0.impulse / mass.mass;
    let velocity = (throw_direction * throw_strength) + velocity.linvel;

    let mut points = Vec::with_capacity(TRAJECTORY_STEPS + 1);
    points.push(start_pos);

    let mut pos = start_pos;
    let mut vel = velocity;

    // Simulate projectile motion
    for _ in 0..TRAJECTORY_STEPS {
        vel *= 1.0 - damping.linear_damping * TRAJECTORY_TIME_STEP;
        vel += GRAVITY * TRAJECTORY_TIME_STEP;
        pos += vel * TRAJECTORY_TIME_STEP;
        points.push(pos);
    }

    // Draw trajectory line
    for i in 0..points.len() - 1 {
        gizmos.line(points[i], points[i + 1], Color::srgba(1.0, 1.0, 0.0, 0.6));
    }

    // Draw points at intervals
    for (i, point) in points.iter().enumerate() {
        if i % 5 == 0 {
            gizmos.sphere(*point, 0.02, Color::srgba(1.0, 0.5, 0.0, 0.7));
        }
    }
}

fn throw_dir3(forward: &Dir3, right: &Dir3, dir: Dir2) -> Dir3 {
    let tilt = Quat::from_axis_angle(right.as_vec3(), 30.0f32.to_radians());
    let yaw = Quat::from_axis_angle(Dir3::Y.as_vec3(), -dir.to_angle() - PI / 2.);
    Dir3::new_unchecked(yaw * tilt * forward.as_vec3())
}
