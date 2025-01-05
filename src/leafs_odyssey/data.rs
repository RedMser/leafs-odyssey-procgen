use std::{collections::VecDeque, error::Error, io::{Seek, Write}};

use binrw::{binwrite, BinResult, BinWrite, NullString};

use crate::null_sink::NullSink;

pub static mut HACKY_LIST: Vec<u32> = vec![];

#[binwrite]
#[brw(little, magic = b"StFB")]
pub struct LOWorld {
    #[bw(ignore)]
    room_count: u32,

    #[bw(calc = 0)] // always
    unknown1: u32,
    zone_stem_offset: u32,
    #[bw(calc = room_count + 1)]
    room_count_plus1: u32,
    pub zone: LOZone,
    pub stems: Vec<LOStem>,
}

impl LOWorld {
    pub fn new(zone: LOZone, stems: Vec<LOStem>) -> Self {
        Self {
            room_count: zone.rooms.len() as u32,
            zone_stem_offset: 0,
            zone,
            stems,
        }
    }

    pub unsafe fn write_world<W: Write + Seek>(&mut self, write: &mut W) -> Result<(), Box<dyn Error>> {
        // First pass: store stem lengths/offsets
        let mut sink = NullSink::new();
        self.write(&mut sink)?;

        assert_eq!(HACKY_LIST.len() % 2, 0);

        let mut funky_list = HACKY_LIST
            .chunks_exact(2)
            .flat_map(|el| {
                [el[0] - 4, el[1]-el[0] + 4]
            })
            .collect::<VecDeque<u32>>();

        self.zone_stem_offset = funky_list.pop_front().unwrap();
        self.zone.stem_offset = self.zone_stem_offset;
        self.zone.stem_length = funky_list.pop_front().unwrap();

        for room in &mut self.zone.rooms {
            room.stem_offset = funky_list.pop_front().unwrap();
            room.stem_length = funky_list.pop_front().unwrap();
        }

        assert!(funky_list.is_empty());

        self.write(write)?;

        HACKY_LIST.clear();
        Ok(())
    }
}

#[binwrite]
#[brw(little)]
pub struct LOZone {
    #[bw(calc = 1007)] // always
    unknown2: u32,
    pub stem_offset: u32,
    pub stem_length: u32,
    #[bw(calc = 4)]
    magic_len: u8,
    #[bw(calc = b"zone".into())]
    magic: Vec<u8>,
    pub rooms: Vec<LORoom>,
}

#[binwrite]
#[brw(little)]
#[derive(Clone)]
pub struct LORoom {
    #[bw(calc = 1002)] // always
    unknown1: u32,
    pub stem_offset: u32,
    pub stem_length: u32,
    #[bw(calc = 4)]
    magic_len: u8,
    #[bw(calc = b"room".into())]
    magic: Vec<u8>,
}

#[binrw::writer(writer)]
fn remember_stream_position(
    _: &u32,
) -> BinResult<()> {
    unsafe {
        HACKY_LIST.push(writer.stream_position().unwrap() as u32);
    }
    Ok(())
}

#[binwrite]
#[bw(stream = stream)]
#[brw(little, magic = b"metS")]
pub struct LOStem {
    #[bw(write_with = remember_stream_position)]
    byte_offset: u32,
    pub name: NullString,
    pub content: LOStemContent,
    #[bw(write_with = remember_stream_position)]
    byte_length: u32,
}

impl LOStem {
    pub fn from_content(content: LOStemContent) -> Self {
        let name = match content {
            LOStemContent::TileZoneMap { .. } => "tile_zone_map",
            LOStemContent::TileMapEdit { .. } => "tilemap_edit",
        };

        Self {
            byte_length: 0,
            byte_offset: 0,
            name: name.into(),
            content,
        }
    }
}

