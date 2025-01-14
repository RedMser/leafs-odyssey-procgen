#![feature(file_buffered)]

mod image_to_tiles;

use std::{error::Error, path::Path};

use leafs_odyssey_data::{data::*, io::get_worlds_folder};

use crate::image_to_tiles::*;

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: read cmdline args
    let rooms = image_to_tiles("C:\\Users\\VRLand\\Desktop\\leaf.png");

    let room_coords = rooms.indices_row_major()
        .enumerate()
        .map(|(i, (y, x))| LORoomInfo {
            id: (i+1) as u32,
            width: 24,
            height: 16,
            x_position: (x * 24) as i16,
            y_position: (y * 16) as i16,
            z_position: 0,
        })
        .collect::<Vec<_>>();

    let world_guids = random_guid(3);
    let world_guids = parse_guid(&world_guids).unwrap();

    let mut world = LOWorld::new(
        LOZone {
            stem_offset: 0,
            stem_length: 0,
            rooms: vec![
                LORoom {
                    stem_offset: 0,
                    stem_length: 0,
                }; room_coords.len()
            ],
        },
        velcro::vec![
            LOStem::from_content(LOStemContent::TileZoneMap {
                name: "Image Art".into(),
                room_info: room_coords,
                description: "(no description provided)".into(),
                author: "Rust".into(),
                guid_world: world_guids[0],
                guid_author1: world_guids[1],
                guid_author2: world_guids[2],
                world_revision: 1,
                _unknown4: 0,
            }),
            ..rooms.as_row_major()
                .into_iter()
                .map(|room| LOStem::from_content(room)),
        ],
    );

    unsafe {
        let mut fa = std::fs::File::create(
            Path::new(&get_worlds_folder()?).join("generated_image_art.world")
        )?;
        world.write_world(&mut fa)?;
    }

    Ok(())
}
