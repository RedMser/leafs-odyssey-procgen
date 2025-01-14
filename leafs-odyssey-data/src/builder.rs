use array2d::Array2D;

use crate::data::*;

const INVALID_GUID: &'static str = "00000000";

pub struct World {
    pub name: String,
    pub description: String,
    pub author: Author,
    pub guid: String,
    pub revision: u32,
    pub rooms: Vec<Room>,
    /// Default width for rooms. Used to compute room position in the world.
    pub room_width: u16,
    /// Default height for rooms. Used to compute room position in the world.
    pub room_height: u16,
    last_room_id: u32,
}

impl World {
    pub fn new() -> Self {
        Self {
            name: "untitled".into(),
            description: "(no description provided)".into(),
            author: Author::new("Player", &format!("{0}-{0}", INVALID_GUID)),
            guid: INVALID_GUID.into(),
            revision: 0,
            rooms: vec![],
            room_width: 24,
            room_height: 16,
            last_room_id: 0,
        }
    }

    pub fn new_room(&mut self, position: (i16, i16, i16)) -> Room {
        let position = (
            position.0 * self.room_width as i16,
            position.1 * self.room_height as i16,
            position.2,
        );
        let mut room = Room::new_sized(position, self.room_width, self.room_height);
        self.last_room_id += 1;
        room.id = self.last_room_id;
        room
    }

    pub fn with_metadata(mut self, name: &str, description: &str) -> Self {
        self.name = name.into();
        self.description = description.into();
        self
    }

    pub fn with_identity(mut self, guid: &str, author: Author) -> Self {
        self.guid = guid.into();
        self.author = author;
        self
    }
}

pub struct Author {
    pub name: String,
    pub guid: String,
}

impl Author {
    pub fn new(name: &str, guid: &str) -> Self {
        Self {
            name: name.into(),
            guid: guid.into()
        }
    }
}

pub struct Room {
    pub id: u32,
    pub name: String,
    pub position: (i16, i16, i16),
    pub width: u16,
    pub height: u16,
    pub music: LOMusic,
    pub revision: u32,
    pub tilemap: Tilemap,
}

impl Room {
    pub fn new(position: (i16, i16, i16)) -> Self {
        Self::new_sized(position, 24, 16)
    }

    pub fn new_sized(position: (i16, i16, i16), width: u16, height: u16) -> Self {
        Self {
            position,
            id: 0,
            name: "untitled".into(),
            width,
            height,
            music: LOMusic::None,
            revision: 0,
            tilemap: Tilemap::new(width, height),
        }
    }

    pub fn with_metadata(mut self, name: &str, music: LOMusic) -> Self {
        self.name = name.into();
        self.music = music;
        self
    }
}

pub struct Tilemap {
    pub layers: (
        Array2D<LOTile>,
        Array2D<LOTile>,
        Array2D<LOTile>,
        Array2D<LOTile>,
        Array2D<LOTile>,
    ),
}

impl Tilemap {
    /// Floors and Walls.
    pub const LAYER1: u8 = 0;
    /// Obstacles, Trapdoors, Ladders, Waypoints, Goal Star, Pressure Plates, Sacrifice Altars and Toggle Floors.
    pub const LAYER2: u8 = 1;
    /// Crumbly Walls, Toggle Doors and Monster Gates.
    pub const LAYER3: u8 = 2;
    /// Keys, Push Blocks, Start Point.
    pub const LAYER4: u8 = 3;
    /// Monsters, Key Doors, Statue Rubble, Poison Trail, Sign, Stacks and Toggle Switches.
    pub const LAYER5: u8 = 4;

    pub const LAYER_INDICES: std::ops::RangeInclusive<u8> = (Self::LAYER1..=Self::LAYER5);

    pub fn new(width: u16, height: u16) -> Self {
        Self {
            layers: (
                Array2D::filled_with(LOTile::Grass, height as usize, width as usize),
                Array2D::filled_with(LOTile::None, height as usize, width as usize),
                Array2D::filled_with(LOTile::None, height as usize, width as usize),
                Array2D::filled_with(LOTile::None, height as usize, width as usize),
                Array2D::filled_with(LOTile::None, height as usize, width as usize),
            ),
        }
    }

