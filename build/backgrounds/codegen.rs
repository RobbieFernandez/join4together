use quote::{format_ident, quote, ToTokens};

use crate::backgrounds;

pub fn generate_background_struct_src(background: &backgrounds::Background) -> String {
    let ident = format_ident!("{}_BACKGROUND", background.name);
    let tiles = &background.tileset.tiles;
    let palette = &background.tileset.palette;
    let tilemap = &background.tilemap;

    let size = get_size_enum(tilemap.width, tilemap.height);

    quote! {
        static #ident: Background = Background {
            tileset: #tiles,
            tilemap: &#tilemap,
            palette: &#palette,
            size: #size
        };
    }
    .to_string()
}

impl ToTokens for backgrounds::Tilemap {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // Tilemap needs to serialize to an array of gba::TextEntry.
        let text_entries: Vec<String> = self
            .indices
            .iter()
            .map(|x| format!("gba::video::TextEntry::new().with_tile({})", x))
            .collect();

        let text_entries = text_entries.join(",");
        let text_entries = format!("[ {} ]", text_entries);
        let expr: syn::Expr = syn::parse_str(&text_entries).expect("Could not parse TextEntries");

        expr.to_tokens(tokens);
    }
}

fn get_size_enum(width: usize, height: usize) -> syn::Expr {
    let enum_expr = format!("BackgroundSize::Bg{}x{}", width, height);
    syn::parse_str(&enum_expr).expect("Could not parse width/height")
}
