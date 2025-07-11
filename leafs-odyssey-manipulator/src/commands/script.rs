use std::{fs, path::PathBuf};

use crate::room_title_commands::{parse_args_only, RoomCommand, RoomCommandContext};

pub struct ScriptCommand;

impl RoomCommand for ScriptCommand {
    fn names(&self) -> &'static [&'static str] {
        &["script", "scr", "import", "load"]
    }

    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String> {
        let file_path = context.pop_arg().ok_or("Script file path is missing.")?;
        let mut file_path = PathBuf::from(file_path);
        if file_path.extension().is_none() {
            file_path.set_extension("cfg");
        }

        // TODO: avoid cloning script contents unnecessarily here...
        let script_contents = if let Some(script) = context.env.script_cache.get(&file_path) {
            script.clone()
        } else {
            println!("Reading script file \"{:?}\"...", file_path);

            let script = fs::read_to_string(&file_path).map_err(|_| "Failed to read script file.")?;
            context.env.script_cache.insert(file_path, script.clone());
            script
        };

        // Insert into args buffer.
        let new_args = parse_args_only(script_contents.as_str());
        context.push_args(new_args);

        Ok(())
    }
}