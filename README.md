# Leaf's Odyssey Procgen

Rust crates for managing [Leaf's Odyssey](https://store.steampowered.com/app/2880750/Leafs_Odyssey/) world data.
Spawned from a personal reverse engineering effort and will thus not be fully correct or complete.

It is possible to save and load `.world` files made with game version 1.0.13.

## leafs-odyssey-data

Library for reading and writing world data. Includes two APIs:

- A raw `data` API, which is (de)serialized as-is
- A convenience `builder` API, which has a nicer API to interface with.

## leafs-odyssey-manipulator

A command line tool to load an existing world and apply changes to each room.

Basic usage: `./leafs-odyssey-manipulator.exe foo` loads `foo.world` from the worlds directory and saves a manipulated version to `generated_foo.world`.
Can also be run using `cargo run --release -- foo` instead.

Other command line arguments can be listed via `--help`, but the most common ones are:

- `--title Blah` changes the name of the world in the select screen (by default, it appends `[MANIP]` to the end of the name).
- `--dump --dryrun` will print a list of all rooms and their IDs, which can be helpful for weird move shenanigans. It also won't output a manipulated version of the world. Use `--dump-after` instead to see how your changes affected the metadata.

If a room has a certain format in its **title**, it will be manipulated by the tool, for example:

```
This is my room -- move 24 0 0 size 12 8
```

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
- `rename N` - rename the room to N. While the room title (anything before --) is used, there's a 64 characters limit. You can use scripts to circumvent the length limit of room name + commands.
- `script S` - runs commands from the script file named S. It's a path relative to the current working directory. If no file extension is specified, it defaults to `.cfg`. You can use new lines synonymously to spaces in script files!

Arguments of commands may only include spaces if you surround them with "quotes". You can escape literal quotes using a backslash.
Commands are executed in reading order, so the first command runs first.

### Tile syntax

For tile-related commands (argument "T"), syntax is as follows.

A simple example to start with is `Sand+(PushBlock+TerraKey*2)` which places a stack on sand, with the stack consisting of a push block and two terra keys.

Basically, you use the name of a tile as defined in `LOTile` enum in `data.rs`.
A plus `+` means you go up a layer.
Parentheses `( )` define a stack (max height is 16).
Multiplication like `T*n` can be used to substitute `T+T+T+...` n-times.

Complex tiles can be parametrized with colon, such as:

- `BombBug:Left` and other monsters facing direction. Monsters default to facing down.
- `PressurePlate:1,2:3,4` and other wiring elements (list of connection targets).

Sign text is currently not customizable and always shows up blank.

When writing tiles, there are two modes:

- If you specify exactly as many elements as there are layers (5 by default), or if you specify 5 or more elements, then the entire coordinate is replaced. Order is from bottom to top layer. Check `Tilemap::LAYER_*` constants for default layer info. Also stacks count as a single layer (since they're internally a single tile with additional info). Use `None` as filler.
- If you specify any less than that, then a "smart merge" is done instead, which tries to keep existing contents and use appropriate layers automatically (e.g. if you specify `Sand`, it won't touch any objects but only replace the floor type).

### Condition syntax

For condition-related commands (argument "C"), syntax is nearly equivalent to tile syntax "T" above.

- If you specify exactly as many elements as there are layers (5 by default), then an exact match is required. So every layer is checked individually, and only if each tile type matches, will the condition match. Use `None` as filler.
- If you specify any less than that, then the system does a loose check. So filtering for only `BombBug` won't care about the floor type that a bomb bug is on.

## Other examples

Various library examples are found in the `examples` folder, some using the `data` API while others use the `builder` API.

They can be run via `cargo run --bin NAME -- ARGS` whereas `NAME` is the folder name. `ARGS` vary per example, see the usage string inside.
