use std::{cmp::max, fs::read_dir, path::Path};

use asefile::{util, AsepriteFile};

use crate::{
    grid,
    palette::Palette,
    tiles::{Tile4, TileVec},
};

pub mod codegen;

#[derive(Debug)]
pub struct BackgroundError(&'static str);

pub struct Background {
    name: String,
    tileset: Tileset,
    tilemap: Tilemap,
}

pub struct Tileset {
    tiles: TileVec,
    palette: Palette,
}

pub struct Tilemap {
    width: usize,
    height: usize,
    indices: Vec<usize>,
}

pub fn find_backgrounds(directory: &Path) -> Result<Vec<Background>, BackgroundError> {
    // Find all .aseprite files in the directory
    let entries =
        read_dir(directory).map_err(|_e| BackgroundError("Failed to read background dir."))?;
    let mut backgrounds: Vec<Background> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|_e| BackgroundError("Failed to read background file."))?;
        let path = entry.path();

        assert!(
            path.is_file(),
            "/background dir cannot contain nested directories."
        );

        let filename = entry
            .file_name()
            .to_ascii_uppercase()
            .into_string()
            .map_err(|_e| BackgroundError("Failed to construct background filename."))?;

        let filename = filename.replace(".ASEPRITE", "");

        let ase = AsepriteFile::read_file(&path)
            .map_err(|_e| BackgroundError("Failed to parse background file."))?;
        let bg = parse_background(&ase, filename)?;

        backgrounds.push(bg);
    }
    Ok(backgrounds)
}

fn parse_background(
    ase_file: &AsepriteFile,
    filename: String,
) -> Result<Background, BackgroundError> {
    let layers = ase_file.layers();
    let mut layers = layers.filter(|l| l.is_tilemap());

    let layer = layers.next().ok_or(BackgroundError(
        "Background does not contain any tilemap layers.",
    ))?;

    let ase_tilemap = ase_file.tilemap(layer.id(), 0).unwrap();
    let converted_tileset = convert_tileset(ase_file, ase_tilemap.tileset())?;
    let converted_tilemap = convert_tilemap(&ase_tilemap)?;

    let bg = Background {
        name: filename,
        tileset: converted_tileset,
        tilemap: converted_tilemap,
    };
    Ok(bg)
}

fn convert_tileset(
    ase_file: &AsepriteFile,
    tileset: &asefile::Tileset,
) -> Result<Tileset, BackgroundError> {
    // TODO - Actually we do need to remap the palette to make sure this is zero.
    //        For now just assert that it already is.
    let transparency_index = ase_file.transparent_color_index();
    if let Some(transparency_index) = transparency_index {
        assert!(transparency_index == 0);
    }

    let raw_palette = ase_file
        .palette()
        .ok_or(BackgroundError("Background does not have a palette."))?;

    let palette: Palette = raw_palette.into();

    let mapper = util::PaletteMapper::new(
        raw_palette,
        util::MappingOptions {
            transparent: transparency_index,
            failure: 0,
        },
    );

    let num_tiles = tileset.tile_count();

    let tiles: Vec<Tile4> = (0..num_tiles)
        .map(|tile_index| {
            // Convert this tile into a Tile4
            // There is no palette conversion for backgrounds so we just need to pack the raw indices.
            let tile_img = tileset.tile_image(tile_index);
            let (_, frame_image_data) = util::to_indexed_image(tile_img, &mapper);

            // Convert to indexed image
            let tile: Tile4 = frame_image_data.into();
            tile
        })
        .collect();

    let num_tiles = tiles.len();
    let tiles = TileVec::new(tiles, num_tiles, 1);

    Ok(Tileset { palette, tiles })
}

fn convert_tilemap(tilemap: &asefile::Tilemap) -> Result<Tilemap, BackgroundError> {
    let mut indices: Vec<usize> = Vec::new();

    for row in 0..tilemap.height() {
        for col in 0..tilemap.width() {
            let tile = tilemap.tile(col, row);
            let index: usize = tile.id().try_into().unwrap();
            indices.push(index);
        }
    }

    let width: usize = tilemap.width().try_into().unwrap();
    let height: usize = tilemap.height().try_into().unwrap();

    // Tilemap dimensions can be either 32 or 64.
    let target_width = max(32, width.next_power_of_two());
    let target_height = max(32, height.next_power_of_two());

    if target_height > 64 || target_width > 64 {
        Err(BackgroundError("Background cannot exceed 64x64 tiles."))
    } else {
        let indices = grid::resize_grid(indices, width, height, target_width, target_height);

        Ok(Tilemap {
            indices,
            width: target_width,
            height: target_height,
        })
    }
}
