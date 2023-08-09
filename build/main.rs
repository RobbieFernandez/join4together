use std::{
    fs::{self, read_dir},
    path::Path, env,
};

use asefile::AsepriteFile;
use id_tree::{TreeBuilder, Node};
use palette::{add_palette, resolve_palette, Palette};

mod palette;

#[derive(Debug)]
struct SpriteError;

struct SpriteWithPalette {
    file: AsepriteFile,
    palette: palette::Palette
}

fn find_sprites(directory: &Path) -> Result<Vec<AsepriteFile>, SpriteError> {
    // Find all .aseprite files in the directory
    let entries = read_dir(directory).map_err(|_e| SpriteError)?;
    let mut asefiles: Vec<AsepriteFile> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|_e| SpriteError)?;
        let path = entry.path();

        assert!(path.is_file(), "/sprites dir cannot contain nested directories.");
        let ase = AsepriteFile::read_file(&path).map_err(|_e| SpriteError)?;
        asefiles.push(ase);
    }

    Ok(asefiles)
}

fn extract_sprite_palette(sprite: AsepriteFile) -> SpriteWithPalette {
    assert!(sprite.is_indexed_color(), "Only indexed color mode can be used.");

    let palette = sprite.palette().unwrap();
    
    assert!(palette.num_colors() < palette::PAL_BANK_SIZE.try_into().unwrap());

    let raw_colors = (0..palette.num_colors()).map(|i| palette.color(i).unwrap()).map(|c| {
        // Convert each color into xbbbbggggrrrr
        let red: u16 = (u16::from(c.red()) & 0xF8) >> 3;
        let green: u16 = (u16::from(c.green()) & 0xF8) >> 3;
        let blue: u16 = (u16::from(c.blue()) & 0xF8) >> 3;
        red | (green << 5) | (blue << 10)
    }).collect();

    let gba_palette = Palette::new(raw_colors);

    SpriteWithPalette { file:sprite, palette: gba_palette }
}   

fn main() {
    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let base_dir = Path::new(&base_dir);

    let sprite_dir = Path::new(&"sprites");
    let sprite_dir = base_dir.join(sprite_dir);

    let mut palette_tree = TreeBuilder::new().with_root(Node::new(palette::Palette::new(vec![0]))).build();

    let sprites = find_sprites(&sprite_dir).unwrap();
    let sprites: Vec<SpriteWithPalette> = sprites.into_iter().map(|s| extract_sprite_palette(s)).collect();
    
    for sprite in sprites {
        add_palette(&mut palette_tree, sprite.palette.clone());
    }

    let palette_mapper = resolve_palette(palette_tree);

    println!("cargo:warning={:?}", palette_mapper);

}
