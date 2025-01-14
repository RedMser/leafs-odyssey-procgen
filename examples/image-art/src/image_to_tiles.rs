use std::{fs::File, path::Path, str::FromStr};

use array2d::Array2D;
use deltae::{DeltaE, LabValue};
use palette::{encoding::Srgb, rgb::Rgb, white_point::D65, FromColor, Lab};

use leafs_odyssey_data::data::*;

fn hex_to_lab(hex: &str) -> LabValue {
    let rgb = Rgb::<Srgb, u8>::from_str(hex).unwrap();
    let rgb = Rgb::<Srgb, f32>::from_components((rgb.red as f32 / 255.0, rgb.green as f32 / 255.0, rgb.blue as f32 / 255.0));
    let lab = Lab::<D65, f32>::from_color(rgb);
    LabValue::new(lab.l, lab.a, lab.b).unwrap()
}

pub fn image_to_tiles<P: AsRef<Path>>(path: P) -> Array2D<LOStemContent> {
    let color_map = vec![
        (hex_to_lab("#5ca23f"), LOTile::Grass),
        (hex_to_lab("#a67a48"), LOTile::Dirt),
        (hex_to_lab("#a9986d"), LOTile::DirtPath),
        (hex_to_lab("#f3eb7d"), LOTile::Sand),
        (hex_to_lab("#f2f9f9"), LOTile::Snow),
        (hex_to_lab("#4a7391"), LOTile::Wall),
        (hex_to_lab("#92afbd"), LOTile::WallWithWindow),
        (hex_to_lab("#764618"), LOTile::WoodenWall),
        (hex_to_lab("#926a2a"), LOTile::WoodenWallWithWindow),
        (hex_to_lab("#98564b"), LOTile::BrickWall),
        (hex_to_lab("#66ab44"), LOTile::OvergrownGrass),
        (hex_to_lab("#678724"), LOTile::RedFlowers),
        (hex_to_lab("#7db32a"), LOTile::YellowFlowers),
        (hex_to_lab("#979e58"), LOTile::DeadGrass),
        (hex_to_lab("#96c379"), LOTile::SnowyGrass),
        (hex_to_lab("#9f6a67"), LOTile::BrickWallWithWindow),
        (hex_to_lab("#767674"), LOTile::StoneBrickWall),
        (hex_to_lab("#85837d"), LOTile::StoneBrickWallWithWindow),
        (hex_to_lab("#825d35"), LOTile::Cliff),
        (hex_to_lab("#737272"), LOTile::RoughStone),
        (hex_to_lab("#898989"), LOTile::Gravel),
        (hex_to_lab("#65855a"), LOTile::PineNeedles),
        (hex_to_lab("#a5733e"), LOTile::WoodenFloor),
        (hex_to_lab("#959595"), LOTile::StoneFloor),
        (hex_to_lab("#999799"), LOTile::TileFloor),
        (hex_to_lab("#913718"), LOTile::HotCoals),
        (hex_to_lab("#82f4ff"), LOTile::Ice),
        (hex_to_lab("#a393a2"), LOTile::PacificFloor),
        (hex_to_lab("#aba89b"), LOTile::BlockBarrier),
        (hex_to_lab("#d0d0d0"), LOTile::MarbleFloor),
        (hex_to_lab("#a29f9a"), LOTile::CobblestonePath),
        (hex_to_lab("#3079c9"), LOTile::Water),
        (hex_to_lab("#020219"), LOTile::Space),
        (hex_to_lab("#b1e4ff"), LOTile::Sky),
        (hex_to_lab("#cbecfd"), LOTile::Cloud),
        (hex_to_lab("#3b2b19"), LOTile::Pit),
    ];

    let img_reader = File::open_buffered(path).expect("Failed to load image.");
    let img = image::load(img_reader, image::ImageFormat::Png).expect("Failed to load as PNG.");

    let width = img.width();
    let height = img.height();

    let room_count = (
        width / 24,
        height / 16,
    );

    let room_template = LOStemContent::TileMapEdit {
        id: 0,
        name: "untitled".into(),
        width: 24,
        height: 16,
        layers: vec![
            // Floor
            LOLayer {
                width: 24,
                height: 16,
                tiles: vec![LOTile::Space; 24*16],
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
                tiles: vec![LOTile::None; 24*16],
            },
            LOLayer {
                width: 24,
                height: 16,
                tiles: vec![LOTile::None; 24*16],
            },
        ],
        music: LOMusic::None,
        revision: 1,
    };
    let mut rooms = Array2D::filled_with(room_template, room_count.1 as usize, room_count.0 as usize);

    for (room_y, room_x) in rooms.indices_row_major() {
        let room = rooms.get_mut(room_y, room_x).unwrap();
        let i = room_x + room_y * room_count.0 as usize;
        if let LOStemContent::TileMapEdit { id, .. } = room {
            *id = (i+1) as u32;
        }
    }

    for (x, y, rgb) in img.into_rgb32f().enumerate_pixels() {
        let rgb = Rgb::<Srgb, f32>::from_components((rgb.0[0], rgb.0[1], rgb.0[2]));
        let lab = Lab::<D65, f32>::from_color(rgb);
        let lab = LabValue::new(lab.l, lab.a, lab.b).unwrap();
        let min_delta = color_map
            .iter()
            .map(|(comp, tile)| {
                let delta = DeltaE::new(lab, comp, deltae::DEMethod::DE2000);
                (*delta.value(), tile)
            })
            .min_by_key(|&(comp, _tile)| {
                let approx = (comp * 65536.0) as i32;
                approx
            })
            .unwrap().1;

        let xt = x % 24;
        let yt = y % 16;
        let xr = x / 24;
        let yr = y / 16;

        let room = rooms.get_mut(yr as usize, xr as usize).unwrap();
        if let LOStemContent::TileMapEdit { layers, .. } = room {
            layers[0].tiles[(xt + yt * 24) as usize] = min_delta.clone();
        }
    }

    rooms
}