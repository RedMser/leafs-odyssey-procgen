use std::rc::Rc;
use crate::room_title_commands::RoomCommand;

macro_rules! room_commands {
    ( $( $mod_name:ident::$cmd_type:ident ),* $(,)? ) => {
        $(
            mod $mod_name;
        )*

        pub fn get_commands() -> Vec<Rc<dyn RoomCommand>> {
            vec![
                $(
                    Rc::new($mod_name::$cmd_type),
                )*
            ]
        }
    };
}

room_commands! {
    position::PositionCommand,
    rect::RectCommand,
    rename::RenameCommand,
    replace::ReplaceCommand,
    resize::ResizeCommand,
    script::ScriptCommand,
    tile::TileCommand,
    layer_add::LayerAddCommand,
    layer_remove::LayerRemoveCommand,
    layer_size::LayerSizeCommand,
    layer_copy::LayerCopyCommand,
    layer_move::LayerMoveCommand,
    sign::SignCommand,
}