    pub fn get_width(&self) -> u16 {
        self.layers.0.num_columns() as u16
    }
    pub fn get_height(&self) -> u16 {
        self.layers.0.num_rows() as u16
    }

    pub fn select_value(&self, selection: bool) -> TileSelection {
        TileSelection::from_value(self.get_width() as usize, self.get_height() as usize, selection)
    }
    /// Full selection.
    pub fn select_all(&self) -> TileSelection {
        self.select_value(true)
    }
    /// Empty selection.
    pub fn select(&self) -> TileSelection {
        self.select_value(false)
    }

    pub fn write_on_layer(&mut self, layer: u8, tile: &LOTile, selection: &TileSelection) {
        let layer = self.get_layer_mut(layer);

        for ((row, col), value) in selection.bools.enumerate_row_major() {
            if *value {
                let _ = layer.set(row, col, tile.clone());
            }
        }
    }
    pub fn write_on_layer_if<F>(&mut self, layer: u8, tile: &LOTile, selection: &TileSelection, predicate: F)
        where F: Fn((usize, usize), &LOTile) -> bool {
        self.write_on_layer(layer, tile, &selection.clone().predicate_and(|x, y| {
            let iter_tile = self.layers.1.get(y, x).unwrap();
            predicate((x, y), &iter_tile)
        }));
    }
    pub fn write(&mut self, tile: &LOTile, selection: &TileSelection) {
        if tile.is_floor() {
            self.write_floor(tile, selection)
        } else if tile.is_wall() {
            self.write_wall(tile, selection)
        } else if tile.is_obstacle() || tile.is_puzzle_obstacle() {
            self.write_obstacle(tile, selection)
        } else if tile.is_trapdoor() {
            self.write_trapdoor(tile, selection)
        } else {
            self.write_puzzle_element(tile, selection);
        }
    }
    pub fn write_floor(&mut self, tile: &LOTile, selection: &TileSelection) {
        assert!(tile.is_floor());
        self.write_on_layer(Self::LAYER1, tile, selection);
    }
    pub fn write_wall(&mut self, tile: &LOTile, selection: &TileSelection) {
        assert!(tile.is_wall());
        self.write_on_layer(Self::LAYER1, tile, selection);
        self.write_on_layer_if(Self::LAYER2, &LOTile::None, selection, |_, iter_tile| {
            matches!(iter_tile, LOTile::Pillar) || !iter_tile.is_puzzle_obstacle()
        });
        self.write_on_layer(Self::LAYER3, &LOTile::None, selection);
        self.write_on_layer(Self::LAYER4, &LOTile::None, selection);
        self.write_on_layer(Self::LAYER5, &LOTile::None, selection);
    }
    pub fn write_obstacle(&mut self, tile: &LOTile, selection: &TileSelection) {
        assert!(tile.is_obstacle() || tile.is_puzzle_obstacle());
        self.write_on_layer(Self::LAYER2, tile, selection);
        self.write_on_layer(Self::LAYER3, &LOTile::None, selection);
        self.write_on_layer(Self::LAYER4, &LOTile::None, selection);
        self.write_on_layer(Self::LAYER5, &LOTile::None, selection);
    }
    pub fn write_trapdoor(&mut self, tile: &LOTile, selection: &TileSelection) {
        assert!(tile.is_trapdoor());
        let floors = tile.get_trapdoor_floors();
        let canonical_floor = &floors[0];
        // Write canonical floor if the current floor is not a valid trapdoor floor.
        self.write_on_layer_if(Self::LAYER1, canonical_floor, selection, |_, iter_tile| {
            !floors.iter().any(|floor_tile| iter_tile.same_type_as(floor_tile))
        });
        self.write_on_layer(Self::LAYER2, tile, selection);
    }
    pub fn write_puzzle_element(&mut self, tile: &LOTile, selection: &TileSelection) {
        let layer = if tile.is_puzzle_layer3() {
            Self::LAYER3
        } else if tile.is_puzzle_layer4() {
            Self::LAYER4
        } else if tile.is_puzzle_layer5() || tile.is_monster() {
            Self::LAYER5
        } else {
            panic!("Input {:?} is not a puzzle element", tile)
        };

        self.write_on_layer(layer, tile, selection);
        if tile.is_crumbly_wall() {
            self.write_on_layer(Self::LAYER4, &LOTile::None, selection);
            self.write_on_layer(Self::LAYER5, &LOTile::None, selection);
        }
    }

