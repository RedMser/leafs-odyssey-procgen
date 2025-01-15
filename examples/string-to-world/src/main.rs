#![feature(file_buffered)]

mod string_parser;

use std::{env, error::Error, path::Path, process::exit};

use leafs_odyssey_data::{builder::*, data::*, io::get_worlds_folder};

use crate::string_parser::*;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.len() > 2 {
        println!("USAGE: cargo run --bin string-to-world -- [INPUT] [OUTPUT]");
        println!("");
        println!("INPUT (required): path to a text file to load.");
        println!("See valid_tile_names.txt for a list of strings, and bad_apple.txt for an example.");
        println!("");
        println!("OUTPUT: file name of world, defaults to \"generated_string.world\"");
        println!("Will overwrite existing world file!");
        exit(1);
    }

    let mut world = World::new()
        .with_metadata("[Generated] String", "Yup!")
        .with_identity(&random_guid_segment(), Author::new("Rust", "00000000-FFFFFFFF"));
    
    import_string(&args[0], &mut world)?;

    let world_name = "generated_string.world".into();
    let world_name = args.get(1).unwrap_or(&world_name);
    let world_path = if world_name.contains('/') || world_name.contains('\\') {
        Path::new(world_name)
    } else {
        &Path::new(&get_worlds_folder()?).join(world_name)
    };
    println!("Writing file \"{:?}\"...", world_path);

    unsafe {
        let mut fa = std::fs::File::create(world_path)?;
        let mut world = LOWorld::try_from(world)?;
        world.write_world(&mut fa)?;
    }

    Ok(())
}
