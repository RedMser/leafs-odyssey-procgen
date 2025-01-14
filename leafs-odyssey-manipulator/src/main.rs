mod room_title_commands;

use binrw::BinRead;
use std::{error::Error, path::Path};

use leafs_odyssey_data::{data::*, io::get_worlds_folder};
use room_title_commands::apply_world_commands;

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: parse input and output filenames via cmdline args
    let mut fa = std::fs::File::open(
        Path::new(&get_worlds_folder()?).join("antichamber.world")
    )?;

    let mut world = LOWorld::read(&mut fa)?;

    apply_world_commands(&mut world);
    
    unsafe {
        let mut fa = std::fs::File::create(
            Path::new(&get_worlds_folder()?).join("antichambernew.world")
        )?;

        let mut world = LOWorld::try_from(world)?;
        world.write_world(&mut fa)?;
    }

    Ok(())
}
