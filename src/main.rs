#![allow(dead_code)]
#![allow(static_mut_refs)]

pub mod leafs_odyssey_data;
mod null_sink;

use std::error::Error;

use leafs_odyssey_data::*;

fn main() -> Result<(), Box<dyn Error>> {
    let mut world = LOWorld::new(
        LOZone {
            stem_offset: 0,
            stem_length: 0,
            rooms: vec![
                LORoom {
                    stem_offset: 0,
                    stem_length: 0,
                },
            ],
        },
        vec![
            LOStem::from_content(LOStemContent::TileZoneMap {
                name: "Generat".into(),
                room_info: vec![
                    LORoomInfo {
                        id: 1,
                        width: 24,
                        height: 16,
                        x_position: 0,
                        y_position: 0,
                        z_position: 0,
                    },
                ],
                description: "(no description provided)".into(),
                author: "RedMser".into(),
                guid_world: parse_single_guid("444DEC65")?,
                guid_author1: parse_single_guid("65D50024")?,
                guid_author2: parse_single_guid("C5A59B05")?,
                world_revision: 1,
                unknown5: 0,
                unknown6: 0,
            }),
            LOStem::from_content(LOStemContent::TileMapEdit {
                name: "untitled".into(),
                width: 24,
                height: 16,
                layers: vec![
                    // Floor
                    LOLayer {
                        width: 24,
                        height: 16,
                        tiles: vec![LOTile::Grass; 24*16],
                    },
                    LOLayer {
                        width: 24,
                        height: 16,
                        tiles: vec![LOTile::None; 24*16],
                    },
                    LOLayer {
                        width: 24,
                        height: 16,
                        tiles: vec![LOTile::None; 24*16],
                    },
                    // SpawnPoint is in here
                    LOLayer {
                        width: 24,
                        height: 16,
                        tiles: vec![LOTile::None; 24*16]
                            .iter()
                            .enumerate()
                            .map(|(i, tile)| {
                                if i == 40 || i == 41 {
                                    LOTile::StartPoint {
                                        direction: LODirection::Down,
                                    }
                                } else {
                                    tile.clone()
                                }
                            })
                            .collect(),
                    },
                    // Sign is in here
                    LOLayer {
                        width: 24,
                        height: 16,
                        tiles: vec![LOTile::None; 24*16],
                    },
                ],
                music: LOMusic::None,
                revision: 1,
            }),
        ],
    );

    unsafe {
        let mut fa = std::fs::File::create(
            "C:\\Users\\VRLand\\AppData\\Roaming\\leafsodyssey_worlds\\procgenGENERATED.world",
        )?;
        world.write_world(&mut fa)?;
    }

    Ok(())
}
