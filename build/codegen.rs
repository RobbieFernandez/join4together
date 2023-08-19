use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    palette,
    tiles::{convert_sprite_to_tiles, Tile4, TileVec},
    SpriteWithPalette,
};

pub fn generate_palette_array_src(
    palette: &palette::Palette
) -> String {
    quote! {
        static PALETTE: [u16; 256] = #palette;
    }.to_string()
}

pub fn generate_sprite_struct_src(
    sprite: &SpriteWithPalette,
    palette_mapper: &palette::PaletteMapper,
) -> String {
    let mapped_palette = palette_mapper
        .map_palette(&sprite.palette, sprite.transparency_index)
        .expect("Failed to map palette.");

    let tile_vec = convert_sprite_to_tiles(sprite, &mapped_palette);

    generate_sprite_code(
        sprite,
        tile_vec,
        mapped_palette.palette_bank(),
    )
}


fn generate_sprite_code(sprite: &SpriteWithPalette, tile_data: TileVec, palette_bank: u8) -> String {
    // Build token for the tile data literal first.
    // We have this as a Vec, but in the generated code it needs to be an array.

    let struct_name = format_ident!("{}_SPRITE", sprite.name.to_string());
    let width = tile_data.num_cols() * 8;
    let height = tile_data.num_rows() * 8;

    let (shape, size) = if width == height {
        let shape = "ObjShape::Square";

        let size = match width {
            8 => 0,
            16 => 1,
            32 => 2,
            64 => 3,
            _ => panic!("Invalid tile size.")
        };

        (shape, size)
    } else if width > height {
        let shape = "ObjShape::Horizontal";
        
        let size = match width {
            16 => 0,
            32 => {
                match height {
                    8 => 1,
                    16 => 2,
                    _ => panic!("Invalid tile size.")
                }
            },
            64 => 3,
            _ => panic!("Invalid tile size")
        };

        (shape, size)
    } else {
        let shape = "ObjShape::Vertical";

        let size = match width {
            8 => {
                match height {
                    16 => 0,
                    32 => 1,
                    _ => panic!("Invalid tile size.")
                }
            },
            16 => 2,
            32 => 3,
            _ => panic!("Invalid tile size.")
        };

        (shape, size)
    };
    let shape_type: syn::Type = syn::parse_str(&shape).unwrap();
    let size = size as u16;

    quote! {
        static #struct_name: Sprite = Sprite {
            tiles: #tile_data,
            palette_bank: #palette_bank,
            shape: #shape_type,
            size: #size
        };
    }
    .to_string()
}

impl ToTokens for Tile4 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let hex_literals: Vec<String> = self.iter().map(|i| format!("{:x}", i)).collect();
        let hex_literals = hex_literals.join(", ");
        let hex_literals = format!("[ {} ]", hex_literals);
        let expr: syn::Expr = syn::parse_str(&hex_literals).unwrap();

        expr.to_tokens(tokens);
    }
}

impl ToTokens for TileVec {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let tiles: Vec<String> = self.iter().map(|tile| {
            quote!{ #tile }.to_string()
        }).collect();
        
        let tiles = tiles.join(",");
        let tiles = format!("&[ {} ]", tiles);

        let expr: syn::Expr = syn::parse_str(&tiles).unwrap();
        expr.to_tokens(tokens);
    }
}

impl ToTokens for palette::Palette {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let tiles: Vec<String> = self.iter().map(|tile| {
            quote!{ #tile }.to_string()
        }).collect();
        
        let tiles = tiles.join(",");
        let tiles = format!("[ {} ]", tiles);

        let expr: syn::Expr = syn::parse_str(&tiles).unwrap();
        expr.to_tokens(tokens);
    }
}