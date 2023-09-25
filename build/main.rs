use std::{env, path::Path};

use id_tree::{Node, TreeBuilder};
use palette::{add_palette, resolve_palette};

use crate::sprites::find_sprites;

mod backgrounds;
mod binpack;
mod grid;
mod math;
mod palette;
mod sprites;
mod tiles;

fn main() {
    println!("cargo:rerun-if-changed=assets/");

    let base_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Error reading 'CARGO_MANIFEST_DIR' environment variable. ");

    let base_dir = Path::new(&base_dir);

    let output_dir = env::var("OUT_DIR").expect("Error reading 'OUT_DIR' environment variable. ");
    let output_dir = Path::new(&output_dir);

    // Generate sprite source code.
    let sprite_dir = Path::new(&"assets/sprites");
    let sprite_dir = base_dir.join(sprite_dir);
    let sprite_source = get_sprite_source(&sprite_dir);

    let sprite_output_file = output_dir.join(Path::new("sprite_data.rs"));
    write_source(&sprite_source, &sprite_output_file);

    // Generate background source code.
    let background_dir = Path::new(&"assets/backgrounds");
    let background_source = get_background_source(background_dir);
    let background_output_file = output_dir.join(Path::new("background_data.rs"));
    write_source(&background_source, &background_output_file);

    // Generate LUT source code.
    let lut_src = math::generate_lookup_table_src();
    let lut_output_file = output_dir.join(Path::new("lut_data.rs"));

    write_source(&lut_src, &lut_output_file);
}

fn get_sprite_source(sprite_dir: &Path) -> String {
    let mut palette_tree = TreeBuilder::new()
        .with_root(Node::new(palette::Palette::new(vec![0])))
        .build();

    let sprites = find_sprites(sprite_dir).unwrap();

    for sprite in &sprites {
        add_palette(&mut palette_tree, sprite.palette.clone());
    }

    let palette_mapper = resolve_palette(palette_tree);

    let palette_source: String =
        sprites::codegen::generate_palette_array_src(&palette_mapper.full_palette(), "OBJ");

    let struct_definitions: Vec<String> = sprites
        .iter()
        .map(|s| sprites::codegen::generate_sprite_struct_src(s, &palette_mapper))
        .collect();

    let struct_definitions: String = struct_definitions.join("\n");

    format!("{}\n{}", palette_source, struct_definitions)
}

fn get_background_source(background_dir: &Path) -> String {
    let backgrounds = backgrounds::find_backgrounds(background_dir);
    let backgrounds = backgrounds.expect("Error building backgrounds."); // TODO - Better error handling.

    let background_definitions: Vec<String> = backgrounds
        .iter()
        .map(backgrounds::codegen::generate_background_struct_src)
        .collect();

    background_definitions.join("")
}

fn write_source(source_text: &str, output_path: &Path) {
    let syntax_tree: syn::File =
        syn::parse_str(source_text).expect("Error parsing generated code.");

    let formatted = prettyplease::unparse(&syntax_tree);

    std::fs::write(output_path, formatted.as_bytes()).expect("Error writing file.");
}
