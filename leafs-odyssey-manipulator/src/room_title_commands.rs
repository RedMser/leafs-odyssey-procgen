use std::{collections::HashMap, path::PathBuf, rc::Rc};

use leafs_odyssey_data::data::*;

pub trait RoomCommand {
    fn names(&self) -> &'static [&'static str];
    fn execute(&self, context: &mut RoomCommandContext) -> Result<(), String>;
}

pub struct WorldCommandContext<'w> {
    pub world: &'w mut LOWorld,
    pub script_cache: &'w mut HashMap<PathBuf, String>,
    pub verbose: bool,
}

pub struct RoomCommandContext<'w> {
    args: Vec<String>,
    pub room_id: u32,
    pub override_room_name: Option<String>,
    pub env: &'w mut WorldCommandContext<'w>,
    pub sign_text: HashMap<String, String>,
}

impl<'w> RoomCommandContext<'w> {
    pub fn new(args: Vec<String>, world: &'w mut WorldCommandContext<'w>, room_id: u32) -> Self {
        Self {
            args,
            room_id,
            override_room_name: None,
            env: world,
            sign_text: Default::default(),
        }
    }

    pub fn get_room_info(&self) -> &LORoomInfo {
        for stem in &self.env.world.stems {
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
        for stem in &mut self.env.world.stems {
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
        let arg = self.args.pop();
        if self.env.verbose {
            println!("[v] Pop arg {}", &arg.clone().unwrap_or("(null)".into()));
        }
        arg
    }

    pub fn has_arg(&self) -> bool {
        !self.args.is_empty()
    }

    pub fn args_count(&self) -> usize {
        self.args.len()
    }

    /// args should not be reversed (so keep it in reading order)!
    pub fn push_args(&mut self, args: Vec<String>) {
        let i = self.args.len();
        for arg in args {
            self.args.insert(i, arg);
        }
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

    let tokens = parse_args_only(args_string);

    (room_name.to_owned(), tokens)
}

pub fn parse_args_only(room_args: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = room_args.chars().peekable();
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
            ' ' | '\r' | '\n' if !in_quotes => {
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

    tokens
}

pub fn apply_world_commands<C>(world: &mut LOWorld, registered_commands: C, increment_room_revision: bool, verbose: bool, mut autoscript_cache: HashMap::<RoomCoordinates, String>) -> WorldCommandsResult
where
    C: IntoIterator<Item = Rc<dyn RoomCommand>>,
{
    let mut results = WorldCommandsResult::default();
    let mut room_args_map = HashMap::<u32, (String, Vec<String>)>::new();
    let registered_commands = registered_commands
        .into_iter()
        .flat_map(|cmd| cmd.names().iter().map(move |name| (name, cmd.clone())))
        .collect::<HashMap<_, _>>();

    // Pre-pass for autoscript: find out which room ids have which coordinates
    let mut room_id_to_autoscript = HashMap::<u32, String>::new();
    if !autoscript_cache.is_empty() {
        for stem in &world.stems {
            match &stem.content {
                LOStemContent::TileZoneMap { room_info, .. } => {
                    for room in room_info {
                        let coords = RoomCoordinates(room.x_position, room.y_position, room.z_position);
                        if let Some(autoscript) = autoscript_cache.remove(&coords) {
                            if verbose {
                                println!("Room ID {} at coords {:?} loaded autoscript {}", &room.id, &coords, &autoscript);
                            }
                            room_id_to_autoscript.insert(room.id, autoscript);
                        }
                    }
                    break;
                },
                _ => {},
            }
        }

        if !autoscript_cache.is_empty() {
            let size = autoscript_cache.len();
            let contents = autoscript_cache.into_values()
                .collect::<Vec<_>>()
                .join(", ");
            println!("Following {} autoscript configs were found, but without corresponding rooms: {}", size, contents);
        }
    }

    // Pass 1: collect command args immutably + stem contents
    for stem in &world.stems {
        match &stem.content {
            LOStemContent::TileMapEdit { id, name, .. } => {
                let (room_name, mut args) = parse_args(&name.to_string());

                // Insert autoscript
                if let Some(autoscript) = room_id_to_autoscript.remove(&id) {
                    args.push("script".into());
                    args.push(autoscript);
                }

                if !args.is_empty() {
                    // We're using Vec::pop(), so args pop from end to start.
                    args.reverse();
                    room_args_map.insert(*id, (room_name, args));
                }
            },
            _ => {},
        }
    }

    // Pass 2: run all commands
    let mut script_cache = HashMap::new();
    for (id, (room_name, args)) in room_args_map.into_iter() {
        let mut env = WorldCommandContext {
            world,
            script_cache: &mut script_cache,
            verbose,
        };
        let mut ctx = RoomCommandContext::new(args, &mut env, id);

        // Keep looping over args until we've exhausted the list.
        let mut modified = false;
        let mut i = 0;
        const LIMIT: i32 = 50000;
        'arg: loop {
            i += 1;
            if i >= LIMIT {
                println!("ERROR: Command buffer ran out > {}! Do you possibly have infinite recursion? E.g. a script command that runs itself...", LIMIT);
                break;
            }

            let arg = ctx.pop_arg();
            match arg {
                Some(arg) => {
                    // Check if arg is a valid command.
                    let cmd = registered_commands.get(&arg.as_str());
                    match cmd {
                        Some(cmd) => {
                            // Run the command!
                            if verbose {
                                println!("[v] Exec command {}", &arg);
                            }
                            let res = cmd.execute(&mut ctx);
                            if let Err(err) = res {
                                results.errors.push(format!("Command number {} \"{}\" errored in room {} ({}): {}", i, &arg, &id, &room_name, &err));
                            } else {
                                modified = true;
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
        for stem in &mut ctx.env.world.stems {
            match &mut stem.content {
                LOStemContent::TileMapEdit { id, name, revision, .. } => {
                    if *id == ctx.room_id {
                        *name = ctx.override_room_name.unwrap_or_else(|| room_name.clone()).into();
                        if modified && increment_room_revision {
                            *revision = *revision + 1;
                        }
                        break;
                    }
                },
                _ => {},
            }
        }
    }

    results
}
