use leafs_odyssey_data::data::*;

use crate::{room_title_commands::{RoomCommand, RoomCommandContext}, tile_parser::parse_string_to_items};

pub struct LayerAddCommand;

impl RoomCommand for LayerAddCommand {
    fn names(&self) -> &'static [&'static str] {
        &["addlayer", "newlayer"]
    }

    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String> {
        let count = context.pop_arg().ok_or("Layer count is missing.")?.parse::<usize>().map_err(|_| "Layer count is not an integer.")?;
        if count <= 0 {
            return Err(format!("Expected a positive layer count, but got {}", count));
        }
        let tiles = context.pop_arg().ok_or("Tile is missing.")?;
        let tiles = parse_string_to_items(&tiles)?;
        if tiles.len() != 1 {
            return Err(format!("Expected a single tile, but got {}", tiles.len()));
        }
        let tiles = tiles.into_iter().map(|item| LOTile::from(&item)).next().unwrap();

        for stem in &mut context.env.world.stems {
            match &mut stem.content {
                LOStemContent::TileMapEdit { id, layers, .. } => {
                    if *id == context.room_id {
                        for _ in 0..count {
                            let layer = LOLayer { width: 24, height: 16, tiles: vec![tiles.clone(); 24*16] };
                            layers.push(layer);
                        }
                        break;
                    }
                },
                _ => {},
            }
        }

        Ok(())
    }
}