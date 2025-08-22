# Leaf's Odyssey Procgen

Rust crates for managing [Leaf's Odyssey](https://store.steampowered.com/app/2880750/Leafs_Odyssey/) world data.
Spawned from a personal reverse engineering effort and will thus not be fully correct or complete.

It is possible to save and load `.world` files made with game version 1.0.15.

## leafs-odyssey-manipulator

A command line tool to load an existing world and apply changes to each room.

Basic usage: `./leafs-odyssey-manipulator.exe foo` loads `foo.world` from the worlds directory and saves a manipulated version to `generated_foo.world`.
Can also be run using `cargo run --release -- foo` instead.

Other command line arguments can be listed via `--help`, but the most common ones are:

- `--title Blah` changes the name of the world in the select screen (by default, it appends `[MANIP]` to the end of the name).
- `--dump --dryrun` will print a list of all rooms and their IDs, without running the manipulator. Use `--dump-after` instead of `--dump` to see how your changes affected the metadata.

If a room has a certain format in its **title**, it will be manipulated by the tool, for example:

```
This is my room -- move 24 0 0 size 12 8
```

This would move the room to the right by 24 tiles, while also reducing its size to 12x8. It'll be named `This is my room` in the manipulated world.

Following commands are supported:

- `move X Y Z` - moves this room by this many **tiles**. So to move a room by a full size, X must be a multiple of 24 and Y must be a multiple of 16. +X is east, +Y is south, +Z is up.
- `size W H` - changes the room's size to the given width/height. Rooms are 24x16 by default, but can be made smaller or bigger. Note that the tilemap is not resized, and stays at 24x16 size.
- `tile X Y T` - places tile T at the given coordinate. Syntax details on T are below.
- `rect X1 Y1 X2 Y2 T` - places tile T within the given rectangle area. Syntax details on T are below.
- `replace C T` - finds all tiles that match the condition C, then runs the tile command on them with T. It's not really a true find-and-replace, but just a complex way to specify coordinates. Syntax details on C and T are below.
- `addlayer N T` - adds N amount of new 24x16 sized tilemap layers, each filled with T, to the end of the room's tilemap list. There's a limit of 64 tilemap layers per room. Syntax details on T are below (multiple tiles are not supported).
- `removelayer I` - removes the tilemap layer at 0-based index I.
- `sizelayer I W H` - resizes tilemap layer at 0-based index I to the given size.
- `copylayer I J` - copies the contents of tilemap layer I to J (both are 0-based indices).
- `movelayer I J` - moves the tilemap layer from I to J (both are 0-based indices).
- `sign I C` - assign a unique identifier I with the corresponding string contents C. When placing a sign, refer to this ID to write the given contents to the sign. Stored per room, and must be assigned *before* creating the sign. Use quotes around text contents to allow whitespace.
- `rename N` - rename the room to N. Without this command, the room's name (anything before `--`) is used, but the editor imposes a 64 characters limit. You can use this command inside of a script to circumvent the length limit.
- `script S` - runs commands from the given script file S. It's a path, by default it's relative to the current working directory (can be tweaked with `--script-dir` launch parameter). If no file extension is specified, it defaults to `.cfg`. You can use new lines synonymously to spaces in script files! Use `/* */` to create comments.

Arguments of commands may only include spaces if you surround them with "quotes". You can escape literal quotes using a backslash.
Commands are executed in reading order, so the first command runs first.

### Tile syntax

For tile-related commands (argument "T"), syntax is as follows.

A simple example to start with is `Sand+(PushBlock+TerraKey*2)` which places a stack on sand, with the stack consisting of a push block and two terra keys.

Basically, you use the name of a tile as defined in the `LOTile` enum in `data.rs`.
A plus `+` means you go up a layer.
Parentheses `( )` define a stack.
Multiplication like `T*n` can be used to substitute `T+T+T+...` n-times.

Complex tiles can be parametrized with colon, such as:

- `BombBug:Left` and other monsters facing direction. Monsters default to facing down.
- `PressurePlate:1,2:3,4` and other wiring elements (list of connection targets).
- `Sign:id` together with the `sign` command above.

When writing tiles, there are two modes:

- If you specify exactly as many elements as there are layers (5 by default), or if you specify 5 or more elements, then the entire coordinate is replaced. Order is from bottom to top layer. Stacks count as a single layer (since they're internally a single tile with additional info). Use `None` as filler.
- If you specify any less than that, then a "smart merge" is done instead, which tries to keep existing contents and use appropriate layers automatically (e.g. if you specify `Sand`, it won't touch any objects but only replace the floor type).

#### Condition syntax

For condition-related commands (argument "C"), syntax is nearly equivalent to tile syntax "T" above.

- If you specify exactly as many elements as there are layers (5 by default), then an exact match is required. So every layer is checked individually, and only if each tile type matches, will the condition match. Use `None` as filler.
- If you specify any less than that, then the system does a loose check. So filtering for only `BombBug` won't care about the floor type that a bomb bug is on.

#### Stack quirks

For most of the limits here, the workaround is to use layers instead of stacks.

- Only the tiles available in the editor can be used in stacks (see the `LOStackTile` enum in `data.rs`). The only exception is `None`, however it doesn't seem to do anything useful.
- A stack can only be 16 tall at maximum.
- All wiring elements in a stack must share the same set of connections.
  - The manipulator will add up all connections from every wiring element of a stack, and combine them into one list.
  - If you have overlapping stacks on different layers, then only the first stack's connections are used.

### The layer system

Each room in a Leaf's Odyssey world comes with its own `tilemap_edit` stem, which hosts a list of layers.
By default, each room has 5 layers, which are roughly used as follows:

1. Floors and walls
2. Obstacles, floor-level puzzle elements (trapdoors, ladders, pressure plates, toggle floors, ...)
3. Crumbly walls, toggle doors and monster gates
4. Keys and push/multipush/monster blocks
5. Monsters, key doors, toggle switches, signs, rubble, poison trail, stacks

So by placing a tile via the editor or via a simple `tile` command, they are put into their appropriate layer.
But you do not need to adhere to this, and can place tiles on any layer, as well as creating more layers.

From my testing, layers and their order don't seem to matter much.

## leafs-odyssey-data

Library for reading and writing world data. Includes two APIs:

- A raw `data` API, which is (de)serialized as-is via the `binrw` library.
- A convenience `builder` API, which has a nicer API to interface with.

## Other examples

Various library examples are found in the `examples` folder, some using the `data` API while others use the `builder` API.

They can be run via `cargo run --bin NAME -- ARGS` whereas `NAME` is the folder name. `ARGS` vary per example, see the usage string inside.