const TILE_ZONE_MAP_UNKNOWN3: [u8; 0xA5] = [
    0x01, 0x00, 0x01, 0x09, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x22, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x23, 0x00, 0x00, 
    0x01, 0x00, 0x00, 0x00, 0x01, 0x0C, 0x00, 0x00, 0x00, 0x24, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 
    0x00, 0x08, 0x00, 0x00, 0x00, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 
    0x00, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 
    0x00, 0x00, 0x00, 0x35, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x06, 0x00, 0x00, 0x00, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x39, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 
];

const TILE_MAP_EDIT_UNKNOWN3: [u8; 0x19D] = [
    0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x1C, 0x00, 0x00, 0x00, 0x01, 0x00, 
    0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x20, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 
    0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x2D, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 
    0x0F, 0x00, 0x00, 0x00, 0x30, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 
    0x00, 0x00, 0x74, 0x6F, 0x67, 0x67, 0x6C, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 
    0x00, 0x02, 0x00, 0x00, 0x00, 0x60, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 
    0x00, 0x30, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x74, 
    0x6F, 0x67, 0x67, 0x6C, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x02, 0x00, 
    0x00, 0x00, 0x37, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x31, 0x00, 
    0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x02, 0x00, 0x00, 0x00, 0x05, 0x00, 
    0x00, 0x00, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 
    0x03, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x35, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0F, 0x00, 
    0x00, 0x00, 0x30, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 
    0x74, 0x6F, 0x67, 0x67, 0x6C, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x02, 
    0x00, 0x00, 0x00, 0x2A, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x20, 
    0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 
    0x00, 0x2B, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x20, 0x00, 0x02, 
    0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x62, 
    0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x37, 0x00, 0xFF, 0xFF, 0xFF, 
    0xFF, 0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x74, 0x69, 0x6C, 0x65, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x30, 0x00, 
    0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x74, 0x6F, 0x67, 0x67, 
    0x6C, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x20, 0x00, 0x02, 0x00, 0x00, 
    0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
];

#[binwrite]
#[brw(little)]
#[derive(Clone)]
pub enum LOStemContent {
    TileZoneMap {
        #[bw(calc = 0)] // always
        unknown1: u16,
        #[bw(calc = 1)] // always
        unknown2: u32,
        /// Name of the zone / world.
        name: NullString,
        #[bw(calc = TILE_ZONE_MAP_UNKNOWN3.to_vec())] // always
        unknown3: Vec<u8>,
        #[bw(calc = room_info.len() as u32)]
        room_count: u32,
        room_info: Vec<LORoomInfo>,
        #[bw(calc = 1)] // always (except for MoMA which has 6, trip to slumari has 17)
        unknown4: u32,
        description: NullString,
        author: NullString,
        #[bw(pad_after = 4)]
        guid_world: u32,
        #[bw(pad_after = 4)]
        guid_author1: u32,
        guid_author2: u32,
        world_revision: u32,
        #[bw(calc = room_info.len() as u32)]
        room_count_again: u32,
        unknown6: u32,
    },
    TileMapEdit {
        #[bw(calc = 0)] // always
        unknown1: u16,
        id: u32,
        /// Name of the map / room.
        name: NullString,
        /// Should be 24.
        width: u16,
        /// Should be 16.
        height: u16,
        #[bw(calc = TILE_MAP_EDIT_UNKNOWN3.to_vec())]
        unknown3: Vec<u8>,
        #[bw(calc = layers.len() as u32)]
        layer_count: u32,
        /// Should have 5 elements.
        layers: Vec<LOLayer>,
        music: LOMusic,
        /// Map / Room revision number
        revision: u32,
    },
}

#[binwrite]
#[brw(little, repr = i32)]
#[derive(Clone)]
pub enum LOMusic {
    None = -1,
    ThrowRock = 0,
    Outside = 1,
    Quest = 2,
    Crystal = 3,
    Peril = 4,
    Marble = 5,
    Descent = 6,
    Aqua = 7,
    Beyond = 8,
    Shards = 9,
    Superfluid = 10,
    Dust = 11,
    Dread = 12,
    Aspire = 13,
    Rapture = 14,
}