    pub fn get_layer(&self, layer: u8) -> &Array2D<LOTile> {
        match layer {
            Self::LAYER1 => &self.layers.0,
            Self::LAYER2 => &self.layers.1,
            Self::LAYER3 => &self.layers.2,
            Self::LAYER4 => &self.layers.3,
            Self::LAYER5 => &self.layers.4,
            _ => panic!("Layer out of bounds."),
        }
    }

    pub fn get_layer_mut(&mut self, layer: u8) -> &mut Array2D<LOTile> {
        match layer {
            Self::LAYER1 => &mut self.layers.0,
            Self::LAYER2 => &mut self.layers.1,
            Self::LAYER3 => &mut self.layers.2,
            Self::LAYER4 => &mut self.layers.3,
            Self::LAYER5 => &mut self.layers.4,
            _ => panic!("Layer out of bounds."),
        }
    }

    pub fn into_layers(self) -> Vec<LOLayer> {
        vec![
            Self::into_layer(self.layers.0),
            Self::into_layer(self.layers.1),
            Self::into_layer(self.layers.2),
            Self::into_layer(self.layers.3),
            Self::into_layer(self.layers.4),
        ]
    }

    fn into_layer(layer: Array2D<LOTile>) -> LOLayer {
        LOLayer {
            width: layer.num_columns() as u16,
            height: layer.num_rows() as u16,
            tiles: layer.as_row_major(),
        }
    }
}

#[derive(Clone)]
pub struct TileSelection {
    pub bools: Array2D<bool>,
}

impl From<(usize, usize)> for TileSelection {
    // TODO: this is really inefficient, maybe we should have multiple backends?
    fn from(value: (usize, usize)) -> Self {
        let mut selection = Self::new(value.0, value.1);
        let _ = selection.bools.set(value.1, value.0, true);
        selection
    }
}

impl From<(usize, usize, usize, usize)> for TileSelection {
    fn from(value: (usize, usize, usize, usize)) -> Self {
        Self::new(value.0+value.2, value.1+value.3)
            .set_rect(value.0, value.1, value.2, value.3, true)
    }
}

impl TileSelection {
    pub fn from_value(width: usize, height: usize, value: bool) -> Self {
        Self {
            bools: Array2D::filled_with(value, height, width)
        }
    }
    /// Full selection.
    pub fn all(width: usize, height: usize) -> Self {
        Self::from_value(width, height, true)
    }
    /// Empty selection.
    pub fn new(width: usize, height: usize) -> Self {
        Self::from_value(width, height, false)
    }

