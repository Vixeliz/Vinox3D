use std::collections::HashSet;
use tokio::sync::mpsc::{Receiver, Sender};

use bevy::{prelude::*, render::primitives::Aabb, tasks::AsyncComputeTaskPool, utils::FloatOrd};
use bevy_tweening::*;
use vinox_common::world::chunks::{
    ecs::{
        update_chunk_lights, update_priority_chunk_lights, ChunkCell, ChunkManager, ChunkUpdate,
        CurrentChunks, RemoveChunk, SimulationRadius, ViewRadius,
    },
    positions::{voxel_to_global_voxel, world_to_chunk, ChunkPos},
    storage::{BlockData, BlockTable, ChunkData, RawChunk, HORIZONTAL_DISTANCE, VERTICAL_DISTANCE},
};

use crate::states::{
    components::GameState,
    game::rendering::meshing::{build_mesh, priority_mesh},
};

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Default, Resource)]
pub struct PlayerChunk {
    pub chunk_pos: IVec3,
}

#[derive(Default, Resource)]
pub struct PlayerBlock {
    pub pos: IVec3,
}

#[derive(Default, Resource, Debug)]
pub struct PlayerTargetedBlock {
    pub block: Option<BlockData>,
    pub pos: Option<IVec3>,
}

#[derive(Default, Debug)]
pub enum VoxelAxis {
    #[default]
    North,
    South,
    West,
    East,
}

#[derive(Default, Resource, Debug, Deref, DerefMut)]
pub struct PlayerDirection(pub VoxelAxis);

pub struct CreateChunkEvent {
    pub pos: IVec3,
    pub raw_chunk: RawChunk,
}

pub struct SetBlockEvent {
    pub chunk_pos: IVec3,
    pub voxel_pos: UVec3,
    pub block_type: BlockData,
}
pub struct UpdateChunkEvent {
    pub pos: IVec3,
}

#[derive(Default, Resource)]
pub struct ChunkQueue {
    pub mesh: Vec<(IVec3, RawChunk)>,
    pub remove: HashSet<IVec3>,
}

impl PlayerChunk {
    pub fn is_in_radius(&self, pos: IVec3, view_radius: &ViewRadius) -> bool {
        !(pos.x > (view_radius.horizontal + self.chunk_pos.x)
            || pos.x < (-view_radius.horizontal + self.chunk_pos.x)
            || pos.z > (view_radius.horizontal + self.chunk_pos.z)
            || pos.z < (-view_radius.horizontal + self.chunk_pos.z)
            || pos.y > (view_radius.vertical + self.chunk_pos.y)
            || pos.y < (-view_radius.vertical + self.chunk_pos.y))
    }
}

pub fn update_player_location(
    player_query: Query<&Aabb, With<ControlledPlayer>>,
    mut player_chunk: ResMut<PlayerChunk>,
    mut player_block: ResMut<PlayerBlock>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let new_chunk = world_to_chunk(player_transform.center.into());
        if new_chunk != player_chunk.chunk_pos {
            player_chunk.chunk_pos = new_chunk;
        }
        if player_transform.center.floor().as_ivec3() != player_block.pos {
            player_block.pos = player_transform.center.floor().as_ivec3();
        }
    }
}
pub fn update_player_direction(
    mut player_direction: ResMut<PlayerDirection>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
) {
    if let Ok(camera) = camera_query.get_single() {
        let forward = camera.forward();

        let east_dot = forward.dot(Vec3::X);
        let west_dot = forward.dot(Vec3::NEG_X);
        let north_dot = forward.dot(Vec3::Z);
        let south_dot = forward.dot(Vec3::NEG_Z);
        let numbers = [east_dot, west_dot, north_dot, south_dot];
        let closest = numbers.iter().max_by_key(|&num| FloatOrd(*num)).unwrap();
        **player_direction = if *closest == east_dot {
            VoxelAxis::East
        } else if *closest == west_dot {
            VoxelAxis::West
        } else if *closest == north_dot {
            VoxelAxis::North
        } else {
            VoxelAxis::South
        };
    }
}
pub fn unload_chunks(
    mut commands: Commands,
    remove_chunks: Query<(&ChunkPos, Entity), With<RemoveChunk>>,
    mut current_chunks: ResMut<CurrentChunks>,
) {
    for (chunk, entity) in remove_chunks.iter() {
        current_chunks.remove_entity(*chunk).ok_or(0).ok();
        commands.entity(entity).despawn_recursive();

        // if current_chunks.get_entity(*chunk).is_some() {
        // let tween = Tween::new(
        //     EaseFunction::QuadraticInOut,
        //     Duration::from_secs(1),
        //     TransformPositionLens {
        //         end: Vec3::new(
        //             (chunk.x * (CHUNK_SIZE) as i32) as f32,
        //             ((chunk.y * (CHUNK_SIZE) as i32) as f32) - CHUNK_SIZE as f32,
        //             (chunk.z * (CHUNK_SIZE) as i32) as f32,
        //         ),

        //         start: Vec3::new(
        //             (chunk.x * (CHUNK_SIZE) as i32) as f32,
        //             (chunk.y * (CHUNK_SIZE) as i32) as f32,
        //             (chunk.z * (CHUNK_SIZE) as i32) as f32,
        //         ),
        //     },
        // )
        // .with_repeat_count(RepeatCount::Finite(1))
        // .with_completed_event(0);
        // commands.entity(chunk_entity).insert(Animator::new(tween));
        // commands.entity(chunk_entity).remove::<RemoveChunk>();
        // commands.entity(chunk_entity).remove::<ChunkData>();
        // current_chunks.remove_entity(*chunk).ok_or(0).ok();
        // // }
        // commands
        //     .entity(current_chunks.get_entity(*chunk).unwrap())
        //     .despawn_recursive();
    }
}

