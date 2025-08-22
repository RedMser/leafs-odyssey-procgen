use leafs_odyssey_data::data::*;

use crate::{room_title_commands::{RoomCommand, RoomCommandContext}, tile_parser::{items_to_tiles, parse_string_to_items}, utils::write_tiles};

pub struct RectCommand;

impl RoomCommand for RectCommand {
    fn names(&self) -> &'static [&'static str] {
        &["rect", "rectangle", "box", "fill"]
    }

    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String> {
        let x1 = context.pop_arg().ok_or("X1 position is missing.")?.parse::<usize>().map_err(|_| "X1 position is not an integer.")?;
        if x1 <= 0 {
            return Err("X1 position must be > 0".to_owned());
        }
        let y1 = context.pop_arg().ok_or("Y1 position is missing.")?.parse::<usize>().map_err(|_| "Y1 position is not an integer.")?;
        if y1 <= 0 {
            return Err("Y1 position must be > 0".to_owned());
        }
        let x2 = context.pop_arg().ok_or("X2 position is missing.")?.parse::<usize>().map_err(|_| "X2 position is not an integer.")?;
        if x2 <= 0 {
            return Err("X2 position must be > 0".to_owned());
        }
        let y2 = context.pop_arg().ok_or("Y2 position is missing.")?.parse::<usize>().map_err(|_| "Y2 position is not an integer.")?;
        if y2 <= 0 {
            return Err("Y2 position must be > 0".to_owned());
        }
        let tiles = context.pop_arg().ok_or("Tile is missing.")?;
        let tiles = parse_string_to_items(&tiles)?;
        let tiles = items_to_tiles(tiles, &context.sign_text);

        for stem in &mut context.env.world.stems {
            match &mut stem.content {
                LOStemContent::TileMapEdit { id, layers, .. } => {
                    if *id == context.room_id {
                        write_tiles(tiles, layers, |tilemap| tilemap.select().add_rect(x1-1, y1-1, x1.abs_diff(x2)+1, y1.abs_diff(y2)+1))?;
                        break;
                    }
                },
                _ => {},
            }
        }

        Ok(())
    }
}