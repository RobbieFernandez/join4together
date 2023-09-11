use std::ops::Deref;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{grid, palette, sprites::SpriteWithPalette};

const TILE_SIZE: usize = 8;

#[derive(Debug)]
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
    num_cols: usize,
}

impl TileVec {
    pub fn new(vec: Vec<Tile4>, num_cols: usize, num_rows: usize) -> Self {
        Self {
            _vec: vec,
            num_cols,
            num_rows,
        }
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

impl From<Vec<u8>> for Tile4 {
    fn from(tile: Vec<u8>) -> Self {
        let num_pixels = tile.len();
        assert!(num_pixels == TILE_SIZE * TILE_SIZE);

        let tile = unflatten_tiles(tile)[0];
        let tile: [u32; 8] = tile
            .iter()
            .map(pack_tile_row)
            .collect::<Vec<u32>>()
            .try_into()
            .expect("Tile row is not the corect size.");

        Self { _data: tile }
    }
}

impl ToTokens for Tile4 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let hex_literals: Vec<String> = self.iter().map(|i| format!("{:#0x}", i)).collect();
        let hex_literals = hex_literals.join(", ");
        let hex_literals = format!("[ {} ]", hex_literals);
        let expr: syn::Expr = syn::parse_str(&hex_literals)
            .expect("Error producing hex representation of sprite tiles.");

        expr.to_tokens(tokens);
    }
}

impl ToTokens for TileVec {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let tiles: Vec<String> = self
            .iter()
            .map(|tile| quote! { #tile }.to_string())
            .collect();

        let tiles = tiles.join(",");
        let tiles = format!("&[ {} ]", tiles);

        let expr: syn::Expr = syn::parse_str(&tiles).unwrap();
        expr.to_tokens(tokens);
    }
}

/// Calculate the smallest valid dimensions the sprite can fit in.
///
/// Valid sizes are:
///
/// | Size | Square | Horizontal | Vertical |
/// |:-:|:-:|:-:|:-:|
/// | 0 | 8x8 | 16x8 | 8x16 |
/// | 1 | 16x16 | 32x8 | 8x32 |
/// | 2 | 32x32 | 32x16 | 16x32 |
/// | 3 | 64x64 | 64x32 | 32x64 |
fn get_padded_sprite_dimensions(width: usize, height: usize) -> (usize, usize) {
    // Surely there's a smarter way to do this. Oh well
    let mut sizes = [
        (8, 8),
        (16, 8),
        (8, 16),
        (16, 16),
        (32, 8),
        (8, 32),
        (32, 32),
        (32, 16),
        (16, 32),
        (64, 64),
        (64, 32),
        (32, 64),
    ];

    sizes.sort_by_key(|s| s.0 * s.1);

    *sizes
        .iter()
        .find(|s| s.0 >= width && s.1 >= height)
        .expect("Sprite image is too large.")
}

/// Convert from the original Sprite representation, to a 1-dimensional vector of Tiles.
pub fn convert_sprite_to_tiles(
    sprite: &SpriteWithPalette,
    mapped_palette: &palette::MappedPalette,
) -> Vec<TileVec> {
    let mut tile_vecs: Vec<TileVec> = Vec::new();

    for i in 0..sprite.num_frames {
        let image_data: &Vec<u8> = sprite.image_data.get(i).unwrap();

        // First re-map the palette indices so that we know 0 == transparent during the padding.
        let converted_image_data: Vec<u8> = image_data
            .iter()
            .map(|i| mapped_palette.map_index(*i))
            .collect();

        // Sprites sizes must be powers of 2, and cannot be less than TILE_SIZE.

        // Sprites can only be specific sizes. These are:
        //     Square: 8x8   16x16   32x32   64x64
        //     Horiz:  16x8  32x8    32x16   64x32
        //     Vert:   8x16  8x32    16x32   32x64
        let (padded_width, padded_height) =
            get_padded_sprite_dimensions(sprite.width, sprite.height);

        let converted_image_data = grid::resize_grid(
            converted_image_data,
            sprite.width,
            sprite.height,
            padded_width,
            padded_height,
        );

        // Convert 1d index array into a vector of 2d tiles
        let (tiles, n_cols, n_rows) =
            flat_image_matrix_to_flat_tiles(&converted_image_data, padded_width, padded_height);
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

        let tile_vec = TileVec::new(tile_data, n_cols, n_rows);
        tile_vecs.push(tile_vec);
    }

    tile_vecs
}

/// From a 1-dimensional representation of an indexed image, create a 1-dimensional
/// representation of the tiles that the image is made from.
fn flat_image_matrix_to_flat_tiles(
    flat: &[u8],
    width: usize,
    height: usize,
) -> (Vec<u8>, usize, usize) {
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
            panic!("Tile contains index that cannot fit into 4 bits.");
        }

        // Within the byte, the left pixel's index will occupy the LSB
        let byte = left_index | (right_index << 4);
        let byte: u32 = byte.into();

        // The left pixels occupy the LSB, so shift left as we go.
        packed |= byte << (i * 8);
    }

    packed
}
