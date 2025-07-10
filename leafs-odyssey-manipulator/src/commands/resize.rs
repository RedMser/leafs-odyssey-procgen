use leafs_odyssey_data::data::*;

use crate::room_title_commands::{RoomCommand, RoomCommandContext};

pub struct ResizeCommand;

impl RoomCommand for ResizeCommand {
    fn names(&self) -> &'static [&'static str] {
        &["resize", "size"]
    }

    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String> {
        let new_width = context.pop_arg().ok_or("Width is missing.")?.parse::<u16>().map_err(|_| "Width is not an integer.")?;
        let new_height = context.pop_arg().ok_or("Height is missing.")?.parse::<u16>().map_err(|_| "Height is not an integer.")?;

        // Update size in world
        let room_info = context.get_room_info_mut();
        room_info.width = new_width;
        room_info.height = new_height;

        // Update size of room (keep layer size same, since there's little benefit to not have it be 24x16)
        for stem in &mut context.world.stems {
            match &mut stem.content {
                LOStemContent::TileMapEdit { width, height, .. } => {
                    *width = new_width;
                    *height = new_height;
                },
                _ => {},
            }
        }

        Ok(())
    }
}