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

Usage: `./leafs-odyssey-manipulator.exe foo bar` loads `foo.world` from the worlds directory and saves a manipulated version to `bar.world`.
Can also be run using `cargo run --release -- foo bar` instead.

If a room has a certain format in its **title**, it will be manipulated by the tool:

```
!command1,arg1,arg2,command2,arg1,arg2,arg3,...
```

Following commands are supported:

- `move,X,Y,Z` - moves this room by this many **tiles**. So to move a room by a full size, X must be a multiple of 24 and Y must be a multiple of 16. +X is east, +Y is south, +Z is up.
- `size,W,H` - changes the room's size to the given width/height. Rooms are 24x16 by default, but can be made smaller or bigger. Note that the full 24x16 tilemap is used regardless of this size, as the game expects it.
- `name,NAME` - sets the name of the room (may not contain a comma)

So for example, the following room title will move the room up by one full size, half its size, and give it a title:

```
!rename,Hello World,move,0,-16,0,resize,12,8
```

The world will also get `[MANIP]` suffixed in its title, so it can be distinguished in the world list.
The world's revision number, as well as each edited room's revision number, is incremented by one.
The world's GUID is **not** modified, which may impact existing player and replay data! **Create backups!**

## Examples

Various library examples are found in the `examples` folder, some using the `data` API while others use the `builder` API.

They can be run via `cargo run --bin NAME -- ARGS` whereas `NAME` is the folder name. `ARGS` vary per example, see the usage string inside.
