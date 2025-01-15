#![feature(file_buffered)]

mod image_to_tiles;

use std::{env, error::Error, path::Path, process::exit};

use leafs_odyssey_data::{data::*, io::get_worlds_folder};

use crate::image_to_tiles::*;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.len() > 2 {
        println!("USAGE: cargo run --bin image-art -- [INPUT] [OUTPUT]");
        println!("");
        println!("INPUT (required): path to an image file to load.");
        println!("See https://crates.io/crates/image for a full list of supported image formats.");
        println!("");
        println!("OUTPUT: file name of world, defaults to \"generated_image_art.world\"");
        println!("Will overwrite existing world file!");
        exit(1);
    }

    println!("Loading image \"{}\"...", &args[0]);

    let rooms = image_to_tiles(&args[0]);

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
                name: "[Generated] Image Art".into(),
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

    let world_name = "generated_image_art.world".into();
    let world_name = args.get(1).unwrap_or(&world_name);
    let world_path = if world_name.contains('/') || world_name.contains('\\') {
        Path::new(world_name)
    } else {
        &Path::new(&get_worlds_folder()?).join(world_name)
    };
    println!("Writing file \"{:?}\"...", world_path);

    unsafe {
        let mut fa = std::fs::File::create(world_path)?;
        world.write_world(&mut fa)?;
    }

    Ok(())
}
