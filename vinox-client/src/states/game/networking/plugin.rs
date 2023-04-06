use bevy::prelude::*;
use vinox_common::networking::protocol::EntityBuffer;

use crate::states::components::GameState;

use super::{
    components::{ChatMessages, ClientLobby, NetworkMapping},
    syncing::{
        client_naive_inventory, client_send_naive_position, get_id, get_messages, lerp_new_location,
    },
};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClientLobby::default())
            .insert_resource(NetworkMapping::default())
            .insert_resource(EntityBuffer::default())
            .insert_resource(ChatMessages::default())
            .add_system(
                client_send_naive_position
                    .in_schedule(CoreSchedule::FixedUpdate)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_systems(
                (
                    get_messages,
                    lerp_new_location,
                    get_id,
                    client_naive_inventory,
                )
                    .in_set(OnUpdate(GameState::Game)),
            );
    }
}
