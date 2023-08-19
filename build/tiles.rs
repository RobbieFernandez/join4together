use std::ops::Deref;

use crate::{palette, SpriteWithPalette};

const TILE_SIZE: usize = 8;

pub struct Tile4 {
    _data: [u32; TILE_SIZE],
}

impl Tile4 {
    fn new(data: [u32; TILE_SIZE]) -> Self {
        Self { _data: data }
    }
}

impl Deref for Tile4 {
    type Target = [u32; TILE_SIZE];

    fn deref(&self) -> &Self::Target {
        &self._data
    }
}
pub struct TileVec {
    _vec: Vec<Tile4>,
    num_rows: usize,
    num_cols: usize
}

impl TileVec {
    pub fn new(vec: Vec<Tile4>, num_rows: usize, num_cols: usize) -> Self {
        Self { _vec: vec, num_cols, num_rows }
    }

    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    pub fn num_cols(&self) -> usize {
        self.num_cols
    }
}

impl Deref for TileVec {
    type Target = Vec<Tile4>;

    fn deref(&self) -> &Self::Target {
        &self._vec
    }
}

/// Convert from the original Sprite representation, to a 1-dimensional vector of Tiles.
pub fn convert_sprite_to_tiles(
    sprite: &SpriteWithPalette,
    mapped_palette: &palette::MappedPalette,
) -> TileVec {
    // First re-map the palette indices so that we know 0 == transparent during the padding.
    let converted_image_data: Vec<u8> = sprite
        .image_data
        .iter()
        .map(|i| mapped_palette.map_index(*i))
        .collect();

    // Pad the vector to make sure the dimensions are divisible by 8
    let converted_image_data =
        align_image_vec_to_tiles(converted_image_data, sprite.width, sprite.height);

    // Convert 1d index array into a vector of 2d tiles
    let (tiles, n_cols, n_rows)  = flat_image_matrix_to_flat_tiles(&converted_image_data, sprite.width, sprite.height);
    let tiles = unflatten_tiles(tiles);

    // Now we can pack each tile into an array of u32s.
    let mut tile_data: Vec<Tile4> = Vec::new();

    for tile in tiles {
        let converted_tile: [u32; 8] = tile
            .iter()
            .map(pack_tile_row)
            .collect::<Vec<u32>>()
            .try_into()
            .expect("Tile row is not the corect size.");

        tile_data.push(Tile4::new(converted_tile));
    }

    TileVec::new(tile_data, n_cols, n_rows)
}

// Pad the input vector to make sure the dimensions of the image are divisible by 8.
// This is done by adding transparent pixels along the right and bottom edges as needed.
fn align_image_vec_to_tiles(image_vec: Vec<u8>, width: usize, height: usize) -> Vec<u8> {
    let mut aligned: Vec<u8> = Vec::new();

    let right_padding = match width % TILE_SIZE {
        0 => 0,
        n => TILE_SIZE - n,
    };

    let bottom_padding = match height % TILE_SIZE {
        0 => 0,
        n => TILE_SIZE - n,
    };

    if right_padding == 0 && bottom_padding == 0 {
        // Already aligned
        return image_vec;
    }

    let num_columns = width + right_padding;
    let num_rows = height + bottom_padding;

    // Add each row, with padding, into the aligned vec.
    for row in 0..height {
        let row_start = row * width;
        let mut row_slice = Vec::from(&image_vec[row_start..(row_start + width)]);
        row_slice.resize(num_columns, 0);
        aligned.append(&mut row_slice);
    }

    // Add any needed extra rows
    aligned.resize(num_rows * num_columns, 0);

    aligned
}

/// From a 1-dimensional representation of an indexed image, create a 1-dimensional
/// representation of the tiles that the image is made from.
fn flat_image_matrix_to_flat_tiles(flat: &[u8], width: usize, height: usize) -> (Vec<u8>, usize, usize) {
    let num_tile_cols = width / TILE_SIZE;
    let num_tile_rows = height / TILE_SIZE;

    let mut unwrapped_tiles: Vec<u8> = Vec::new();

    // Outer loop goes through each tile, left-to-right and top-to-bottom
    for tile_y in 0..num_tile_rows {
        for tile_x in 0..num_tile_cols {
            // Find where this tile begins in the entire index_array
            let tile_start = (tile_y * TILE_SIZE * width) + (tile_x * TILE_SIZE);

            // For each tile, go scanline by scanline to create the 1d tile array.
            for scanline in 0..TILE_SIZE {
                let scanline_offset = scanline * width;

                for x in 0..TILE_SIZE {
                    let i = tile_start + scanline_offset + x;
                    unwrapped_tiles.push(flat[i]);
                }
            }
        }
    }

    (unwrapped_tiles, num_tile_cols, num_tile_rows)
}

/// Convert a 1-dimensional representaion of image tiles into a vector of 2-dimensional tiles.
fn unflatten_tiles(flat_tiles: Vec<u8>) -> Vec<[[u8; TILE_SIZE]; TILE_SIZE]> {
    // How many entries a tile takes up with a flat representation.
    let tile_length: usize = TILE_SIZE * TILE_SIZE;
    let num_tiles = flat_tiles.len() / tile_length;

    let mut output_vec: Vec<[[u8; TILE_SIZE]; TILE_SIZE]> = Vec::with_capacity(num_tiles);

    for tile_number in 0..num_tiles {
        let tile_start = tile_number * tile_length;
        let tile_slice = &flat_tiles[tile_start..(tile_start + tile_length)];

        let mut unwrapped_tile = [[0u8; TILE_SIZE]; TILE_SIZE];

        for (i, tile_row) in unwrapped_tile.iter_mut().enumerate() {
            let row_start = i * TILE_SIZE;
            tile_row.clone_from_slice(&tile_slice[row_start..row_start + TILE_SIZE]);
        }

        output_vec.push(unwrapped_tile);
    }

    output_vec
}

// Pack a tile row (represented as 8 4-bit indices) into a single u32.
fn pack_tile_row(indices: &[u8; 8]) -> u32 {
    let mut packed: u32 = 0;
    let max_index = 0xF;

    for (i, chunk) in indices.chunks(2).enumerate() {
        let left_index = chunk[0];
        let right_index = chunk[1];

        if left_index > max_index || right_index > max_index {
            panic!("Tile data contains index that cannot fit into 4 bits.");
        }

        // Within the byte, the left pixel's index will occupy the LSB
        let byte = (left_index | right_index << 4) as u32;

        // The left pixels occupy the LSB, so shift left as we go.
        packed |= byte << (i * 8);
    }

    packed
}
