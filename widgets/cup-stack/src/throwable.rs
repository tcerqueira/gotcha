use bevy::{color::palettes::css::PURPLE, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::{
    camera::move_camera,
    input::{IMPULSE_MAGNITUDE, ThrowAction},
};

pub struct ThrowablePlugin;

impl Plugin for ThrowablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_throwable);
        app.add_systems(
            Update,
            (
                throw_object,
                follow_camera.after(move_camera),
                draw_trajectory_prediction.after(follow_camera),
            ),
        );
        // app.add_systems(Update, debug_throwables);
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
            impulse: ExternalImpulse { impulse: Vec3::new(0., 0., 1.), ..default() },
            ccd: Ccd::enabled(),
            restitution: Restitution::coefficient(0.7),
            mass: ColliderMassProperties::Density(THROW_MASS_DENSITY),
            read_mass: ReadMassProperties::default(),
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
    let material =
        materials.add(StandardMaterial { base_color: PURPLE.into(), ..Default::default() });
    commands.spawn(ThrowableBundle {
        mesh: Mesh3d(mesh.clone()),
        material: MeshMaterial3d(material.clone()),
        ..default()
    });
    commands.insert_resource(ThrowableHandles { mesh, material });
}

fn follow_camera(
    throwable: Option<Single<&mut Transform, With<Throwable>>>,
    camera: Single<&Transform, (With<Camera3d>, Without<Throwable>)>,
) {
    if let Some(mut throwable) = throwable {
        throwable.translation = camera.translation + camera.forward() * 1. + camera.down() * 0.2;
        throwable.rotation = camera.rotation;
    }
}

fn throw_object(
    mut commands: Commands,
    mut event_r: EventReader<ThrowAction>,
    throwable: Option<Single<(Entity, &mut ExternalImpulse), With<Throwable>>>,
    camera: Single<&Transform, With<Camera3d>>,
    throwable_handles: Res<ThrowableHandles>,
) {
    let Some(mut throwable) = throwable else {
        return;
    };
    let (entity, ref mut external_impulse) = *throwable;

    for action in event_r.read() {
        let ThrowAction::Throw { impulse, angle: _ } = action else {
            continue;
        };
        commands
            .entity(entity)
            .remove::<Throwable>()
            .remove::<RigidBody>()
            .insert(RigidBody::Dynamic);

        let throw_direction = throw_dir3(&camera.forward(), &camera.right());
        external_impulse.impulse = throw_direction * *impulse;

        commands.spawn(ThrowableBundle {
            mesh: Mesh3d(throwable_handles.mesh.clone()),
            material: MeshMaterial3d(throwable_handles.material.clone()),
            ..default()
        });
    }
}

#[expect(dead_code)]
#[allow(clippy::type_complexity)]
fn debug_throwables(balls: Query<(Entity, &Transform), (With<Ball>, Changed<Transform>)>) {
    for (ent, transform) in &balls {
        debug!("{ent} at {:?}", transform.translation);
    }
}

const TRAJECTORY_STEPS: usize = 50;
const TRAJECTORY_TIME_STEP: f32 = 0.025;
const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

#[allow(clippy::type_complexity)]
fn draw_trajectory_prediction(
    camera: Single<&Transform, With<Camera3d>>,
    throwable: Option<Single<(&Transform, &ReadMassProperties, &Damping), With<Throwable>>>,
    mut gizmos: Gizmos,
) {
    let Some(throwable) = throwable else {
        return;
    };
    let (transform, mass, damping) = *throwable;

    let start_pos = transform.translation;
    let throw_direction = throw_dir3(&camera.forward(), &camera.right());

    let throw_strength = IMPULSE_MAGNITUDE / mass.mass;
    let velocity = throw_direction * throw_strength;

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

fn throw_dir3(forward: &Dir3, right: &Dir3) -> Dir3 {
    Dir3::new_unchecked(
        Quat::from_axis_angle(right.as_vec3(), 20.0f32.to_radians()) * forward.as_vec3(),
    )
}
