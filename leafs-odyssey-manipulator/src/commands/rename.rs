use crate::room_title_commands::{RoomCommand, RoomCommandContext};

pub struct RenameCommand;

impl RoomCommand for RenameCommand {
    fn names(&self) -> &'static [&'static str] {
        &["rename", "name", "title"]
    }

    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String> {
        let new_name = context.pop_arg().ok_or("Name is missing.")?;

        context.override_room_name = Some(new_name);

        Ok(())
    }
}