#[binwrite]
#[brw(little, repr = u32)]
#[derive(Clone)]
pub enum LODirection {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

#[binwrite]
#[brw(little)]
#[derive(Clone)]
pub enum LOTile {
    // Seems to be treated the same as magic 0x01, 0x04, 0x07, 0x63 (and possibly every invalid value above that?)
    #[bw(magic = 0x00u32)]
    None,

    // Floor Tiles (Layer 1)
    #[bw(magic = 0x0Au32)]
    Grass,
    #[bw(magic = 0x0Bu32)]
    Dirt,
    #[bw(magic = 0x3Du32)]
    DirtPath,
    #[bw(magic = 0x3Bu32)]
    Sand,
    #[bw(magic = 0x3Cu32)]
    Snow,
    #[bw(magic = 0x38u32)]
    OvergrownGrass,
    #[bw(magic = 0x39u32)]
    RedFlowers,
    #[bw(magic = 0x3Au32)]
    YellowFlowers,
    #[bw(magic = 0x40u32)]
    DeadGrass,
    #[bw(magic = 0x3Fu32)]
    SnowyGrass,
    #[bw(magic = 0x3Eu32)]
    Gravel,
    #[bw(magic = 0x41u32)]
    PineNeedles,
    #[bw(magic = 0x13u32)]
    WoodenFloor,
    #[bw(magic = 0x44u32)]
    StoneFloor,
    #[bw(magic = 0x45u32)]
    TileFloor,
    #[bw(magic = 0x46u32)]
    MarbleFloor,
    #[bw(magic = 0x4Bu32)]
    CobblestonePath,
    #[bw(magic = 0x0Cu32)]
    Water,
    #[bw(magic = 0x17u32)]
    Space,
    #[bw(magic = 0x50u32)]
    Sky,
    #[bw(magic = 0x51u32)]
    Cloud,
    #[bw(magic = 0x52u32)]
    Pit,

    // Walls (Layer 1)
    #[bw(magic = 0x02u32)]
    Wall,
    #[bw(magic = 0x05u32)]
    WallWithWindow,
    #[bw(magic = 0x48u32)]
    WoodenWall,
    #[bw(magic = 0x57u32)]
    WoodenWallWithWindow,
    #[bw(magic = 0x49u32)]
    BrickWall,
    #[bw(magic = 0x59u32)]
    BrickWallWithWindow,
    #[bw(magic = 0x4Au32)]
    StoneBrickWall,
    #[bw(magic = 0x58u32)]
    StoneBrickWallWithWindow,
    #[bw(magic = 0x5Du32)]
    Cliff,
    #[bw(magic = 0x5Eu32)]
    RoughStone,

    // Obstacles (Layer 2)
    #[bw(magic = 0x14u32)]
    Bush,
    #[bw(magic = 0x4Cu32)]
    PineTree,
    #[bw(magic = 0x4Du32)]
    AutumnTree,
    #[bw(magic = 0x4Eu32)]
    Tree,
    #[bw(magic = 0x4Fu32)]
    DeadTree,
    #[bw(magic = 0x36u32)]
    Pillar,
    #[bw(magic = 0x42u32)]
    WoodenFence,
    #[bw(magic = 0x43u32)]
    IronFence,
    #[bw(magic = 0x47u32)]
    Rock,
    #[bw(magic = 0x5Fu32)]
    Cattails,
    #[bw(magic = 0x33u32)]
    TallGrass,
    #[bw(magic = 0x53u32)]
    Curtain,
    #[bw(magic = 0x61u32)]
    Lamppost,

    // Puzzle Elements on Layer 1
    #[bw(magic = 0x10u32)]
    HotCoals,
    #[bw(magic = 0x12u32)]
    Ice,
    #[bw(magic = 0x34u32)]
    PacificFloor,
    #[bw(magic = 0x35u32)]
    BlockBarrier,

