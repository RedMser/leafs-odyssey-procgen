use std::collections::{HashMap, VecDeque};

use crate::leafs_odyssey::data::*;

enum RoomCommand {
    Resize {
        width: u16,
        height: u16,
    },
    Move {
        x_offset: i16,
        y_offset: i16,
        z_offset: i16,
    },
    Name {
        name: String,
    },
}

pub fn apply_world_commands(world: &mut LOWorld) {
    let mut instructions = HashMap::<u32, Vec<RoomCommand>>::new();

    // Pass 1: collect commands immutably
    for stem in &world.stems {
        match &stem.content {
            LOStemContent::TileMapEdit { id, name, .. } => {
                let commands = parse_commands(&name.to_string());
                if !commands.is_empty() {
                    instructions.insert(
                        *id,
                        commands,
                    );
                }
            },
            _ => {},
        }
    }

    // Pass 2: update world and rooms
    for stem in &mut world.stems {
        match &mut stem.content {
            LOStemContent::TileZoneMap { ref mut room_info, .. } => {
                for room in room_info {
                    let Some(commands) = instructions.get(&room.id) else {
                        continue;
                    };

                    for command in commands {
                        match command {
                            RoomCommand::Resize { width, height } => {
                                room.width = *width;
                                room.height = *height;
                            },
                            RoomCommand::Move { x_offset, y_offset, z_offset } => {
                                room.x_position += x_offset;
                                room.y_position += y_offset;
                                room.z_position += z_offset;
                            },
                            _ => {},
                        }
                    }
                }
            },
            LOStemContent::TileMapEdit { id, name, width, height, .. } => {
                let Some(commands) = instructions.get(id) else {
                    continue;
                };

                let mut renamed = false;
                for command in commands {
                    match command {
                        RoomCommand::Name { name: new_name } => {
                            *name = new_name.clone().into();
                            renamed = true;
                        },
                        RoomCommand::Resize { width: new_width, height: new_height } => {
                            *width = *new_width;
                            *height = *new_height;
                        },
                        _ => {},
                    }
                }

                if !renamed {
                    // fallback name
                    *name = "".into();
                }
            },
        }
    }
}

fn parse_commands(name: &str) -> Vec<RoomCommand> {
    let mut commands = Vec::new();

    if name.is_empty() || name.chars().next().unwrap() != '!' {
        return commands;
    }

    let mut name_parts = name[1..].split(',').collect::<VecDeque<_>>();
    while !name_parts.is_empty() {
        let command_name = name_parts.pop_front().unwrap();
        match command_name {
            "resize" => {
                commands.push(RoomCommand::Resize {
                    width: name_parts.pop_front().unwrap().parse().unwrap(),
                    height: name_parts.pop_front().unwrap().parse().unwrap(),
                });
            },
            "move" => {
                commands.push(RoomCommand::Move {
                    x_offset: name_parts.pop_front().unwrap().parse().unwrap(),
                    y_offset: name_parts.pop_front().unwrap().parse().unwrap(),
                    z_offset: name_parts.pop_front().unwrap().parse().unwrap(),
                });
            },
            "name" | "rename" => {
                commands.push(RoomCommand::Name {
                    name: name_parts.pop_front().unwrap().into(),
                });
            },
            _ => panic!("unknown room command {}", command_name)
        }
    }

    commands
}