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

## Other examples

Various library examples are found in the `examples` folder, some using the `data` API while others use the `builder` API.

They can be run via `cargo run --bin NAME -- ARGS` whereas `NAME` is the folder name. `ARGS` vary per example, see the usage string inside.
