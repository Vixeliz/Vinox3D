use crate::states::components::GameState;

use super::{
    crafting::crafting_ui,
    debug::debug,
    dropdown::{create_ui, ConsoleOpen, Toast},
    inventory::{inventory, status_bar, CurrentItemsHeld, Holding},
};
use bevy::prelude::*;

pub struct UiPlugin;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct InUi(pub bool);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConsoleOpen(false))
            .insert_resource(CurrentItemsHeld::default())
            .insert_resource(Holding(false))
            .insert_resource(InUi(false))
            .insert_resource(Toast::default())
            .add_systems(
                (create_ui, status_bar, inventory, crafting_ui, debug)
                    .chain()
                    .in_set(OnUpdate(GameState::Game)),
            );
    }
}
