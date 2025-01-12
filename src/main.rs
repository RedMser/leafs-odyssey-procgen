#![allow(dead_code)]
#![allow(static_mut_refs)]
#![feature(file_buffered)]
#![feature(cfg_match)]

pub mod leafs_odyssey;
mod terrain_gen;
mod null_sink;
mod image_to_tiles;
mod import_godot_string;

use std::{error::Error, fs, path::Path};

use import_godot_string::import_godot_string;
use leafs_odyssey::{builder::*, data::*, io::get_worlds_folder};
use terrain_gen::{generate_terrain, terrain_to_rooms};
use image_to_tiles::image_to_tiles;

fn main() -> Result<(), Box<dyn Error>> {
    let mut world = World::new()
        .with_metadata("What Go On", "This is cool stuff!")
        .with_identity(&random_guid_segment(), Author::new("RedMser", "65D50024-C5A59B05"));
    
    //import_godot_string("./lop.txt", &mut world)?;

    world.room_width = 4;
    world.room_height = 4;

    for x in 0..32 {
        for y in 0..24 {
            let music = if x % 2 == 0 {
                if y % 2 == 0 {
                    LOMusic::Aqua
                } else {
                    LOMusic::Dust
                }
            } else {
                if y % 2 == 0 {
                    LOMusic::Superfluid
                } else {
                    LOMusic::Rapture
                }
            };
            let mut room = world.new_room((x, y, 0))
                .with_metadata(&format!("{x},{y}"), music);
            room.tilemap = Tilemap::new(24, 16);
            let tile = if x % 2 == 0 {
                if y % 2 == 0 {
                    LOTile::Gravel
                } else {
                    LOTile::Sand
                }
            } else {
                if y % 2 == 0 {
                    LOTile::RedFlowers
                } else {
                    LOTile::Dirt
                }
            };
            room.tilemap.write(&tile, &room.tilemap.select_all());
            room.tilemap.write(&LOTile::BombBug { direction: LODirection::Down }, &room.tilemap.select().add(8 + (x % 2) as usize, 8 + (y % 2) as usize));
            if x == 0 && y == 0 {
                room.tilemap.write(&LOTile::StartPoint { direction: LODirection::Down }, &room.tilemap.select().add(0, 0));
            }
            world.rooms.push(room);
        }
    }

    unsafe {
        let mut fa = std::fs::File::create(
            Path::new(&get_worlds_folder()?).join("procgenWTF.world")
        )?;

        let mut world = LOWorld::try_from(world)?;
        world.write_world(&mut fa)?;
    }

    Ok(())
}

fn old_main() -> Result<(), Box<dyn Error>> {
    //let rooms = terrain_to_rooms(generate_terrain(), (8, 8));
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
                name: "Generat".into(),
                room_info: room_coords,
                description: "(no description provided)".into(),
                author: "RedMser".into(),
                guid_world: parse_single_guid("444DEC65")?,
                guid_author1: parse_single_guid("65D50024")?,
                guid_author2: parse_single_guid("C5A59B05")?,
                world_revision: 1,
                unknown6: 0,
            }),
            ..rooms.as_row_major()
                .into_iter()
                .map(|room| LOStem::from_content(room)),
        ],
    );

    unsafe {
        let mut fa = std::fs::File::create(
            Path::new(&get_worlds_folder()?).join("procgenGENERATED.world")
        )?;
        world.write_world(&mut fa)?;
    }

    Ok(())
}
