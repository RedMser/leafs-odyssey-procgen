use leafs_odyssey_data::data::*;

use crate::{room_title_commands::{RoomCommand, RoomCommandContext}};

pub struct LayerRemoveCommand;

impl RoomCommand for LayerRemoveCommand {
    fn names(&self) -> &'static [&'static str] {
        &["removelayer", "deletelayer"]
    }

    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String> {
        let index = context.pop_arg().ok_or("Layer index is missing.")?.parse::<usize>().map_err(|_| "Layer index is not an integer.")?;

        for stem in &mut context.env.world.stems {
            match &mut stem.content {
                LOStemContent::TileMapEdit { id, layers, .. } => {
                    if *id == context.room_id {
                        layers.remove(index);
                        break;
                    }
                },
                _ => {},
            }
        }

        Ok(())
    }
}