    // Puzzle Elements on Layer 2
    #[bw(magic = 0x11u32)]
    SteppingStone,
    #[bw(magic = 0x1Du32)]
    Waypoint,
    #[bw(magic = 0x19u32)]
    LadderUp,
    #[bw(magic = 0x1Au32)]
    LadderDown,
    #[bw(magic = 0x26u32)]
    TrapdoorOverPit,
    #[bw(magic = 0x27u32)]
    TrapdoorOverWater,
    #[bw(magic = 0x5Au32)]
    TrapdoorOverHotCoals,
    #[bw(magic = 0x5Bu32)]
    TrapdoorOverIce,
    #[bw(magic = 0x5Cu32)]
    TrapdoorOverPacificFloor,
    #[bw(magic = 0x18u32)]
    GoalStar,
    #[bw(magic = 0x2Du32)]
    PressurePlate {
        #[bw(calc = connections.len() as u32)]
        connections_count: u32,
        connections: Vec<LOConnection>,
    },
    #[bw(magic = 0x60u32)]
    SacrificeAltar {
        #[bw(calc = connections.len() as u32)]
        connections_count: u32,
        connections: Vec<LOConnection>,
    },
    #[bw(magic = 0x31u32)]
    ToggleFloorInitiallyClosed,
    #[bw(magic = 0x32u32)]
    ToggleFloorInitiallyOpen,

    // Puzzle Elements on Layer 3
    #[bw(magic = 0x06u32)]
    CrumblyWall,
    #[bw(magic = 0x54u32)]
    CrumblyBrickWall,
    #[bw(magic = 0x55u32)]
    CrumblyWoodenWall,
    #[bw(magic = 0x56u32)]
    CrumblyStoneBrickWall,
    #[bw(magic = 0x0Du32)]
    MonsterGate,
    #[bw(magic = 0x16u32)]
    InvertedMonsterGate,
    #[bw(magic = 0x2Eu32)]
    ToggleDoorInitiallyClosed,
    #[bw(magic = 0x2Fu32)]
    ToggleDoorInitiallyOpen,

    // Puzzle Elements on Layer 4
    #[bw(magic = 0x0Eu32)]
    PrimeKey,
    #[bw(magic = 0x1Eu32)]
    TerraKey,
    #[bw(magic = 0x22u32)]
    SkyKey,
    #[bw(magic = 0x20u32)]
    InfernalKey,
    #[bw(magic = 0x24u32)]
    StarKey,
    #[bw(magic = 0x08u32)]
    PushBlock,
    #[bw(magic = 0x09u32)]
    MultiPushBlock,
    #[bw(magic = 0x29u32)]
    MonsterBlock,
    #[bw(magic = 0x03u32)]
    StartPoint {
        direction: LODirection,
    },

    // Puzzle Elements on Layer 5
    #[bw(magic = 0x0Fu32)]
    PrimeDoor,
    #[bw(magic = 0x1Fu32)]
    TerraDoor,
    #[bw(magic = 0x23u32)]
    SkyDoor,
    #[bw(magic = 0x21u32)]
    InfernalDoor,
    #[bw(magic = 0x25u32)]
    StarDoor,
    #[bw(magic = 0x28u32)]
    StatueRubble,
    #[bw(magic = 0x2Cu32)]
    PoisonTrail,
    #[bw(magic = 0x37u32)]
    Sign {
        text: NullString,
    },
    #[bw(magic = 0x62u32)]
    Stack {
        #[bw(calc = tiles.len() as u32)]
        tiles_count: u32,
        tiles: Vec<LOStackElement>,
    },
    #[bw(magic = 0x30u32)]
    ToggleSwitch {
        #[bw(calc = connections.len() as u32)]
        connections_count: u32,
        connections: Vec<LOConnection>,
    },

