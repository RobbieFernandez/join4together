use core::cmp::Ordering;

use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    palette,
    sprites::SpriteWithPalette,
    tiles::{convert_sprite_to_tiles, TileVec},
};

pub fn generate_palette_array_src(palette: &palette::Palette, var_prefix: &str) -> String {
    let ident = format_ident!("{}_PALETTE", var_prefix);

    quote! {
        pub static #ident: [gba::video::Color; 256] = #palette;
    }
    .to_string()
}

pub fn generate_sprite_struct_src(
    sprite: &SpriteWithPalette,
    palette_mapper: &palette::PaletteMapper,
) -> String {
    let mapped_palette = palette_mapper
        .map_palette(&sprite.palette, sprite.transparency_index)
        .expect("Failed to map palette.");

    let tile_vecs = convert_sprite_to_tiles(sprite, &mapped_palette);

    let num_frames = tile_vecs.len();

    // Generate sprite for each frame, as well as an animation referencing each frame.
    match num_frames.cmp(&1) {
        Ordering::Equal => {
            let struct_name = format_ident!("{}_SPRITE", sprite.name.to_string());
            let tile_vec = tile_vecs.get(0).unwrap();
            generate_sprite_struct_code(
                sprite,
                struct_name,
                tile_vec,
                mapped_palette.palette_bank(),
            )
        }
        Ordering::Greater => {
            let frame_structs: Vec<String> = tile_vecs
                .iter()
                .enumerate()
                .map(|(i, tile_vec)| {
                    let struct_name =
                        format_ident!("{}_FRAME_{}_SPRITE", sprite.name.to_string(), i);
                    generate_sprite_struct_code(
                        sprite,
                        struct_name,
                        tile_vec,
                        mapped_palette.palette_bank(),
                    )
                })
                .collect();

            let frame_structs = frame_structs.join("\n");

            let animation_struct_name = format_ident!("{}_ANIMATION", sprite.name.to_string());

            let anim_struct =
                generate_animation_struct_src(num_frames, animation_struct_name, |i| {
                    format_ident!("{}_FRAME_{}_SPRITE", sprite.name.to_string(), i)
                });

            format!("{}\n{}", frame_structs, anim_struct)
        }
        _ => {
            panic!("Failed to generate any tiles for the sprite.");
        }
    }
}

fn generate_sprite_struct_code(
    sprite: &SpriteWithPalette,
    struct_name: Ident,
    tile_data: &TileVec,
    palette_bank: u8,
) -> String {
    // Build token for the tile data literal first.
    // We have this as a Vec, but in the generated code it needs to be an array.
    let width = tile_data.num_cols() * 8;
    let height = tile_data.num_rows() * 8;

    let (shape, size) = match width.cmp(&height) {
        Ordering::Equal => {
            let shape = "ObjShape::Square";

            let size: u16 = match width {
                8 => 0,
                16 => 1,
                32 => 2,
                64 => 3,
                _ => panic!("Invalid tile size."),
            };

            (shape, size)
        }
        Ordering::Greater => {
            let shape = "ObjShape::Horizontal";

            let size: u16 = match width {
                16 => 0,
                32 => match height {
                    8 => 1,
                    16 => 2,
                    _ => panic!("Invalid tile size."),
                },
                64 => 3,
                _ => panic!("Invalid tile size"),
            };

            (shape, size)
        }
        Ordering::Less => {
            let shape = "ObjShape::Vertical";

            let size: u16 = match width {
                8 => match height {
                    16 => 0,
                    32 => 1,
                    _ => panic!("Invalid tile size."),
                },
                16 => 2,
                32 => 3,
                _ => panic!("Invalid tile size."),
            };

            (shape, size)
        }
    };

    let shape_type: syn::Type = syn::parse_str(shape).unwrap();

    let width = sprite.width;
    let height = sprite.height;

    quote! {
        pub static #struct_name: Sprite = Sprite {
            tiles: #tile_data,
            palette_bank: #palette_bank,
            shape: #shape_type,
            size: #size,
            width: #width,
            height: #height
        };
    }
    .to_string()
}

fn generate_animation_struct_src<F>(
    num_frames: usize,
    struct_name: Ident,
    frame_name_generator: F,
) -> String
where
    F: Fn(usize) -> Ident,
{
    let size = num_frames;

    let animation_struct_type = format!("Animation<{}>", size);
    let animation_struct_type: syn::Type = syn::parse_str(&animation_struct_type).unwrap();

    let frame_arr: Vec<String> = (0..num_frames)
        .map(|i| {
            let frame_identifer = frame_name_generator(i);
            quote!( &#frame_identifer ).to_string()
        })
        .collect();

    let frame_arr = frame_arr.join(", ");
    let frame_arr = format!("[ {} ]", frame_arr);
    let frame_arr: syn::Expr = syn::parse_str(&frame_arr).unwrap();

    // TODO - The tick rate needs to be determined from the file somehow.

    quote!(
        pub static #struct_name: #animation_struct_type = Animation {
            tick_rate: 6u8,
            sprites: #frame_arr
        };
    )
    .to_string()
}
