use array2d::Array2D;
use fastlem::{core::{parameters::TopographicalParameters, units::Length}, lem::generator::TerrainGenerator, models::surface::{builder::TerrainModel2DBulider, sites::Site2D, terrain::Terrain2D}};

use leafs_odyssey_data::data::*;

const POINTS: usize = 256;
const BOUNDS_MIN: Length = 0.0;
const BOUNDS_MAX: Length = 100.0;
const BOUNDS_RANGE: Length = BOUNDS_MAX - BOUNDS_MIN;

pub fn generate_terrain() -> Terrain2D {
    let model = TerrainModel2DBulider::from_random_sites(POINTS, Site2D::new(BOUNDS_MIN, BOUNDS_MIN), Site2D::new(BOUNDS_MAX, BOUNDS_MAX))
        .relaxate_sites(1)
        .unwrap()
        .build()
        .unwrap();

    let terrain_generator = TerrainGenerator::default()
        .set_model(model)
        .set_parameters(
            (0..POINTS)
                .map(|i| TopographicalParameters::default().set_erodibility(0.5 + (i as f64 / POINTS as f64)))
                .collect::<_>(),
        );

    let terrain = terrain_generator.generate().unwrap();
    terrain
}

pub fn terrain_to_rooms(terrain: Terrain2D, room_count: (usize, usize)) -> Array2D<LOStemContent> {
    let room_template = LOStemContent::TileMapEdit {
        id: 0,
        name: "".into(),
        width: 24,
        height: 16,
        layers: vec![],
        music: LOMusic::None,
        revision: 1,
    };
    let mut rooms = Array2D::filled_with(room_template, room_count.1, room_count.0);

    let room_size = (
        BOUNDS_RANGE as f64 / room_count.0 as f64,
        BOUNDS_RANGE as f64 / room_count.1 as f64
    );

    for (room_y, room_x) in rooms.indices_row_major() {
        let mut tiles = Array2D::filled_with(0.0, 16, 24);
        
        for (tile_y, tile_x) in tiles.indices_row_major() {
            let site = Site2D::new(
                BOUNDS_MIN + BOUNDS_RANGE * (room_x as f64 / room_count.0 as f64) + room_size.0 * (tile_x as f64 / 24.0),
                BOUNDS_MIN + BOUNDS_RANGE * (room_y as f64 / room_count.1 as f64) + room_size.1 * (tile_y as f64 / 16.0),
            );
            let elevation = terrain.get_elevation(&site).unwrap_or_default();
            //println!("room {}x{} @ {}x{} -> {}x{} -> {}", room_x, room_y, tile_x, tile_y, site.x, site.y, elevation);
            tiles.set(tile_y, tile_x, elevation).unwrap();
        }

        let i = room_x + room_y * room_count.0;
        let room = rooms.get_mut(room_y, room_x).unwrap();
        if let LOStemContent::TileMapEdit { id, name, layers, .. } = room {
            *id = (i+1) as u32;
            *name = format!("Room {}x{}x0", room_x, room_y).into();
            layers.append(&mut vec![
                // Floor
                LOLayer {
                    width: 24,
                    height: 16,
                    tiles: tiles.elements_row_major_iter()
                        .map(|elevation| elevation_to_tile(*elevation))
                        .collect(),
                },
                LOLayer {
                    width: 24,
                    height: 16,
                    tiles: vec![LOTile::None; 24*16],
                },
                LOLayer {
                    width: 24,
                    height: 16,
                    tiles: vec![LOTile::None; 24*16],
                },
                LOLayer {
                    width: 24,
                    height: 16,
                    tiles: vec![LOTile::None; 24*16]
                        .iter()
                        .enumerate()
                        .map(|(i, tile)| {
                            if room_x == room_count.0 / 2 && room_y == room_count.1 / 2 && i == 24*16/2 {
                                LOTile::StartPoint {
                                    direction: LODirection::Down,
                                }
                            } else {
                                tile.clone()
                            }
                        })
                        .collect(),
                },
                LOLayer {
                    width: 24,
                    height: 16,
                    tiles: vec![LOTile::None; 24*16],
                },
            ]);
        }
    }

    rooms
}

fn elevation_to_tile(elevation: f64) -> LOTile {
    match elevation {
        v if (..0.001).contains(&v) => LOTile::Water,
        v if (0.001..0.1).contains(&v) => LOTile::Sand,
        v if (0.1..1.0).contains(&v) => LOTile::Grass,
        v if (1.0..2.0).contains(&v) => LOTile::DirtPath,
        v if (2.0..3.0).contains(&v) => LOTile::Dirt,
        v if (3.0..).contains(&v) => LOTile::RoughStone,
        _ => panic!("Value out of range!"),
    }
}