    // Monsters (Layer 5)
    #[bw(magic = 0x15u32)]
    AngryEye,
    #[bw(magic = 0x1Cu32)]
    BombBug {
        direction: LODirection,
    },
    #[bw(magic = 0x1Bu32)]
    Statue,
    #[bw(magic = 0x2Bu32)]
    Slug {
        direction: LODirection,
    },
    #[bw(magic = 0x2Au32)]
    FlyingSnake {
        direction: LODirection,
    },

    /// To try out stuff for yourself.
    Custom {
        id: u32,
        custom_data: Vec<u8>,
    },
}

impl LOTile {
    pub fn same_type_as(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    /// Floor Tiles (Layer 1)
    pub fn is_floor(&self) -> bool {
        matches!(
            self,
            Self::Grass
                | Self::Dirt
                | Self::DirtPath
                | Self::Sand
                | Self::Snow
                | Self::OvergrownGrass
                | Self::RedFlowers
                | Self::YellowFlowers
                | Self::DeadGrass
                | Self::SnowyGrass
                | Self::Gravel
                | Self::PineNeedles
                | Self::WoodenFloor
                | Self::StoneFloor
                | Self::TileFloor
                | Self::MarbleFloor
                | Self::CobblestonePath
                | Self::Water
                | Self::Space
                | Self::Sky
                | Self::Cloud
                | Self::Pit
        )
    }

    /// Walls (Layer 1)
    pub fn is_wall(&self) -> bool {
        matches!(
            self,
            Self::Wall
                | Self::WallWithWindow
                | Self::WoodenWall
                | Self::WoodenWallWithWindow
                | Self::BrickWall
                | Self::BrickWallWithWindow
                | Self::StoneBrickWall
                | Self::StoneBrickWallWithWindow
                | Self::Cliff
                | Self::RoughStone
        )
    }

    /// Obstacles (Layer 2)
    pub fn is_obstacle(&self) -> bool {
        matches!(
            self,
            Self::Bush
                | Self::PineTree
                | Self::AutumnTree
                | Self::Tree
                | Self::DeadTree
                | Self::Pillar
                | Self::WoodenFence
                | Self::IronFence
                | Self::Rock
                | Self::Cattails
                | Self::TallGrass
                | Self::Curtain
                | Self::Lamppost
        )
    }

    /// Puzzle Elements on Layer 1
    pub fn is_puzzle_floor(&self) -> bool {
        matches!(
            self,
            Self::HotCoals
                | Self::Ice
                | Self::PacificFloor
                | Self::BlockBarrier
        )
    }

    /// Puzzle Elements on Layer 2
    pub fn is_puzzle_obstacle(&self) -> bool {
        self.is_trapdoor() || matches!(
            self,
            Self::SteppingStone
                | Self::Waypoint
                | Self::LadderUp
                | Self::LadderDown
                | Self::GoalStar
                | Self::PressurePlate { .. }
                | Self::SacrificeAltar { .. }
                | Self::ToggleFloorInitiallyClosed
                | Self::ToggleFloorInitiallyOpen
        )
    }

    /// Layer 2
    pub fn is_trapdoor(&self) -> bool {
        matches!(
            self,
            Self::TrapdoorOverPit
                | Self::TrapdoorOverWater
                | Self::TrapdoorOverHotCoals
                | Self::TrapdoorOverIce
                | Self::TrapdoorOverPacificFloor
        )
    }

    /// Multiple options, but first is the "canonical" one.
    pub fn get_trapdoor_floors(&self) -> Vec<LOTile> {
        match self {
            LOTile::TrapdoorOverPit => vec![LOTile::Pit, LOTile::Space, LOTile::Sky, LOTile::Cloud],
            LOTile::TrapdoorOverWater => vec![LOTile::Water],
            LOTile::TrapdoorOverPacificFloor => vec![LOTile::PacificFloor],
            LOTile::TrapdoorOverHotCoals => vec![LOTile::HotCoals],
            LOTile::TrapdoorOverIce => vec![LOTile::Ice],
            _ => panic!("Input was not a trapdoor tile."),
        }
    }

    /// Puzzle Elements on Layer 3
    pub fn is_puzzle_layer3(&self) -> bool {
        self.is_crumbly_wall() || matches!(
            self,
            Self::MonsterGate
                | Self::InvertedMonsterGate
                | Self::ToggleDoorInitiallyClosed
                | Self::ToggleDoorInitiallyOpen
        )
    }

    // Layer 3.
    pub fn is_crumbly_wall(&self) -> bool {
        matches!(
            self,
            Self::CrumblyWall
                | Self::CrumblyBrickWall
                | Self::CrumblyWoodenWall
                | Self::CrumblyStoneBrickWall
        )
    }

    /// Puzzle Elements on Layer 4
    pub fn is_puzzle_layer4(&self) -> bool {
        matches!(
            self,
            Self::PrimeKey
                | Self::TerraKey
                | Self::SkyKey
                | Self::InfernalKey
                | Self::StarKey
                | Self::PushBlock
                | Self::MultiPushBlock
                | Self::MonsterBlock
                | Self::StartPoint { .. }
        )
    }

    /// Puzzle Elements on Layer 5
    pub fn is_puzzle_layer5(&self) -> bool {
        matches!(
            self,
            Self::PrimeDoor
                | Self::TerraDoor
                | Self::SkyDoor
                | Self::InfernalDoor
                | Self::StarDoor
                | Self::StatueRubble
                | Self::PoisonTrail
                | Self::Sign { .. }
                | Self::Stack { .. }
                | Self::ToggleSwitch { .. }
        )
    }

    /// Monsters (Layer 5)
    pub fn is_monster(&self) -> bool {
        matches!(
            self,
            Self::AngryEye
                | Self::BombBug { .. }
                | Self::Statue
                | Self::Slug { .. }
                | Self::FlyingSnake { .. }
        )
    }
}

#[binwrite]
#[brw(little)]
#[derive(Clone)]
pub struct LOConnection {
    pub x_position: u16,
    pub y_position: u16,
}

#[binwrite]
#[brw(little)]
#[derive(Clone)]
pub struct LOStackElement {
    pub tile: LOStackTile,
    pub direction: LOStackDirection,
    #[bw(if(matches!(tile, LOStackTile::ToggleSwitch)), calc = connections.len() as u32)]
    connections_count: u32,
    #[bw(if(matches!(tile, LOStackTile::ToggleSwitch)))]
    connections: Vec<LOConnection>,
}

#[binwrite]
#[brw(little, repr = u16)]
#[derive(Clone)]
pub enum LOStackDirection {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

#[binwrite]
#[brw(little)]
#[derive(Clone)]
pub enum LOStackTile {
    #[bw(magic = 0x00u16)]
    None,

