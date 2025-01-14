use std::{collections::HashMap, error::Error, fs, io::BufRead, path::Path};

use leafs_odyssey_data::{builder::*, data::*};

pub fn import_string<P: AsRef<Path>>(path: P, world: &mut World) -> Result<(), Box<dyn Error>> {
    let fd = fs::File::open_buffered(path)?;

    #[derive(Clone)]
    struct Layer {
        tile_stacks: Vec<Vec<LOTile>>,
        width: Option<usize>,
        height: usize,
    }

    enum Step {
        Metadata,
        Layer(Layer),
    }

    let mut metadata = HashMap::new();
    let mut layers = vec![];
    let mut step = Step::Metadata;
    for line in fd.lines() {
        let line = line?;
        match step {
            Step::Metadata => {
                if line.is_empty() {
                    step = Step::Layer(Layer {
                        tile_stacks: vec![],
                        width: None,
                        height: 0,
                    });
                } else {
                    let eq = line.find('=');
                    if let Some(eq) = eq {
                        let (key, value) = line.split_at(eq);
                        let value = value.trim_start_matches('=');
                        metadata.insert(String::from(key), String::from(value));
                    } else {
                        println!("WARNING: metadata line ignored because missing equals: {}", line);
                    }
                }
            },
            Step::Layer(ref mut layer) => {
                if line.is_empty() {
                    layers.push(layer.clone());
                    step = Step::Layer(Layer {
                        tile_stacks: vec![],
                        width: None,
                        height: 0,
                    });
                } else {
                    let row = line.split(' ');
                    let size_before = layer.tile_stacks.len();
                    layer.tile_stacks.extend(row.map(|tile| import_tile_stack(tile)));

                    if layer.width.is_none() {
                        let size_after = layer.tile_stacks.len();
                        layer.width = Some(size_after - size_before);
                    }
                    layer.height += 1;
                }
            },
        }
    }

    // TODO: create multiple rooms on X/Y as needed
    for (i, layer) in layers.iter().enumerate() {
        let position = (0, 0, i as i16);
        let mut room = world.new_room(position);

        if let Some(room_title) = metadata.get(&format!("RoomTitle({},{},{})", position.0, position.1, position.2)) {
            room.name = room_title.into();
        }
        if let Some(room_music) = metadata.get(&format!("RoomMusic({},{},{})", position.0, position.1, position.2)) {
            room.music = import_music(room_music);
        }

        let mut x = 0;
        let mut y = 0;
        let selection = room.tilemap.select();
        for tile_stack in &layer.tile_stacks {
            for tile in tile_stack {
                room.tilemap.write(&tile, &selection.clone().add(x, y));
            }
            x += 1;
            if x >= layer.width.unwrap() {
                x = 0;
                y += 1;
            }
            if y >= layer.height {
                break;
            }
        }
        world.rooms.push(room);
    }

    Ok(())
}

fn import_tile_stack(tiles: &str) -> Vec<LOTile> {
    tiles.split('+').map(|tile| import_tile(tile)).collect()
}

