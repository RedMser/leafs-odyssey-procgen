#![feature(file_buffered)]

mod string_parser;

use std::{error::Error, path::Path};

use leafs_odyssey_data::{builder::*, data::*, io::get_worlds_folder};

use crate::string_parser::*;

fn main() -> Result<(), Box<dyn Error>> {
    let mut world = World::new()
        .with_metadata("Bad Apple", "Yup!")
        .with_identity(&random_guid_segment(), Author::new("Rust", "00000000-FFFFFFFF"));
    
    // TODO: pass txt file as cmdline args
    import_string("./bad_apple.txt", &mut world)?;

    unsafe {
        let mut fa = std::fs::File::create(
            Path::new(&get_worlds_folder()?).join("bad_apple_generated.world")
        )?;

        let mut world = LOWorld::try_from(world)?;
        world.write_world(&mut fa)?;
    }

    Ok(())
}