    // Floor Tiles (Layer 1)
    /// TODO: what happens for any of these?
    #[bw(magic = 0x0Au16)]
    Grass,
    #[bw(magic = 0x0Bu16)]
    Dirt,
    #[bw(magic = 0x3Du16)]
    DirtPath,
    #[bw(magic = 0x3Bu16)]
    Sand,
    #[bw(magic = 0x3Cu16)]
    Snow,
    #[bw(magic = 0x38u16)]
    OvergrownGrass,
    #[bw(magic = 0x39u16)]
    RedFlowers,
    #[bw(magic = 0x3Au16)]
    YellowFlowers,
    #[bw(magic = 0x40u16)]
    DeadGrass,
    #[bw(magic = 0x3Fu16)]
    SnowyGrass,
    #[bw(magic = 0x3Eu16)]
    Gravel,
    #[bw(magic = 0x41u16)]
    PineNeedles,
    #[bw(magic = 0x13u16)]
    WoodenFloor,
    #[bw(magic = 0x44u16)]
    StoneFloor,
    #[bw(magic = 0x45u16)]
    TileFloor,
    #[bw(magic = 0x46u16)]
    MarbleFloor,
    #[bw(magic = 0x4Bu16)]
    CobblestonePath,
    #[bw(magic = 0x0Cu16)]
    Water,
    #[bw(magic = 0x17u16)]
    Space,
    #[bw(magic = 0x50u16)]
    Sky,
    #[bw(magic = 0x51u16)]
    Cloud,
    #[bw(magic = 0x52u16)]
    Pit,

