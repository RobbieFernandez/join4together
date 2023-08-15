use std::ops::Deref;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::SpriteWithPalette;

// type Tile4 = [u32; 8];

const TILE_SIZE: usize = 8;
const TILE_SIZE_U32: u32 = TILE_SIZE as u32;

struct Tile4 {
    _data: [u32; 8],
}

impl Tile4 {
    fn new(data: [u32; 8]) -> Self {
        Self { _data: data }
    }
}

pub fn generate_sprite_struct_src(
    sprite: &SpriteWithPalette,
    palette_mapper: &crate::palette::PaletteMapper,
) -> String {
    // struct needs to hold:
    //  palbank index
    //  tile data (4BPP pixels, indices into the palbank)
    //  size (measured in tiles)

    // Convert 1d index array into an array of 1d tiles, whose palette has not yet been converted.
    let tiles = create_index_tiles(&sprite.image_data, sprite.width, sprite.height);

    // Now we can convert each tile.
    let mut tile_data: Vec<Tile4> = Vec::new();

    let mapped_palette = palette_mapper
        .map_palette(&sprite.palette, sprite.transparency_index)
        .expect("Failed to map palette.");

    for tile in tiles {
        let mut converted_tile: [u32; 8] = [0; 8];

        for (i, index_row) in tile.iter().enumerate() {
            // Re-map all of the palette indices.
            let mut row: [u8; 8] = [0; 8];
            for i in 0..index_row.len() {
                row[i] = mapped_palette.map_index(index_row[i]);
            }

            // Pack into a u32 and push it to the tile
            converted_tile[i] = pack_indices(row);
        }

        tile_data.push(Tile4::new(converted_tile));
    }

    generate_code(
        sprite,
        TileVec::new(tile_data),
        mapped_palette.palette_bank(),
    )
}

fn create_index_tiles(index_array: &Vec<u8>, width: u32, height: u32) -> Vec<[[u8; 8]; 8]> {
    let num_tile_cols = width / TILE_SIZE_U32;
    let num_tile_rows = height / TILE_SIZE_U32;

    let mut tiles: Vec<[[u8; 8]; 8]> = Vec::new();

    // Outer loop goes through each tile, left-to-right and top-to-bottom
    for tile_x in 0..num_tile_cols {
        for tile_y in 0..num_tile_rows {
            let mut tile: [[u8; 8]; 8] = [[0; 8]; 8];

            // Find where this tile begins in the entire index_array
            let tile_start = (tile_y * TILE_SIZE_U32 * width) + (tile_x * TILE_SIZE_U32);

            // For each tile, go scanline by scanline to create the 1d tile array.
            for scanline in 0..(TILE_SIZE as u32) {
                let mut scanline_array: [u8; 8] = [0; 8];
                let scanline_offset = scanline * width;

                for x in 0..TILE_SIZE_U32 {
                    let i = tile_start + scanline_offset + x;
                    scanline_array[x as usize] = index_array[i as usize];
                }

                tile[scanline as usize] = scanline_array;
            }

            // Now push that array into the final vec.
            tiles.push(tile);
        }
    }

    tiles
}

fn pack_indices(indices: [u8; 8]) -> u32 {
    // TODO
    0
}

fn generate_code(sprite: &SpriteWithPalette, tile_data: TileVec, palette_bank: u8) -> String {
    // Build token for the tile data literal first.
    // We have this as a Vec, but in the generated code it needs to be an array.

    let tile_length = tile_data.len();
    let struct_name = format_ident!("{}Sprite", sprite.name.to_string());
    let tile_type = "gba::video::Tile4";

    // TODO - Generate the required struct.
    //        Correct type identifier here.
    //        Include sprite size.
    quote! {
        static #struct_name: SpriteData {
            tiles: [ #tile_type : #tile_length ] = #tile_data;
            palette_bank = #palette_bank;
        }
    }
    .to_string()
}

impl ToTokens for Tile4 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let hex_literals: Vec<String> = self._data.iter().map(|i| format!("{:x}", i)).collect();
        let hex_literals = hex_literals.join(", ");

        quote! { [ #hex_literals ]}.to_tokens(tokens);
    }
}

struct TileVec {
    _vec: Vec<Tile4>,
}

impl TileVec {
    fn new(vec: Vec<Tile4>) -> Self {
        Self { _vec: vec }
    }
}

impl Deref for TileVec {
    type Target = Vec<Tile4>;

    fn deref(&self) -> &Self::Target {
        &self._vec
    }
}

impl ToTokens for TileVec {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Convert from this vec into an array literal
        // (quote! { [ #hex_literals ]}).to_tokens(tokens);
    }
}
