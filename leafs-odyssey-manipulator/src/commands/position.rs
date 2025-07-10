use crate::room_title_commands::{RoomCommand, RoomCommandContext};

pub struct PositionCommand;

impl RoomCommand for PositionCommand {
    fn names(&self) -> &'static [&'static str] {
        &["position", "pos", "move"]
    }

    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String> {
        let new_x = context.pop_arg().ok_or("X position is missing.")?.parse::<i16>().map_err(|_| "X position is not an integer.")?;
        let new_y = context.pop_arg().ok_or("Y position is missing.")?.parse::<i16>().map_err(|_| "Y position is not an integer.")?;
        let new_z = context.pop_arg().ok_or("Z position is missing.")?.parse::<i16>().map_err(|_| "Z position is not an integer.")?;

        let room_info = context.get_room_info_mut();
        room_info.x_position = new_x;
        room_info.y_position = new_y;
        room_info.z_position = new_z;

        Ok(())
    }
}