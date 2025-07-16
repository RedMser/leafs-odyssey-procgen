use leafs_odyssey_data::data::*;

use crate::{room_title_commands::{RoomCommand, RoomCommandContext}, tile_parser::parse_string_to_items, utils::write_tiles};

pub struct TileCommand;

impl RoomCommand for TileCommand {
    fn names(&self) -> &'static [&'static str] {
        &["tile", "tiles"]
    }

    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String> {
        let x = context.pop_arg().ok_or("X position is missing.")?.parse::<usize>().map_err(|_| "X position is not an integer.")?;
        let y = context.pop_arg().ok_or("Y position is missing.")?.parse::<usize>().map_err(|_| "Y position is not an integer.")?;
        let tiles = context.pop_arg().ok_or("Tile is missing.")?;
        let tiles = parse_string_to_items(&tiles)?;
        let tiles = tiles.into_iter().map(|item| LOTile::from(&item)).collect::<Vec<_>>();

        for stem in &mut context.env.world.stems {
            match &mut stem.content {
                LOStemContent::TileMapEdit { id, layers, .. } => {
                    if *id == context.room_id {
                        write_tiles(tiles, layers, |tilemap| tilemap.select().add(x-1, y-1))?;
                        break;
                    }
                },
                _ => {},
            }
        }

        Ok(())
    }
}