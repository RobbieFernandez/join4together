use std::{
    fs::{self, read_dir},
    path::Path,
};

use asefile::AsepriteFile;
use id_tree::Tree;
use palette::{add_palette, resolve_palette, Palette};

mod palette;

fn find_sprites(directory: &Path) -> impl Iterator<Item = AsepriteFile> {
    // Find all .aseprite files in the directory
    let entries = read_dir(directory);

    Ok(entries
        .filter_map(|e| e.ok())
        .map(|e| AsepriteFile::read_file(&e.path())))
}

fn main() {
    let mut palette_tree: Tree<Palette> = Tree::new();

    let pal1 = Palette::new(vec![1u16, 2u16]);
    let palettes = vec![pal1];

    for pal in palettes {
        add_palette(&mut palette_tree, pal);
    }

    let palette_mapper = resolve_palette(palette_tree);
}
