#![feature(file_buffered)]

use leafs_odyssey_data::io::get_worlds_folder;

use libafl::{corpus::NopCorpus, inputs::BytesInput, mutators::{havoc_mutations, MutationResult, Mutator, StdScheduledMutator}, state::StdState};
use libafl_bolts::rands::StdRand;
use std::{env, error::Error, fs, path::PathBuf, process::exit};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    
    if args.is_empty() || args.len() > 3 {
        println!("USAGE: cargo run --bin fuzzing -- [TEMPLATE WORLD NAME] [WORLD NAME PREFIX] [COUNT]");
        println!("");
        println!("PATH TO TEMPLATE WORLD (required): name of a world to load.");
        println!("It is used as a template for fuzzing.");
        println!("");
        println!("WORLD NAME PREFIX: prefix of generated worlds, defaults to \"fuzzing_\"");
        println!("Will overwrite existing world files!");
        println!("");
        println!("COUNT: number of worlds to generate, defaults to 100");
        exit(1);
    }

    let worlds_folder = PathBuf::from(&get_worlds_folder()?);
    let mut template_name = args[0].clone();
    let template_path: PathBuf;
    if template_name.contains('/') || template_name.contains('\\') {
        template_path = PathBuf::from(&template_name);
    } else {
        if !template_name.ends_with(".world") {
            template_name += ".world";
        }
        template_path = worlds_folder.join(&template_name);
    }

    let template_bytes = fs::read(&template_path).expect(&format!("Unable to read template file {}", template_path.to_string_lossy()));
    println!("Template input loaded: {} bytes", template_bytes.len());
    let prefix_name = args.get(1).map_or("fuzzing_", |v| v);
    let fuzz_count = args.get(2).map_or("100", |v| v).parse::<i32>().expect("Fuzz count parameter is not formatted as an integer.");

    // fuzzing via libafl
    let mut state = StdState::new(
        StdRand::new(),
        NopCorpus::new(),
        NopCorpus::new(),
        &mut (),
        &mut (),
    )?;
    let mut mutator = StdScheduledMutator::new(havoc_mutations());
    for i in 1..=fuzz_count {
        let output_name = format!("{}{}.world", prefix_name, i);
        let output_path = worlds_folder.join(&output_name);
        let mut input_bytes = BytesInput::from(&template_bytes[..]);
        let result = mutator.mutate(&mut state, &mut input_bytes)?;
        if matches!(result, MutationResult::Mutated) {
            let input_bytes = input_bytes.into_inner();
            fs::write(output_path, &input_bytes)?;
            println!("Wrote {} with {} bytes", output_name, input_bytes.len());
        }
    }

    Ok(())
}
