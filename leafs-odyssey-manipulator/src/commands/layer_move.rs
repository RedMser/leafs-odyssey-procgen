use leafs_odyssey_data::data::*;

use crate::{room_title_commands::{RoomCommand, RoomCommandContext}};

pub struct LayerMoveCommand;

impl RoomCommand for LayerMoveCommand {
    fn names(&self) -> &'static [&'static str] {
        &["movelayer"]
    }

    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String> {
        let src_index = context.pop_arg().ok_or("Layer source index is missing.")?.parse::<usize>().map_err(|_| "Layer source index is not an integer.")?;
        let dest_index = context.pop_arg().ok_or("Layer target index is missing.")?.parse::<usize>().map_err(|_| "Layer target index is not an integer.")?;

        for stem in &mut context.env.world.stems {
            match &mut stem.content {
                LOStemContent::TileMapEdit { id, layers, .. } => {
                    if *id == context.room_id {
                        let layer = layers.remove(src_index);
                        layers.insert(dest_index, layer);
                        break;
                    }
                },
                _ => {},
            }
        }

        Ok(())
    }
}