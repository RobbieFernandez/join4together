use std::{env, fs::read_dir, path::Path};

use asefile::{util, AsepriteFile};
use id_tree::{Node, TreeBuilder};
use palette::{add_palette, resolve_palette};

mod binpack;
mod codegen;
mod palette;
mod tiles;

#[derive(Debug)]
struct SpriteError;

pub struct SpriteWithPalette {
    name: String,
    palette: palette::Palette,
    width: usize,
    height: usize,
    image_data: Vec<u8>,
    transparency_index: Option<u8>,
}

fn find_sprites(directory: &Path) -> Result<Vec<SpriteWithPalette>, SpriteError> {
    // Find all .aseprite files in the directory
    let entries = read_dir(directory).map_err(|_e| SpriteError)?;
    let mut asefiles: Vec<SpriteWithPalette> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|_e| SpriteError)?;
        let path = entry.path();

        assert!(
            path.is_file(),
            "/sprites dir cannot contain nested directories."
        );

        let filename = entry
            .file_name()
            .to_ascii_uppercase()
            .into_string()
            .map_err(|_e| SpriteError)?;

        let filename = filename.replace(".ASEPRITE", "");

        let ase = AsepriteFile::read_file(&path).map_err(|_e| SpriteError)?;
        asefiles.push(extract_sprite_palette(ase, filename));
    }

    Ok(asefiles)
}

fn extract_sprite_palette(ase: AsepriteFile, filename: String) -> SpriteWithPalette {
    assert!(
        ase.is_indexed_color(),
        "Only indexed color mode can be used."
    );

    let img = ase.frame(0).image();

    let transparency_index = ase.transparent_color_index();

    let raw_palette = ase.palette().unwrap();

    let mapper = util::PaletteMapper::new(
        raw_palette,
        util::MappingOptions {
            transparent: transparency_index,
            failure: 0,
        },
    );

    // image_data is a 1-dimensional representation of a 2-dimensional matrix. Each
    // element in the array represents an index, which identifies the colour in the palette
    // belonging to that pixel.
    let ((width, height), image_data) = util::to_indexed_image(img, &mapper);

    let pal_vec: Vec<u16> = (0..raw_palette.num_colors())
        .map(|i| {
            let full_color = raw_palette
                .color(i)
                .expect("Palette index not found in palette.");

            // Convert each color into xbbbbggggrrrr

            // Convert each channel from 8-bit into 5-bit (shift right by 3)
            // Then wrap each channel in a u16 so we can combine them.
            let red: u16 = u16::from(full_color.red() >> 3);
            let green: u16 = u16::from(full_color.green() >> 3);
            let blue: u16 = u16::from(full_color.blue() >> 3);

            red | (green << 5) | (blue << 10)
        })
        .collect();

    let palette = palette::Palette::new(pal_vec);

    SpriteWithPalette {
        name: filename,
        palette,
        image_data,
        transparency_index,
        width: width.try_into().unwrap(),
        height: height.try_into().unwrap(),
    }
}

fn main() {
    println!("cargo:rerun-if-changed=sprites/");

    let base_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Error reading 'CARGO_MANIFEST_DIR' environment variable. ");

    let base_dir = Path::new(&base_dir);

    let sprite_dir = Path::new(&"sprites");
    let sprite_dir = base_dir.join(sprite_dir);

    let mut palette_tree = TreeBuilder::new()
        .with_root(Node::new(palette::Palette::new(vec![0])))
        .build();

    let sprites = find_sprites(&sprite_dir).unwrap();

    for sprite in &sprites {
        add_palette(&mut palette_tree, sprite.palette.clone());
    }

    let palette_mapper = resolve_palette(palette_tree);

    let palette_source: String =
        codegen::generate_palette_array_src(&palette_mapper.full_palette());

    let struct_definitions: Vec<String> = sprites
        .iter()
        .map(|s| codegen::generate_sprite_struct_src(s, &palette_mapper))
        .collect();

    let struct_definitions: String = struct_definitions.join("\n");

    let source = format!("{}\n{}", palette_source, struct_definitions);

    let output_path = env::var("OUT_DIR").expect("Error reading 'OUT_DIR' environment variable. ");

    let output_path = Path::new(&output_path);
    let output_path = output_path.join(Path::new("sprite_data.rs"));

    write_source(&source, &output_path)
}

fn write_source(source_text: &str, output_path: &Path) {
    let syntax_tree: syn::File =
        syn::parse_str(source_text).expect("Error parsing generated code.");

    let formatted = prettyplease::unparse(&syntax_tree);

    std::fs::write(output_path, formatted.as_bytes()).expect("Error writing file.");
}
