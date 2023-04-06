use bevy::prelude::*;
use bevy_quinnet::shared::ClientId;

#[derive(Resource, Deref, DerefMut)]
pub struct NetworkIP(pub String);

use serde::{Deserialize, Serialize};

use crate::{ecs::bundles::Inventory, world::chunks::storage::BlockData};

#[derive(Component)]
pub struct NetworkedEntity;

#[derive(Debug, Component, Default)]
pub struct Player {
    pub id: ClientId,
}

// Networking related
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct NetworkedEntities {
    pub entities: Vec<Entity>,
    pub translations: Vec<Vec3>,
    pub yaws: Vec<f32>,
    pub head_pitchs: Vec<f32>,
}

#[derive(Default, Resource)]
pub struct EntityBuffer {
    pub entities: [NetworkedEntities; 30],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    Position {
        player_pos: Vec3,
        yaw: f32,
        head_pitch: f32,
    },
    Interact {
        entity: Entity,
        attack: bool,
    },

    SentBlock {
        chunk_pos: IVec3,
        voxel_pos: [u8; 3],
        block_type: BlockData,
    },
    Join {
        user_name: String,
        password: String,
        id: ClientId,
    },
    Leave {
        id: ClientId,
    },
    ChatMessage {
        message: String,
    },
    Inventory {
        inventory: Box<Inventory>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    ChatMessage {
        user_name: String,
        message: String,
        id: u64,
    },
    ClientId {
        id: ClientId,
    },
    PlayerCreate {
        entity: Entity,
        id: ClientId,
        translation: Vec3,
        yaw: f32,
        head_pitch: f32,
        user_name: String,
        init: bool,
        inventory: Box<Inventory>,
    },
    PlayerRemove {
        id: ClientId,
    },
    SentBlock {
        chunk_pos: IVec3,
        voxel_pos: [u8; 3],
        block_type: BlockData,
    },
    NetworkedEntities {
        networked_entities: NetworkedEntities,
    },
    LevelData {
        chunk_data: Vec<u8>,
        pos: IVec3,
    },
}