    pub fn get_width(&self) -> usize {
        self.bools.num_columns()
    }
    pub fn get_height(&self) -> usize {
        self.bools.num_rows()
    }
    pub fn get_selection(&self) -> impl DoubleEndedIterator<Item = (usize, usize)> + Clone + use<'_> {
        self.bools.enumerate_row_major()
            .filter_map(|((row, col), mask)| {
                if *mask {
                    Some((row, col))
                } else {
                    None
                }
            })
    }
    /* TODO: not sure how I can borrow "self.bools" but not "self"
    /// Every index in the area (does not matter if selected or unselected).
    pub fn get_area(&self) -> impl DoubleEndedIterator<Item = (usize, usize)> + Clone + use<'_> {
        self.bools.indices_row_major()
    }
    /// Every index in the area (does not matter if selected or unselected).
    pub fn get_bools(&self) -> impl DoubleEndedIterator<Item = ((usize, usize), &bool)> + Clone + use<'_> {
        self.bools.enumerate_row_major()
    }
    */

    pub fn invert(mut self, selection: TileSelection) -> Self {
        for (row, col) in selection.get_selection() {
            if let Some(value_ref) = self.bools.get_mut(row, col) {
                *value_ref = !*value_ref;
            }
        }
        self
    }
    pub fn invert_all(self) -> Self {
        let size = (self.get_width(), self.get_height());
        self.invert(
            TileSelection::all(size.0, size.1)
        )
    }

    pub fn set_all(mut self, value: bool) -> Self {
        for (row, col) in self.bools.indices_row_major() {
            let _ = self.bools.set(row, col, value);
        }
        self
    }
    pub fn add_all(self) -> Self {
        self.set_all(true)
    }
    pub fn remove_all(self) -> Self {
        self.set_all(false)
    }

    pub fn set(mut self, x: usize, y: usize, value: bool) -> Self {
        let _ = self.bools.set(y, x, value);
        self
    }
    pub fn add(self, x: usize, y: usize) -> Self {
        self.set(x, y, true)
    }
    pub fn remove(self, x: usize, y: usize) -> Self {
        self.set(x, y, false)
    }

    pub fn predicate<F>(mut self, predicate: F) -> Self
        where F: Fn((usize, usize), bool) -> bool {
        for (row, col) in self.bools.indices_row_major() {
            // Convert ref to value copy to avoid double borrow.
            let value = {
                let reference = self.bools.get(row, col);
                *reference.unwrap()
            };
            let result = predicate((col, row), value);
            if result != value {
                let _ = self.bools.set(row, col, result);
            }
        }
        self
    }

    pub fn predicate_and<F>(self, predicate: F) -> Self
        where F: Fn(usize, usize) -> bool {
        self.predicate(|(x, y), value| {
            if !value { return false; }
            predicate(x, y)
        })
    }

    pub fn predicate_or<F>(self, predicate: F) -> Self
        where F: Fn(usize, usize) -> bool {
        self.predicate(|(x, y), value| {
            if value { return true;}
            predicate(x, y)
        })
    }

    pub fn set_rect(mut self, x: usize, y: usize, width: usize, height: usize, value: bool) -> Self {
        for y in y..y+height {
            for x in x..x+width {
                let _ = self.bools.set(y, x, value);
            }
        }
        self
    }
    pub fn add_rect(self, x: usize, y: usize, width: usize, height: usize) -> Self {
        self.set_rect(x, y, width, height, true)
    }
    pub fn remove_rect(self, x: usize, y: usize, width: usize, height: usize) -> Self {
        self.set_rect(x, y, width, height, false)
    }
}

impl From<Room> for LOStemContent {
    fn from(value: Room) -> Self {
        Self::TileMapEdit {
            id: value.id,
            name: value.name.into(),
            width: value.width,
            height: value.height,
            layers: value.tilemap.into_layers(),
            music: value.music,
            revision: value.revision,
        }
    }
}

impl TryFrom<World> for LOWorld {
    type Error = String;

    fn try_from(value: World) -> Result<Self, Self::Error> {
        let room_count = value.rooms.len();

        let room_info = value
            .rooms
            .iter()
            .enumerate()
            .map(|(i, room)| LORoomInfo {
                id: (i + 1) as u32,
                width: room.width,
                height: room.height,
                x_position: room.position.0,
                y_position: room.position.1,
                z_position: room.position.2,
            })
            .collect();

        let author_guids = parse_guid(&value.author.guid)?;
        assert_eq!(author_guids.len(), 2);

        let mut stems = vec![LOStem::from_content(LOStemContent::TileZoneMap {
            room_info,
            name: value.name.into(),
            description: value.description.into(),
            author: value.author.name.into(),
            guid_world: parse_single_guid(&value.guid)?,
            guid_author1: author_guids[0],
            guid_author2: author_guids[1],
            world_revision: value.revision,
            _unknown4: 0,
        })];

        stems.append(&mut value.rooms
            .into_iter()
            .map(|room| LOStem::from_content(LOStemContent::from(room)))
            .collect());

        Ok(Self::new(
            LOZone {
                stem_offset: 0,
                stem_length: 0,
                rooms: vec![
                    LORoom {
                        stem_offset: 0,
                        stem_length: 0,
                    };
                    room_count
                ],
            },
            stems,
        ))
    }
}
