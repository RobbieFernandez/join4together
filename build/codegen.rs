use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    palette,
    tiles::{convert_sprite_to_tiles, Tile4, TileVec},
    SpriteWithPalette,
};

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

    let tile_length = tile_vec.len();
    let struct_name = format_ident!("{}Sprite", sprite.name.to_string());
    let tile_type = "gba::video::Tile4";
    let palette_bank = mapped_palette.palette_bank();

    // TODO - Include sprite's dimensions (in tiles)
    quote! {
        static #struct_name: SpriteData {
            tiles: [ #tile_type : #tile_length ] = #tile_vec;
            palette_bank = #palette_bank;
        }
    }
    .to_string()
}

impl ToTokens for Tile4 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let hex_literals: Vec<String> = self.iter().map(|i| format!("{:x}", i)).collect();
        let hex_literals = hex_literals.join(", ");

        quote! { [ #hex_literals ]}.to_tokens(tokens);
    }
}

impl ToTokens for TileVec {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Convert from this vec into an array literal
        // (quote! { [ #hex_literals ]}).to_tokens(tokens);
    }
}
