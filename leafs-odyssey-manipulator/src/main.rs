mod commands;
mod room_title_commands;
mod tile_parser;
mod utils;

use binrw::BinRead;
use clap::Parser;
use regex::Regex;
use std::{collections::HashMap, error::Error, fs, path::{Path, PathBuf}};

use leafs_odyssey_data::{data::*, io::get_worlds_folder};
use room_title_commands::apply_world_commands;

#[derive(Parser)]
struct Args {
    /// File name or path of world to load.
    pub input_world_name: String,
    /// File name or path of manipulated world to save.
    /// Defaults to input world name with the prefix "generated_".
    /// Will overwrite the specified world file if it already exists!
    pub output_world_name: Option<String>,
    /// New title to give the world. Defaults to the world's original title with the suffix "[MANIP]".
    #[arg(short = 't', long = "title")]
    pub world_title: Option<String>,
    /// Set this flag to keep the same world GUID.
    /// By default, the GUID is incremented automatically to avoid the world being treated as identical by the game.
    #[arg(long, action)]
    pub keep_guid: bool,
    /// Set this flag to keep the same world revision.
    /// By default, the GUID is incremented automatically to avoid the world being treated as unchanged compared to its original counterpart.
    #[arg(long, action)]
    pub keep_world_revision: bool,
    /// Set this flag to keep the same room revision.
    /// By default, the GUID of every room that has commands in it is incremented automatically to avoid the room being treated as unchanged compared to its original counterpart.
    #[arg(long, action)]
    pub keep_room_revision: bool,
    /// Print unaltered world and room metadata.
    #[arg(long, action)]
    pub dump: bool,
    /// Print altered world and room metadata.
    #[arg(long, action)]
    pub dump_after: bool,
    /// Don't actually write the output file.
    #[arg(long, action)]
    pub dryrun: bool,
    /// Verbose output (e.g. details on command parsing).
    #[arg(short, long, action)]
    pub verbose: bool,
    /// Dumps the room's layer data with the given ID.
    #[arg(short, long, action)]
    pub dump_room: Option<u32>,
    /// Files in the working directory named e.g. myworld_1_2_3.cfg (uses input filename) will automatically be loaded for the room at coordinate (1,2,3).
    #[arg(short, long, action)]
    pub auto_script: bool,
}

fn world_name_to_path(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    Ok(if name.contains('/') || name.contains('\\') {
        PathBuf::from(&name)
    } else {
        let name = if !name.ends_with(".world") {
            name.to_owned() + ".world"
        } else {
            name.to_owned()
        };
        PathBuf::from(&get_worlds_folder()?).join(&name)
    })
}

fn dump_world(world: &LOWorld, dump_room_id: Option<u32>) {
    let mut room_id_to_geometry = HashMap::new();

    // Pass 1: Search for world info + store room coords map
    for stem in &world.stems {
        match &stem.content {
            LOStemContent::TileZoneMap { name, world_revision, guid_world, author, description, guid_author1, guid_author2, start_room, room_info, .. } => {
                println!("=== World Info ===");
                println!("  Title: {}", name);
                println!("  Description: {}", description);
                println!("  Author: {} ({}-{})", author, to_guid_string(*guid_author1), to_guid_string(*guid_author2));
                println!("  World GUID: {}", guid_world);
                println!("  World Revision: {}", world_revision);
                println!("  Starting Room ID: {}", start_room);
                println!("  Room Count: {}", room_info.len());
                println!("");

                for room in room_info {
                    room_id_to_geometry.insert(room.id, (room.x_position, room.y_position, room.z_position, room.width, room.height));
                }
                break;
            },
            _ => {},
        }
    }

    fn render_size(room: u16, geo: u16) -> String {
        if room == geo {
            room.to_string()
        } else {
            format!("{} room, {} world", room, geo)
        }
    }

    // Pass 2: rooms
    for stem in &world.stems {
        match &stem.content {
            LOStemContent::TileMapEdit { id, name, width, height, layers, music, revision, .. } => {
                let room_geo = room_id_to_geometry.get(id);
                if let Some((x, y, z, geo_width, geo_height)) = room_geo {
                    // TODO: special handling if grid coordinate is not a multiple of 24x16:
                    // use whatever logic the game decides where the room is shown on minimap, then include a "tile offset" value from there).
                    println!("=== Room {},{},{} ===", x/24, y/16, z);
                    println!("  ID: {}", id);
                    println!("  Title: {}", name);
                    println!("  Width: {}", render_size(*width, *geo_width));
                    println!("  Height: {}", render_size(*height, *geo_height));
                } else {
                    println!("=== ERROR: Room with ID {} is an orphan (not in the world geometry) ===", id);
                    println!("  ID: {}", id);
                    println!("  Title: {}", name);
                    println!("  Width: {}", width);
                    println!("  Height: {}", height);
                }
                println!("  Music: {}", *music);
                println!("  Room Revision: {}", revision);
                println!("  Layers: {}", layers.len());
                let mut weird_sized_layers = false;
                for (i, layer) in layers.iter().enumerate() {
                    if layer.width != 24 || layer.height != 16 {
                        println!("    Layer {} Size: {}, {}", i, layer.width, layer.height);
                        weird_sized_layers = true;
                    }
                }
                if !weird_sized_layers {
                    println!("    (All layers are sized 24x16)");
                }
                println!("");

                if let Some(dump_room_id) = dump_room_id {
                    if *id == dump_room_id {
                        for (i, layer) in layers.iter().enumerate() {
                            println!("  Contents of Layer {i}:");
                            
                            for y in 0..layer.height {
                                let mut row = vec![];
                                for x in 0..layer.width {
                                    let j = x + y * layer.width;
                                    let tile = &layer.tiles[j as usize];
                                    row.push(format!("{:?}", &tile));
                                }
                                println!("{}", row.join(" "));
                            }
                        }
                    }
                }
            },
            _ => {},
        }
    }
}

