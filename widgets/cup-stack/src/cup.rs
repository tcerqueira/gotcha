use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::game::TABLE_DIM;

pub struct CupsPlugin;

impl Plugin for CupsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TargetsLeft(6));
        app.add_systems(
            PostUpdate,
            (
                update_targets_left.after(PhysicsSet::Writeback),
                // debug_targets_left,
            )
                .chain(),
        );
    }
}

#[derive(Resource)]
pub struct TargetsLeft(pub u8);

#[derive(Component)]
pub struct Cup;

pub const CUP_HEIGHT: f32 = 0.10;
pub const CUP_RADIUS: f32 = 0.04;

#[derive(Bundle)]
pub struct CupBundle {
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub mass: ColliderMassProperties,
    pub damping: Damping,
    pub _tag: Cup,
}

impl Default for CupBundle {
    fn default() -> Self {
        Self {
            mesh: default(),
            material: default(),
            transform: default(),
            rigid_body: default(),
            collider: Collider::cylinder(CUP_HEIGHT / 2., CUP_RADIUS),
            damping: Damping { linear_damping: 0.3, angular_damping: 0.3 },
            mass: ColliderMassProperties::Density(0.1),
            _tag: Cup,
        }
    }
}

fn update_targets_left(cups: Query<&Transform, With<Cup>>, mut targets_left: ResMut<TargetsLeft>) {
    let above_table_count = cups
        .iter()
        .filter(|cup| cup.translation.y > TABLE_DIM.y)
        .count() as u8;
    if above_table_count != targets_left.0 {
        targets_left.0 = above_table_count;
    }
}

#[allow(dead_code)]
fn debug_targets_left(targets_left: Res<TargetsLeft>) {
    if targets_left.is_changed() {
        debug!("targets left = {}", targets_left.0);
    }
}