    // Walls (Layer 1)
    /// TODO: what happens for any of these?
    #[bw(magic = 0x02u16)]
    Wall,
    #[bw(magic = 0x05u16)]
    WallWithWindow,
    #[bw(magic = 0x48u16)]
    WoodenWall,
    #[bw(magic = 0x57u16)]
    WoodenWallWithWindow,
    #[bw(magic = 0x49u16)]
    BrickWall,
    #[bw(magic = 0x59u16)]
    BrickWallWithWindow,
    #[bw(magic = 0x4Au16)]
    StoneBrickWall,
    #[bw(magic = 0x58u16)]
    StoneBrickWallWithWindow,
    #[bw(magic = 0x5Du16)]
    Cliff,
    #[bw(magic = 0x5Eu16)]
    RoughStone,

    // Obstacles (Layer 2)
    /// TODO: what happens for any of these?
    #[bw(magic = 0x14u16)]
    Bush,
    #[bw(magic = 0x4Cu16)]
    PineTree,
    #[bw(magic = 0x4Du16)]
    AutumnTree,
    #[bw(magic = 0x4Eu16)]
    Tree,
    #[bw(magic = 0x4Fu16)]
    DeadTree,
    #[bw(magic = 0x36u16)]
    Pillar,
    #[bw(magic = 0x42u16)]
    WoodenFence,
    #[bw(magic = 0x43u16)]
    IronFence,
    #[bw(magic = 0x47u16)]
    Rock,
    #[bw(magic = 0x5Fu16)]
    Cattails,
    #[bw(magic = 0x33u16)]
    TallGrass,
    #[bw(magic = 0x53u16)]
    Curtain,
    #[bw(magic = 0x61u16)]
    Lamppost,

    // Puzzle Elements on Layer 2
    /// TODO: what happens
    #[bw(magic = 0x11u16)]
    SteppingStone,
    /// TODO: what happens
    #[bw(magic = 0x1Du16)]
    Waypoint,
    /// Not officially supported, can not be used due to stack blocking the tile I assume.
    #[bw(magic = 0x19u16)]
    LadderUp,
    /// Not officially supported, can not be used due to stack blocking the tile I assume.
    #[bw(magic = 0x1Au16)]
    LadderDown,
    /// Not officially supported, but works as expected.
    #[bw(magic = 0x26u16)]
    TrapdoorOverPit,
    /// Not officially supported, but works as expected.
    #[bw(magic = 0x27u16)]
    TrapdoorOverWater,
    /// Not officially supported, but works as expected.
    #[bw(magic = 0x5Au16)]
    TrapdoorOverHotCoals,
    /// Not officially supported, but works as expected.
    #[bw(magic = 0x5Bu16)]
    TrapdoorOverIce,
    /// Not officially supported, but works as expected.
    #[bw(magic = 0x5Cu16)]
    TrapdoorOverPacificFloor,
    /// TODO: what happens
    #[bw(magic = 0x18u16)]
    GoalStar,

    // Puzzle Elements on Layer 3
    /// Not officially supported, but works as expected.
    #[bw(magic = 0x06u16)]
    CrumblyWall,
    /// Not officially supported, but works as expected.
    #[bw(magic = 0x54u16)]
    CrumblyBrickWall,
    /// Not officially supported, but works as expected.
    #[bw(magic = 0x55u16)]
    CrumblyWoodenWall,
    /// Not officially supported, but works as expected.
    #[bw(magic = 0x56u16)]
    CrumblyStoneBrickWall,