fn import_tile(tile: &str) -> LOTile {
    let tilemap: HashMap<&str, LOTile> =
        [("Grass", LOTile::Grass),
        ("Dirt", LOTile::Dirt),
        ("Path", LOTile::DirtPath),
        ("Sand", LOTile::Sand),
        ("Snow", LOTile::Snow),
        ("WallBlue", LOTile::Wall),
        ("WindowBlue", LOTile::WallWithWindow),
        ("WallWood", LOTile::WoodenWall),
        ("WindowWood", LOTile::WoodenWallWithWindow),
        ("WallBrick", LOTile::BrickWall),
        ("TrapdoorPit", LOTile::TrapdoorOverPit),
        ("TrapdoorWater", LOTile::TrapdoorOverWater),
        ("TrapdoorFire", LOTile::TrapdoorOverHotCoals),
        ("TrapdoorIce", LOTile::TrapdoorOverIce),
        ("TrapdoorPacific", LOTile::TrapdoorOverPacificFloor),
        ("KeyYellow", LOTile::PrimeKey),
        ("KeyGreen", LOTile::TerraKey),
        ("KeyBlue", LOTile::SkyKey),
        ("KeyRed", LOTile::InfernalKey),
        ("KeyPink", LOTile::StarKey),
        ("StartPos", LOTile::StartPoint { direction: LODirection::Down }),
        ("AngryEye", LOTile::AngryEye),
        ("BombBug", LOTile::BombBug { direction: LODirection::Down }),
        ("Statue", LOTile::Statue),
        ("Slug", LOTile::Slug { direction: LODirection::Down }),
        ("OvergrownGrass", LOTile::OvergrownGrass),
        ("RedFlowers", LOTile::RedFlowers),
        ("YellowFlowers", LOTile::YellowFlowers),
        ("DeadGrass", LOTile::DeadGrass),
        ("SnowyGrass", LOTile::SnowyGrass),
        ("WindowBrick", LOTile::BrickWallWithWindow),
        ("WallStone", LOTile::StoneBrickWall),
        ("WindowStone", LOTile::StoneBrickWallWithWindow),
        ("Cliff", LOTile::Cliff),
        ("RoughStone", LOTile::RoughStone),
        ("BrokenBlue", LOTile::CrumblyWall),
        ("BrokenBrick", LOTile::CrumblyBrickWall),
        ("BrokenWood", LOTile::CrumblyWoodenWall),
        ("BrokenStone", LOTile::CrumblyStoneBrickWall),
        ("HotCoals", LOTile::HotCoals),
        ("DoorYellow", LOTile::PrimeDoor),
        ("DoorGreen", LOTile::TerraDoor),
        ("DoorBlue", LOTile::SkyDoor),
        ("DoorRed", LOTile::InfernalDoor),
        ("DoorPink", LOTile::StarDoor),
        ("GoalStar", LOTile::GoalStar),
        ("FlyingSnake", LOTile::FlyingSnake { direction: LODirection::Down }),
        ("MonsterBlock", LOTile::MonsterBlock),
        ("StatueRubble", LOTile::StatueRubble),
        ("PoisonTrail", LOTile::PoisonTrail),
        ("Gravel", LOTile::Gravel),
        ("PineNeedles", LOTile::PineNeedles),
        ("WoodFloor", LOTile::WoodenFloor),
        ("StonePath", LOTile::CobblestonePath),
        ("TileFloor", LOTile::TileFloor),
        ("Bush", LOTile::Bush),
        ("PineTree", LOTile::PineTree),
        ("AutumnTree", LOTile::AutumnTree),
        ("Tree", LOTile::Tree),
        ("DeadTree", LOTile::DeadTree),
        ("Ice", LOTile::Ice),
        ("PacificFloor", LOTile::PacificFloor),
        ("BlockBarrier", LOTile::BlockBarrier),
        ("SteppingStone", LOTile::SteppingStone),
        ("Sign", LOTile::Sign { text: "you FOOL".into() }),
        ("PushBlock", LOTile::PushBlock),
        ("MultiPushBlock", LOTile::MultiPushBlock),
        ("PressurePlate", LOTile::PressurePlate { connections: vec![] }),
        ("SacrificeAltar", LOTile::SacrificeAltar { connections: vec![] }),
        ("ToggleSwitch", LOTile::ToggleSwitch { connections: vec![] }),
        ("MarbleFloor", LOTile::MarbleFloor),
        ("CobblestonePath", LOTile::CobblestonePath),
        ("Water", LOTile::Water),
        ("Space", LOTile::Space),
        ("Sky", LOTile::Sky),
        ("Pillar", LOTile::Pillar),
        ("WoodFence", LOTile::WoodenFence),
        ("IronFence", LOTile::IronFence),
        ("Rock", LOTile::Rock),
        ("Cattails", LOTile::Cattails),
        ("Waypoint", LOTile::Waypoint),
        ("LadderUp", LOTile::LadderUp),
        ("LadderDown", LOTile::LadderDown),
        ("MonsterGate", LOTile::MonsterGate),
        ("MonsterGateOpen", LOTile::InvertedMonsterGate),
        ("ToggleDoor", LOTile::ToggleDoorInitiallyClosed),
        ("ToggleDoorOpen", LOTile::ToggleDoorInitiallyOpen),
        ("ToggleFloor", LOTile::ToggleFloorInitiallyClosed),
        ("ToggleFloorOpen", LOTile::ToggleFloorInitiallyOpen),
        ("Cloud", LOTile::Cloud),
        ("Pit", LOTile::Pit),
        ("TallGrass", LOTile::TallGrass),
        ("Curtain", LOTile::Curtain),
        ("Lamppost", LOTile::Lamppost)]
        .iter().cloned().collect();
    tilemap.get(tile).unwrap().clone()
}

fn import_music(music: &str) -> LOMusic {
    let music_map: HashMap<&str, LOMusic> =
        [
            ("None", LOMusic::None),
            ("ThrowRock", LOMusic::ThrowRock),
            ("Outside", LOMusic::Outside),
            ("Quest", LOMusic::Quest),
            ("Crystal", LOMusic::Crystal),
            ("Peril", LOMusic::Peril),
            ("Marble", LOMusic::Marble),
            ("Descent", LOMusic::Descent),
            ("Aqua", LOMusic::Aqua),
            ("Beyond", LOMusic::Beyond),
            ("Shards", LOMusic::Shards),
            ("Superfluid", LOMusic::Superfluid),
            ("Dust", LOMusic::Dust),
            ("Dread", LOMusic::Dread),
            ("Aspire", LOMusic::Aspire),
            ("Rapture", LOMusic::Rapture),
        ]
        .iter().cloned().collect();
    music_map.get(music).unwrap().clone()
}