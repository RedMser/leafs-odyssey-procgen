mod room_title_commands;

use binrw::BinRead;
use std::{env, error::Error, path::PathBuf, process::exit};

use leafs_odyssey_data::{data::*, io::get_worlds_folder};
use room_title_commands::apply_world_commands;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.len() > 2 {
        println!("USAGE: cargo run -- [INPUT] [OUTPUT]");
        println!("");
        println!("INPUT (required): file name of a world to load.");
        println!("");
        println!("OUTPUT: file name of modified world to save.");
        println!("Defaults to the input world name with the prefix \"generated_\".");
        println!("Will overwrite the specified world file if it already exists!");
        exit(1);
    }

    let mut input_name = args[0].clone();
    let input_path: PathBuf;
    if input_name.contains('/') || input_name.contains('\\') {
        input_path = PathBuf::from(&input_name);
    } else {
        if !input_name.ends_with(".world") {
            input_name += ".world";
        }
        input_path = PathBuf::from(&get_worlds_folder()?).join(&input_name);
    }
    
    println!("Reading world \"{:?}\"...", input_path);
    let mut fa = std::fs::File::open(input_path)?;
    let mut world = LOWorld::read(&mut fa)?;

    println!("Applying modifications...");
    let modified = apply_world_commands(&mut world);

    if !modified {
        println!("No room with commands was found! Check README for more info.");
    }

    for stem in &mut world.stems {
        match &mut stem.content {
            LOStemContent::TileZoneMap { name, world_revision, .. } => {
                *name = (name.to_string() + " [MANIP]").into();
                *world_revision = *world_revision + 1;
            },
            _ => {},
        }
    }

    let output_name = String::from("generated_") + &input_name;
    let mut output_name = args.get(1).map(|arg| arg.clone()).unwrap_or(output_name);
    let output_path: PathBuf;
    if output_name.contains('/') || output_name.contains('\\') {
        output_path = PathBuf::from(&output_name);
    } else {
        if !output_name.ends_with(".world") {
            output_name += ".world";
        }
        output_path = PathBuf::from(&get_worlds_folder()?).join(&output_name);
    }
    println!("Writing file \"{:?}\"...", output_path);

    unsafe {
        let mut fa = std::fs::File::create(output_path)?;

        let mut world = LOWorld::try_from(world)?;
        world.write_world(&mut fa)?;
    }

    Ok(())
}