#[derive(Resource)]
pub struct LightingChannel {
    pub tx: Sender<(ChunkData, IVec3)>,
    pub rx: Receiver<(ChunkData, IVec3)>,
}

impl Default for LightingChannel {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        Self { tx, rx }
    }
}

pub fn destroy_chunks(mut commands: Commands, mut query_event: EventReader<TweenCompleted>) {
    for evt in query_event.iter() {
        if evt.user_data == 0 {
            commands.entity(evt.entity).despawn_recursive();
        }
    }
}

pub fn clear_unloaded_chunks(
    mut commands: Commands,
    chunks: Query<(&ChunkPos, Entity)>,
    player_chunk: Res<PlayerChunk>,
    view_radius: Res<ViewRadius>,
) {
    for (chunk, entity) in chunks.iter() {
        if !player_chunk.is_in_radius(**chunk, &view_radius) {
            commands.entity(entity).insert(RemoveChunk);
        }
    }
}

#[allow(clippy::nonminimal_bool)]
pub fn receive_chunks(
    mut current_chunks: ResMut<CurrentChunks>,
    mut commands: Commands,
    mut event: EventReader<CreateChunkEvent>,
    player_chunk: Res<PlayerChunk>,
    view_radius: Res<ViewRadius>,
    block_table: Res<BlockTable>,
    mut light_channel: ResMut<LightingChannel>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    for evt in event.iter() {
        if player_chunk.is_in_radius(evt.pos, &view_radius)
            && current_chunks.get_entity(ChunkPos(evt.pos)).is_none()
        {
            let mut chunk_data = ChunkData::from_raw(evt.raw_chunk.clone());
            let cloned_sender = light_channel.tx.clone();
            let cloned_table = block_table.clone();
            let pos = evt.pos;
            task_pool
                .spawn(async move {
                    cloned_sender
                        .send((chunk_data.complete_relight(&cloned_table), pos))
                        .await
                        .ok();
                })
                .detach();
        }
    }
    while let Ok((chunk, pos)) = light_channel.rx.try_recv() {
        let chunk_id = commands
            .spawn(chunk.clone())
            .insert(ChunkPos(pos))
            .insert(ChunkCell::default())
            .id();

        current_chunks.insert_entity(ChunkPos(pos), chunk_id);

        // Don't mark chunks that won't create any blocks
        if !chunk.is_empty(&block_table) {
            commands.entity(chunk_id).insert(ChunkUpdate);
        }
    }
}

pub fn set_block(
    _commands: Commands,
    mut event: EventReader<SetBlockEvent>,
    // current_chunks: Res<CurrentChunks>,
    // mut chunks: Query<&mut ChunkData>,
    // block_table: Res<BlockTable>,
    mut chunk_manager: ChunkManager,
) {
    for evt in event.iter() {
        chunk_manager.set_block(
            voxel_to_global_voxel(evt.voxel_pos, evt.chunk_pos),
            evt.block_type.clone(),
        );
    }
}

pub fn should_update_chunks(player_chunk: Res<PlayerChunk>) -> bool {
    player_chunk.is_changed()
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentChunks::default())
            .insert_resource(ChunkQueue::default())
            .insert_resource(PlayerChunk::default())
            .insert_resource(PlayerBlock::default())
            .insert_resource(PlayerDirection::default())
            .insert_resource(PlayerTargetedBlock::default())
            .insert_resource(LightingChannel::default())
            .insert_resource(ViewRadius {
                horizontal: HORIZONTAL_DISTANCE as i32,
                vertical: VERTICAL_DISTANCE as i32,
            })
            .insert_resource(SimulationRadius {
                horizontal: 4,
                vertical: 4,
            })
            .add_system(update_player_location.in_set(OnUpdate(GameState::Game)))
            .add_system(update_player_direction.in_set(OnUpdate(GameState::Game)))
            .add_systems(
                (receive_chunks, set_block)
                    .chain()
                    .after(update_player_location)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                clear_unloaded_chunks
                    .after(receive_chunks)
                    .run_if(should_update_chunks)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                update_chunk_lights
                    .after(clear_unloaded_chunks)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                update_priority_chunk_lights
                    .after(clear_unloaded_chunks)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                build_mesh
                    .after(update_chunk_lights)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                priority_mesh
                    .after(update_chunk_lights)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                unload_chunks
                    .after(build_mesh)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                destroy_chunks
                    .after(unload_chunks)
                    // .after(build_mesh)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_event::<UpdateChunkEvent>()
            .add_event::<SetBlockEvent>()
            .add_event::<CreateChunkEvent>();
    }
}