    // Puzzle Elements on Layer 4
    #[bw(magic = 0x0Eu16)]
    PrimeKey,
    #[bw(magic = 0x1Eu16)]
    TerraKey,
    #[bw(magic = 0x22u16)]
    SkyKey,
    #[bw(magic = 0x20u16)]
    InfernalKey,
    #[bw(magic = 0x24u16)]
    StarKey,
    #[bw(magic = 0x08u16)]
    PushBlock,
    #[bw(magic = 0x09u16)]
    MultiPushBlock,
    #[bw(magic = 0x29u16)]
    MonsterBlock,
    /// TODO: what happens
    #[bw(magic = 0x03u16)]
    StartPoint,

    // Puzzle Elements on Layer 5
    /// TODO: what happens
    #[bw(magic = 0x0Fu16)]
    PrimeDoor,
    /// TODO: what happens
    #[bw(magic = 0x1Fu16)]
    TerraDoor,
    /// TODO: what happens
    #[bw(magic = 0x23u16)]
    SkyDoor,
    /// TODO: what happens
    #[bw(magic = 0x21u16)]
    InfernalDoor,
    /// TODO: what happens
    #[bw(magic = 0x25u16)]
    StarDoor,
    #[bw(magic = 0x28u16)]
    StatueRubble,
    /// TODO: what happens
    #[bw(magic = 0x2Cu16)]
    PoisonTrail,
    /// TODO: what happens
    #[bw(magic = 0x37u16)]
    Sign,
    /// TODO: what happens
    #[bw(magic = 0x62u16)]
    Stack,
    #[bw(magic = 0x30u16)]
    ToggleSwitch,

    // Monsters (Layer 5)
    #[bw(magic = 0x15u16)]
    AngryEye,
    #[bw(magic = 0x1Cu16)]
    BombBug,
    #[bw(magic = 0x1Bu16)]
    Statue,
    #[bw(magic = 0x2Bu16)]
    Slug,
    #[bw(magic = 0x2Au16)]
    FlyingSnake,

    // To try out stuff for yourself.
    Custom {
        id: u16,
    },
}

#[binwrite]
#[brw(little)]
#[derive(Clone)]
pub struct LOLayer {
    #[bw(calc = vec![0; 5])]
    unknown2: Vec<u8>,
    pub width: u16,
    pub height: u16,
    #[bw(calc = vec![1, 1, 0, 0, 0x80, 0x3F])]
    unknown1: Vec<u8>,
    #[bw(calc = (*width as u32) * (*height as u32))]
    tile_count: u32,
    #[bw(assert(tiles.len() as u32 == tile_count))]
    pub tiles: Vec<LOTile>,
}

#[binwrite]
#[brw(little)]
#[derive(Clone)]
pub struct LORoomInfo {
    pub id: u32,
    /// In tiles (should be multiple of 24).
    pub x_position: i16,
    /// In tiles (should be multiple of 16).
    pub y_position: i16,
    /// Should be 24.
    pub width: u16,
    /// Should be 16.
    pub height: u16,
    pub z_position: i16,
    #[bw(calc = 0)] // always
    unknown6: u16,
}

/// Given a GUID as provided by the game UI, converts it to fields that can be put into structs.
/// Segments are split by dashes, so each dash causes the returning Vec to return one more element.
pub fn parse_guid(guid: &str) -> Result<Vec<u32>, String> {
    guid.split('-')
        .map(|segment| parse_single_guid(segment))
        .collect()
}

/// Given a GUID segment as provided by the game UI, converts it to fields that can be put into structs.
/// Segments are split by dashes.
pub fn parse_single_guid(guid: &str) -> Result<u32, String> {
    u32::from_str_radix(guid, 16)
        .map_err(|e| format!("Invalid GUID segment '{}': {}", guid, e))
}
