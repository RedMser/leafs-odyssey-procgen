mod room_title_commands;

use binrw::BinRead;
use std::{env, error::Error, path::Path, process::exit};

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

    let input_name = &args[0];
    let input_path = if input_name.contains('/') || input_name.contains('\\') {
        Path::new(input_name)
    } else {
        &Path::new(&get_worlds_folder()?).join(input_name)
    };
    
    println!("Reading world \"{:?}\"...", input_path);
    let mut fa = std::fs::File::open(input_path)?;
    let mut world = LOWorld::read(&mut fa)?;

    println!("Applying modifications...");
    apply_world_commands(&mut world);

    let output_name = String::from("generated_") + input_name;
    let output_name = args.get(1).unwrap_or(&output_name);
    let output_path = if output_name.contains('/') || output_name.contains('\\') {
        Path::new(output_name)
    } else {
        &Path::new(&get_worlds_folder()?).join(output_name)
    };
    println!("Writing file \"{:?}\"...", output_path);

    unsafe {
        let mut fa = std::fs::File::create(output_path)?;

        let mut world = LOWorld::try_from(world)?;
        world.write_world(&mut fa)?;
    }

    Ok(())
}