fn populate_autoscript_cache(world_name: &str, autoscript_cache: &mut HashMap<RoomCoordinates, String>) {
    let regex = Regex::new(&format!(r"^{}_(-?\d+)_(-?\d+)_(-?\d+).cfg$", regex::escape(world_name))).unwrap();
    for entry in fs::read_dir("./").unwrap() {
        if let Ok(entry) = entry {
            if let Ok(file_type) = entry.file_type() {
                if !file_type.is_file() { continue; }

                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                if let Some(matches) = regex.captures(&file_name) {
                    let x_position = matches.get(1).unwrap().as_str().parse().unwrap();
                    let y_position = matches.get(2).unwrap().as_str().parse().unwrap();
                    let z_position = matches.get(3).unwrap().as_str().parse().unwrap();
                    let coordinates = RoomCoordinates(x_position, y_position, z_position);

                    autoscript_cache.insert(coordinates, file_name.into_owned());
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let input_name = args.input_world_name;
    let output_name = args.output_world_name.unwrap_or_else(|| String::from("generated_") + &input_name);

    let input_path = world_name_to_path(&input_name)?;
    let output_path = world_name_to_path(&output_name)?;

    if input_path == output_path {
        println!("Input and output path may not be the same, to avoid accidential data loss!");
        return Ok(());
    }

    let mut autoscript_cache = HashMap::<RoomCoordinates, String>::new();
    if args.auto_script {
        let input_name = Path::new(&input_path).file_stem().unwrap().to_string_lossy();
        populate_autoscript_cache(&input_name, &mut autoscript_cache);
    }

    println!("Reading world \"{:?}\"...", input_path);
    let mut fa = std::fs::File::open(input_path)?;
    let mut world = LOWorld::read(&mut fa)?;

    if args.dump && !args.dump_after {
        dump_world(&world, args.dump_room);
    }

    println!("Applying modifications...");
    let results = apply_world_commands(&mut world, commands::get_commands(), !args.keep_room_revision, args.verbose, autoscript_cache);

    for error in results.errors {
        println!("ERROR: {}", error);
    }

    if !results.modified {
        println!("No room with commands was found! Check README for more info.");
    }

    for stem in &mut world.stems {
        match &mut stem.content {
            LOStemContent::TileZoneMap { name, world_revision, guid_world, .. } => {
                let new_world_title = args.world_title.unwrap_or_else(|| name.to_string() + " [MANIP]");
                *name = new_world_title.into();

                if !args.keep_world_revision {
                    *world_revision = *world_revision + 1;
                }
                if !args.keep_guid {
                    *guid_world = guid_world.wrapping_add(1);
                }
                break;
            },
            _ => {},
        }
    }

    if args.dump_after {
        dump_world(&world, args.dump_room);
    }

    if !args.dryrun {
        println!("Writing file \"{:?}\"...", output_path);
        let mut fa = std::fs::File::create(output_path)?;
        let mut world = LOWorld::try_from(world)?;
        world.write_world(&mut fa)?;
    } else {
        println!("Skipping write because of --dryrun flag... Output file would be at \"{:?}\"...", output_path);
    }

    Ok(())
}
