use std::{
    cmp::{max, min},
    f32::{INFINITY, NEG_INFINITY},
    fmt,
};

use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb, utils::FloatOrd};

use crate::world::chunks::{
    ecs::CurrentChunks,
    positions::{ChunkPos, VoxelPos},
    storage::{BlockData, BlockTable, ChunkData},
};

const EPSILON: f32 = 0.0001;
const MARGIN: Vec3A = Vec3A::new(EPSILON, EPSILON, EPSILON);

#[derive(Clone)]
pub struct CollisionInfo {
    pub collision_aabb: Aabb,
    pub normal: Vec3,
    pub dist: f32,
}

impl std::fmt::Display for CollisionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Collision @ {} norm {} dist {}",
            self.collision_aabb.center.floor(),
            self.normal,
            self.dist
        )
    }
}

pub fn aabb_intersects_world(
    aabb: &Aabb,
    chunks: &Query<&ChunkData>,
    current_chunks: &CurrentChunks,
    block_table: &BlockTable,
) -> bool {
    let area_to_check = (
        (-aabb.half_extents).floor().as_ivec3() + IVec3::NEG_ONE,
        (aabb.half_extents).ceil().as_ivec3() + IVec3::ONE,
    );
    for x in area_to_check.0.x..=area_to_check.1.x {
        for y in area_to_check.0.y..=area_to_check.1.y {
            for z in area_to_check.0.z..=area_to_check.1.z {
                let (check_block_cpos, check_chunk_pos) = VoxelPos::from_world(
                    Vec3::from(aabb.center) + Vec3::new(x as f32, y as f32, z as f32),
                )
                .to_offsets();
                if let Some(chunk_entity) = current_chunks.get_entity(check_chunk_pos) {
                    if let Ok(chunk) = chunks.get(chunk_entity) {
                        let block_data: BlockData =
                            chunk.get(check_block_cpos.x, check_block_cpos.y, check_block_cpos.z);
                        let voxel_pos = VoxelPos::from_offsets(check_block_cpos, check_chunk_pos);
                        if !block_data.is_empty(block_table) {
                            let block_aabb = Aabb {
                                center: voxel_pos.as_vec3a() + Vec3A::new(0.5, 0.5, 0.5),
                                half_extents: Vec3A::new(0.5, 0.5, 0.5),
                            };
                            if aabbs_intersect(&aabb, &block_aabb) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn get_aabb_world_intersectors(
    aabb: &Aabb,
    chunks: &Query<&ChunkData>,
    current_chunks: &CurrentChunks,
    block_table: &BlockTable,
) -> Vec<VoxelPos> {
    let mut intersectors: Vec<VoxelPos> = Vec::new();
    let area_to_check = (
        (-aabb.half_extents).floor().as_ivec3() + IVec3::NEG_ONE,
        (aabb.half_extents).ceil().as_ivec3() + IVec3::ONE,
    );
    for x in area_to_check.0.x..=area_to_check.1.x {
        for y in area_to_check.0.y..=area_to_check.1.y {
            for z in area_to_check.0.z..=area_to_check.1.z {
                let (check_block_cpos, check_chunk_pos) = VoxelPos::from_world(
                    Vec3::from(aabb.center) + Vec3::new(x as f32, y as f32, z as f32),
                )
                .to_offsets();
                if let Some(chunk_entity) = current_chunks.get_entity(check_chunk_pos) {
                    if let Ok(chunk) = chunks.get(chunk_entity) {
                        let block_data: BlockData =
                            chunk.get(check_block_cpos.x, check_block_cpos.y, check_block_cpos.z);
                        let voxel_pos = VoxelPos::from_offsets(check_block_cpos, check_chunk_pos);
                        if !block_data.is_empty(block_table) {
                            let block_aabb = Aabb {
                                center: voxel_pos.as_vec3a() + Vec3A::new(0.5, 0.5, 0.5),
                                half_extents: Vec3A::new(0.5, 0.5, 0.5),
                            };
                            if aabbs_intersect(&aabb, &block_aabb) {
                                intersectors.push(voxel_pos);
                                let yint = block_aabb.max().y <= aabb.min().y;
                                println!(
                                    "Intersecting with {} which is {}, chunk_pos {}, chunk_offsets, {}, voxel_pos {}, block_aabb {:?}, player_aabb {aabb:?}, {} <= {} ? {yint}, diff {}, eval {}",
                                    block_aabb.center.as_ivec3(),
                                    block_data.name,
                                    check_chunk_pos,
                                    check_block_cpos,
                                    voxel_pos,
                                    block_aabb,
                                    block_aabb.max().y,
                                    aabb.min().y,
                                    block_aabb.max().y - aabb.min().y,
                                    (block_aabb.max().y - aabb.min().y).abs() < 0.001,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    intersectors
}
pub fn aabb_vs_world(
    aabb: &Aabb,
    chunks: &Query<&ChunkData>,
    velocity: Vec3,
    current_chunks: &CurrentChunks,
    block_table: &BlockTable,
) -> Option<Vec<CollisionInfo>> {
    let mut collisions: Vec<CollisionInfo> = Vec::new();
    let area_to_check = (
        (-aabb.half_extents)
            .min(-aabb.half_extents + Vec3A::from(velocity))
            .floor()
            .as_ivec3()
            + IVec3::NEG_ONE,
        (aabb.half_extents)
            .max(aabb.half_extents + Vec3A::from(velocity))
            .ceil()
            .as_ivec3()
            + IVec3::ONE,
    );
    for x in area_to_check.0.x..=area_to_check.1.x {
        for y in area_to_check.0.y..=area_to_check.1.y {
            for z in area_to_check.0.z..=area_to_check.1.z {
                let (check_block_cpos, check_chunk_pos) = VoxelPos::from_world(
                    Vec3::from(aabb.center) + Vec3::new(x as f32, y as f32, z as f32),
                )
                .to_offsets();
                if let Some(chunk_entity) = current_chunks.get_entity(check_chunk_pos) {
                    if let Ok(chunk) = chunks.get(chunk_entity) {
                        let block_data: BlockData =
                            chunk.get(check_block_cpos.x, check_block_cpos.y, check_block_cpos.z);
                        let voxel_pos = VoxelPos::from_offsets(check_block_cpos, check_chunk_pos);
                        if !block_data.is_empty(block_table) {
                            let block_aabb = Aabb {
                                center: voxel_pos.as_vec3a() + Vec3A::new(0.5, 0.5, 0.5),
                                half_extents: Vec3A::new(0.5, 0.5, 0.5),
                            };
                            let col = get_collision_info(aabb, &block_aabb, &velocity);
                            if let Some(c) = col {
                                collisions.push(c);
                            }
                        }
                    }
                }
            }
        }
    }
    if !collisions.is_empty() {
        return Some(collisions);
    }
    None
}

#[inline]
// Checks whether AABBs are at least within MARGIN distance of each other
// (Couldn't use exact comparison because of floating point imprecision)
pub fn aabbs_intersect(a: &Aabb, b: &Aabb) -> bool {
    let amin = a.min() - MARGIN;
    let amax = a.max() + MARGIN;
    let bmin = b.min() - MARGIN;
    let bmax = b.max() + MARGIN;
    !(amin.x > bmax.x
        || bmin.x > amax.x
        || amin.y > bmax.y
        || bmin.y > amax.y
        || amin.z > bmax.z
        || bmin.z > amax.z)
}

pub fn get_collision_info(a: &Aabb, b: &Aabb, a_velocity: &Vec3) -> Option<CollisionInfo> {
    // inv_enter is distance between closest sides
    // inv_exit is distance between farthest sides
    let mut inv_enter = Vec3::ZERO;
    let mut inv_exit = Vec3::ZERO;

    if a_velocity.x > 0.0 {
        inv_enter.x = b.min().x - a.max().x;
        inv_exit.x = b.max().x - a.min().x;
    } else {
        inv_enter.x = b.max().x - a.min().x;
        inv_exit.x = b.min().x - a.max().x;
    }

    if a_velocity.y > 0.0 {
        inv_enter.y = b.min().y - a.max().y;
        inv_exit.y = b.max().y - a.min().y;
    } else {
        inv_enter.y = b.max().y - a.min().y;
        inv_exit.y = b.min().y - a.max().y;
    }

    if a_velocity.z > 0.0 {
        inv_enter.z = b.min().z - a.max().z;
        inv_exit.z = b.max().z - a.min().z;
    } else {
        inv_enter.z = b.max().z - a.min().z;
        inv_exit.z = b.min().z - a.max().z;
    }

    // enter is amt time to intersect based on current a_velocity
    // exit is amt time to go past it based on current a_velocity
    let mut enter = Vec3::ZERO;
    let mut exit = Vec3::ZERO;

    if a_velocity.x == 0.0 {
        if inv_enter.x.signum() == inv_exit.x.signum() {
            return None; // Impossible to collide because not already within it on this axis
        }
        enter.x = NEG_INFINITY;
        exit.x = INFINITY;
    } else {
        enter.x = inv_enter.x / a_velocity.x;
        exit.x = inv_exit.x / a_velocity.x;
    }
    if a_velocity.y == 0.0 {
        if inv_enter.y.signum() == inv_exit.y.signum() {
            return None; // Impossible to collide because not already within it on this axis
        }
        enter.y = NEG_INFINITY;
        exit.y = INFINITY;
    } else {
        enter.y = inv_enter.y / a_velocity.y;
        exit.y = inv_exit.y / a_velocity.y;
    }
    if a_velocity.z == 0.0 {
        if inv_enter.z.signum() == inv_exit.z.signum() {
            return None; // Impossible to collide because not alreadz within it on this axis
        }
        enter.z = NEG_INFINITY;
        exit.z = INFINITY;
    } else {
        enter.z = inv_enter.z / a_velocity.z;
        exit.z = inv_exit.z / a_velocity.z;
    }

    let mut normal = Vec3::ZERO;

    let enter_time = max(max(FloatOrd(enter.x), FloatOrd(enter.y)), FloatOrd(enter.z));
    let exit_time = min(min(FloatOrd(exit.x), FloatOrd(exit.y)), FloatOrd(exit.z));
    if enter_time > exit_time
        || (enter.x < -EPSILON && enter.y < -EPSILON && enter.z < -EPSILON)
        || enter.x > 1.0
        || enter.y > 1.0
        || enter.z > 1.0
    {
        // No collision happens here this frame
        None
    } else {
        // This might be a collision this frame
        let dist: f32;
        // if inv_enter.x == 0.0 && inv_enter.z == 0.0 {
        //     if a_velocity.x.abs() > a_velocity.z.abs() {
        //         normal.z = if a_velocity.z < 0.0 { 1.0 } else { -1.0 };
        //         dist = inv_enter.z.abs();
        //     } else {
        //         normal.x = if a_velocity.x < 0.0 { 1.0 } else { -1.0 };
        //         dist = inv_enter.x.abs();
        //     }
        // } else if inv_enter.x == 0.0 && inv_enter.y == 0.0 {
        //     if a_velocity.x.abs() > a_velocity.y.abs() {
        //         normal.y = if a_velocity.y < 0.0 { 1.0 } else { -1.0 };
        //         dist = inv_enter.y.abs();
        //     } else {
        //         normal.x = if a_velocity.x < 0.0 { 1.0 } else { -1.0 };
        //         dist = inv_enter.x.abs();
        //     }
        // } else if inv_enter.y == 0.0 && inv_enter.z == 0.0 {
        //     if a_velocity.y.abs() > a_velocity.z.abs() {
        //         normal.z = if a_velocity.z < 0.0 { 1.0 } else { -1.0 };
        //         dist = inv_enter.z.abs();
        //     } else {
        //         normal.y = if a_velocity.y < 0.0 { 1.0 } else { -1.0 };
        //         dist = inv_enter.y.abs();
        //     }
        if inv_enter.x == 0.0 && a_velocity.x == 0.0 {
            normal.x = if inv_exit.x < 0.0 { 1.0 } else { -1.0 };
            dist = inv_enter.x.abs();
        } else if inv_enter.y == 0.0 && a_velocity.y == 0.0 {
            normal.y = if inv_exit.y < 0.0 { 1.0 } else { -1.0 };
            dist = inv_enter.y.abs();
        } else if inv_enter.z == 0.0 && a_velocity.z == 0.0 {
            normal.z = if inv_exit.z < 0.0 { 1.0 } else { -1.0 };
            dist = inv_enter.z.abs();
        } else if enter.x == 0.0 {
            normal.x = if inv_exit.x < 0.0 { 1.0 } else { -1.0 };
            dist = inv_enter.x.abs();
        } else if enter.y == 0.0 {
            normal.y = if inv_exit.y < 0.0 { 1.0 } else { -1.0 };
            dist = inv_enter.y.abs();
        } else if enter.z == 0.0 {
            normal.z = if inv_exit.z < 0.0 { 1.0 } else { -1.0 };
            dist = inv_enter.z.abs();
        } else if enter_time.0 == enter.x {
            normal.x = if inv_exit.x < 0.0 { 1.0 } else { -1.0 };
            dist = inv_enter.x.abs();
        } else if enter_time.0 == enter.y {
            normal.y = if inv_exit.y < 0.0 { 1.0 } else { -1.0 };
            dist = inv_enter.y.abs();
        } else {
            normal.z = if inv_exit.z < 0.0 { 1.0 } else { -1.0 };
            dist = inv_enter.z.abs();
        }
        Some(CollisionInfo {
            collision_aabb: *b,
            normal,
            dist,
        })
    }
}
