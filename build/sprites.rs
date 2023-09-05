use std::{fs::read_dir, path::Path};

use asefile::{util, AsepriteFile};

use crate::palette::{self, palette_entry_to_15bit_color};

pub mod codegen;

#[derive(Debug)]
pub struct SpriteError;

pub struct SpriteWithPalette {
    pub name: String,
    pub palette: palette::Palette,
    pub width: usize,
    pub height: usize,
    pub image_data: Vec<Vec<u8>>,
    pub transparency_index: Option<u8>,
    pub num_frames: usize,
}

pub fn find_sprites(directory: &Path) -> Result<Vec<SpriteWithPalette>, SpriteError> {
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

    let mut image_data: Vec<Vec<u8>> = Vec::new();

    let transparency_index = ase.transparent_color_index();

    let raw_palette = ase.palette().unwrap();

    let mapper = util::PaletteMapper::new(
        raw_palette,
        util::MappingOptions {
            transparent: transparency_index,
            failure: 0,
        },
    );

    let pal_vec: Vec<u16> = (0..raw_palette.num_colors())
        .map(|i| {
            let full_color = raw_palette
                .color(i)
                .expect("Palette index not found in palette.");

            // Convert each color into xbbbbggggrrrr
            palette_entry_to_15bit_color(full_color)
        })
        .collect();

    let palette = palette::Palette::new(pal_vec);

    let width = ase.width();
    let height = ase.height();

    let num_frames: usize = ase
        .num_frames()
        .try_into()
        .expect("Sprite contains too many frames.");

    for f in 0..num_frames {
        let f: u32 = f.try_into().unwrap();
        let img = ase.frame(f).image();

        // image_data is a 1-dimensional representation of a 2-dimensional matrix. Each
        // element in the array represents an index, which identifies the colour in the palette
        // belonging to that pixel.
        let (_, frame_image_data) = util::to_indexed_image(img, &mapper);
        image_data.push(frame_image_data);
    }

    SpriteWithPalette {
        name: filename,
        palette,
        image_data,
        transparency_index,
        num_frames,
        width,
        height,
    }
}
