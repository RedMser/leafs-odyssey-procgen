use leafs_odyssey_data::data::*;

use crate::{room_title_commands::{RoomCommand, RoomCommandContext}, tile_parser::parse_string_to_items, utils::{conditional_tile_selection, write_tiles}};

pub struct ReplaceCommand;

// TODO: could be neat to do a "real" find-and-replace command: if condition C is met, replace tile X with tile Y
impl RoomCommand for ReplaceCommand {
    fn names(&self) -> &'static [&'static str] {
        &["replace", "find", "search"]
    }

    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String> {
        let predicate = context.pop_arg().ok_or("Condition is missing.")?;
        let predicate = parse_string_to_items(&predicate)?;
        let predicate = predicate.into_iter().map(|item| LOTile::from(&item)).collect::<Vec<_>>();
        let tiles = context.pop_arg().ok_or("Tile is missing.")?;
        let tiles = parse_string_to_items(&tiles)?;
        let tiles = tiles.into_iter().map(|item| LOTile::from(&item)).collect::<Vec<_>>();

        for stem in &mut context.env.world.stems {
            match &mut stem.content {
                LOStemContent::TileMapEdit { id, layers, .. } => {
                    if *id == context.room_id {
                        write_tiles(tiles, layers, |tilemap| conditional_tile_selection(&tilemap, &predicate).expect("TODO error handling gone wrong"))?;
                        break;
                    }
                },
                _ => {},
            }
        }

        Ok(())
    }
}