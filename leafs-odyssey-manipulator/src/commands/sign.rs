use crate::room_title_commands::{RoomCommand, RoomCommandContext};

pub struct SignCommand;

impl RoomCommand for SignCommand {
    fn names(&self) -> &'static [&'static str] {
        &["sign"]
    }

    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String> {
        let sign_id = context.pop_arg().ok_or("Sign ID is missing.")?;
        let sign_text = context.pop_arg().ok_or("Sign text is missing.")?;

        context.sign_text.insert(sign_id, sign_text);

        Ok(())
    }
}