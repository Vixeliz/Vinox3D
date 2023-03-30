use bevy::prelude::*;

use crate::physics::simulate::move_no_collide;

use super::simulate::{move_and_collide, VoxelCollisionEvent};

pub struct PhysicsPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum PhysicsSet {
    Move,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((move_and_collide, move_no_collide))
            .add_event::<VoxelCollisionEvent>();
    }
}
