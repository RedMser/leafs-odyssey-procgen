use std::{collections::HashMap, rc::Rc};

use leafs_odyssey_data::data::*;

pub trait RoomCommand {
    fn names(&self) -> &'static [&'static str];
    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String>;
}

pub struct RoomCommandContext<'w> {
    args: Vec<String>,
    pub room_id: u32,
    pub world: &'w mut LOWorld,
}

impl<'w> RoomCommandContext<'w> {
    pub fn new(args: Vec<String>, world: &'w mut LOWorld, room_id: u32) -> Self {
        Self {
            args,
            room_id,
            world,
        }
    }

    pub fn get_room_info(&self) -> &LORoomInfo {
        for stem in &self.world.stems {
            match &stem.content {
                LOStemContent::TileZoneMap { room_info, .. } => {
                    for room in room_info {
                        if room.id == self.room_id {
                            return &room;
                        }
                    }
                    break;
                },
                _ => {},
            }
        }
        panic!("Invalid room id in RoomCommandContext! {}", self.room_id);
    }

    pub fn get_room_info_mut(&mut self) -> &mut LORoomInfo {
        for stem in &mut self.world.stems {
            match &mut stem.content {
                LOStemContent::TileZoneMap { room_info, .. } => {
                    for room in room_info {
                        if room.id == self.room_id {
                            return room;
                        }
                    }
                    break;
                },
                _ => {},
            }
        }
        panic!("Invalid room id in RoomCommandContext! {}", self.room_id);
    }

    pub fn pop_arg(&mut self) -> Option<String> {
        self.args.pop()
    }

    pub fn has_arg(&self) -> bool {
        !self.args.is_empty()
    }

    pub fn args_count(&self) -> usize {
        self.args.len()
    }
}

#[derive(Default)]
pub struct WorldCommandsResult {
    pub modified: bool,
    pub errors: Vec<String>,
}

/// Returns the room name, followed by a list of args.
pub fn parse_args(room_name: &str) -> (String, Vec<String>) {
    // Args start after the delimiter.
    const DELIMITER: &'static str = " -- ";

    let (room_name, args_string) = if let Some(pos) = room_name.find(DELIMITER) {
        let left = &room_name[..pos];
        let right = &room_name[pos + DELIMITER.len()..];
        (left, right)
    } else {
        // No commands, return room name as-is.
        return (room_name.to_owned(), vec![]);
    };

    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = args_string.chars().peekable();
    let mut in_quotes = false;

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                // Handle escaping inside or outside quotes
                if let Some(&next) = chars.peek() {
                    if next == '"' || next == '\\' {
                        current.push(next);
                        chars.next();
                    } else {
                        current.push(c);
                    }
                } else {
                    current.push(c);
                }
            }
            '"' => {
                in_quotes = !in_quotes;
                // Don't add quote to token
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                // skip adding spaces outside quotes
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    (room_name.to_owned(), tokens)
}

pub fn apply_world_commands<C>(world: &mut LOWorld, registered_commands: C) -> WorldCommandsResult
where
    C: IntoIterator<Item = Rc<dyn RoomCommand>>,
{
    let mut results = WorldCommandsResult::default();
    let mut room_args_map = HashMap::<u32, (String, Vec<String>)>::new();
    let registered_commands = registered_commands
        .into_iter()
        .flat_map(|cmd| cmd.names().iter().map(move |name| (name, cmd.clone())))
        .collect::<HashMap<_, _>>();

    // Pass 1: collect command args immutably + stem contents
    for stem in &world.stems {
        match &stem.content {
            LOStemContent::TileMapEdit { id, name, .. } => {
                let (room_name, mut args) = parse_args(&name.to_string());
                if args.is_empty() {
                    continue;
                }

                // We're using Vec::pop(), so args pop from end to start.
                args.reverse();
                room_args_map.insert(*id, (room_name, args));
            },
            _ => {},
        }
    }

    // Pass 2: run all commands
    for (id, (room_name, args)) in room_args_map.into_iter() {
        let mut ctx = RoomCommandContext::new(args, world, id);

        // Keep looping over args until we've exhausted the list.
        'arg: loop {
            let arg = ctx.pop_arg();
            match arg {
                Some(arg) => {
                    // Check if arg is a valid command.
                    let cmd = registered_commands.get(&arg.as_str());
                    match cmd {
                        Some(cmd) => {
                            // Run the command!
                            let res = cmd.execute(&mut ctx);
                            if let Err(err) = res {
                                results.errors.push(err);
                            } else {
                                results.modified = true;
                            }
                        },
                        None => {
                            results.errors.push(format!("Unknown command: {}", &arg).into());
                            break 'arg;
                        },
                    }
                },
                None => {
                    break 'arg;
                },
            }
        }

        // Rename the corresponding room.
        for stem in &mut world.stems {
            match &mut stem.content {
                LOStemContent::TileMapEdit { name, .. } => {
                    *name = room_name.clone().into();
                },
                _ => {},
            }
        }
    }

    results
}
