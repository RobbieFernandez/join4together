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
    // struct needs to hold:
    //  palbank index
    //  tile data (4BPP pixels, indices into the palbank)
    //  size (measured in tiles)

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

    // TODO - Include sprite's dimensions (in tiles)
    quote! {
        static #struct_name: Sprite = Sprite {
            tiles: #tile_data,
            palette_bank: #palette_bank
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