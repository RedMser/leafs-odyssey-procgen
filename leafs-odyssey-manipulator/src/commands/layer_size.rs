use leafs_odyssey_data::data::*;

use crate::{room_title_commands::{RoomCommand, RoomCommandContext}};

pub struct LayerSizeCommand;

impl RoomCommand for LayerSizeCommand {
    fn names(&self) -> &'static [&'static str] {
        &["sizelayer", "resizelayer"]
    }

    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String> {
        let index = context.pop_arg().ok_or("Layer index is missing.")?.parse::<usize>().map_err(|_| "Layer index is not an integer.")?;
        let new_width = context.pop_arg().ok_or("Width is missing.")?.parse::<u16>().map_err(|_| "Width is not an integer.")?;
        let new_height = context.pop_arg().ok_or("Height is missing.")?.parse::<u16>().map_err(|_| "Height is not an integer.")?;

        for stem in &mut context.env.world.stems {
            match &mut stem.content {
                LOStemContent::TileMapEdit { id, layers, .. } => {
                    if *id == context.room_id {
                        let layer = &mut layers[index];
                        layer.width = new_width;
                        layer.height = new_height;
                        break;
                    }
                },
                _ => {},
            }
        }

        Ok(())
    }
}