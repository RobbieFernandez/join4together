use gba::{
    mmio::*,
    video::{BackgroundControl, Color, DisplayControl, TextEntry, Tile4},
};
use voladdress::{Safe, VolAddress};

use crate::system::{
    constants,
    gba::{ClaimedGridFrames, ClaimedVolRegion, GBA},
};

// How many tiles make up the dimensions of a screenblock.
const SCREENBLOCK_SIZE: usize = 32;

pub enum BackgroundSize {
    Bg32x32,
    Bg64x32,
    Bg32x64,
    Bg64x64,
}

#[derive(Copy, Clone)]
pub enum BackgroundLayer {
    Bg0,
    Bg1,
    Bg2,
    Bg3,
}

pub struct Background {
    tileset: &'static [Tile4],
    tilemap: &'static [TextEntry],
    palette: &'static [Color],
    size: BackgroundSize, //
}

pub struct LoadedBackground<'a> {
    background: &'a Background,
    // Memory for the tileset
    charblock_memory: ClaimedVolRegion<'a, Tile4, Safe, Safe>,
    // Memory for the tilemap
    screenblock_memory:
        ClaimedGridFrames<'a, TextEntry, Safe, Safe, 32, 32, 32, SCREENBLOCK_INDEX_OFFSET>,
    // Memory for the palette
    _palette_memory: ClaimedVolRegion<'a, Color, Safe, Safe>,
    palette_bank: u16,
    layer: BackgroundLayer,
}

impl From<&BackgroundSize> for u16 {
    fn from(size: &BackgroundSize) -> Self {
        match size {
            BackgroundSize::Bg32x32 => 0b00,
            BackgroundSize::Bg64x32 => 0b01,
            BackgroundSize::Bg32x64 => 0b10,
            BackgroundSize::Bg64x64 => 0b11,
        }
    }
}

impl From<BackgroundSize> for u16 {
    fn from(size: BackgroundSize) -> Self {
        (&size).into()
    }
}

impl BackgroundSize {
    pub fn required_screenblocks(&self) -> usize {
        self.screenblock_height() * self.screenblock_width()
    }

    pub fn screenblock_width(&self) -> usize {
        match self {
            Self::Bg32x32 => 1,
            Self::Bg32x64 => 1,
            Self::Bg64x32 => 2,
            Self::Bg64x64 => 2,
        }
    }

    pub fn screenblock_height(&self) -> usize {
        match self {
            Self::Bg32x32 => 1,
            Self::Bg64x32 => 1,
            Self::Bg32x64 => 2,
            Self::Bg64x64 => 2,
        }
    }
}

impl BackgroundLayer {
    pub fn get_horizontal_scroll_register(&self) -> VolAddress<u16, (), Safe> {
        match self {
            BackgroundLayer::Bg0 => BG0HOFS,
            BackgroundLayer::Bg1 => BG1HOFS,
            BackgroundLayer::Bg2 => BG2HOFS,
            BackgroundLayer::Bg3 => BG3HOFS,
        }
    }

    pub fn get_vertical_scroll_register(&self) -> VolAddress<u16, (), Safe> {
        match self {
            BackgroundLayer::Bg0 => BG0VOFS,
            BackgroundLayer::Bg1 => BG1VOFS,
            BackgroundLayer::Bg2 => BG2VOFS,
            BackgroundLayer::Bg3 => BG3VOFS,
        }
    }

    fn get_priority(&self) -> u16 {
        match self {
            BackgroundLayer::Bg0 => 3,
            BackgroundLayer::Bg1 => 2,
            BackgroundLayer::Bg2 => 1,
            BackgroundLayer::Bg3 => 0,
        }
    }

    fn get_display_control_register(&self) -> VolAddress<BackgroundControl, Safe, Safe> {
        match self {
            BackgroundLayer::Bg0 => BG0CNT,
            BackgroundLayer::Bg1 => BG1CNT,
            BackgroundLayer::Bg2 => BG2CNT,
            BackgroundLayer::Bg3 => BG3CNT,
        }
    }

    fn enable(&self) -> DisplayControl {
        match self {
            BackgroundLayer::Bg0 => DISPCNT.read().with_show_bg0(true),
            BackgroundLayer::Bg1 => DISPCNT.read().with_show_bg1(true),
            BackgroundLayer::Bg2 => DISPCNT.read().with_show_bg2(true),
            BackgroundLayer::Bg3 => DISPCNT.read().with_show_bg3(true),
        }
    }

    fn disable(&self) -> DisplayControl {
        match self {
            BackgroundLayer::Bg0 => DISPCNT.read().with_show_bg0(false),
            BackgroundLayer::Bg1 => DISPCNT.read().with_show_bg1(false),
            BackgroundLayer::Bg2 => DISPCNT.read().with_show_bg2(false),
            BackgroundLayer::Bg3 => DISPCNT.read().with_show_bg3(false),
        }
    }
}

impl Background {
    pub fn load<'a>(&'a self, gba: &'a GBA, layer: BackgroundLayer) -> LoadedBackground<'a> {
        LoadedBackground::new(self, gba, layer)
    }
}

impl<'a> LoadedBackground<'a> {
    pub fn get_layer(&self) -> BackgroundLayer {
        self.layer
    }

    fn new(background: &'a Background, gba: &'a GBA, layer: BackgroundLayer) -> Self {
        let charblock_region = gba
            .charblock_memory
            .request_memory(background.tileset.len())
            .expect("Out of charblock memory");

        let screenblocks = gba
            .screenblock_memory
            .request_memory(background.size.required_screenblocks())
            .expect("Out of screenblock memory.");

        // We only have 4BBP tiles, so request a palette bank.
        let mut palette_memory = gba
            .bg_palette_memory
            .request_aligned_memory(1, 16)
            .expect("Out of BG palette memory.");

        let pal_bank_number: u16 = palette_memory.get_start().try_into().unwrap();

        let pal_bank_number = pal_bank_number / 16;

        // Write the palette to memory.
        let bg_pal_bank_region = palette_memory.as_vol_region();

        for (i, color) in background.palette.iter().enumerate() {
            bg_pal_bank_region.get(i).unwrap().write(*color);
        }

        let mut loaded_bg = LoadedBackground {
            background,
            layer,
            _palette_memory: palette_memory,
            charblock_memory: charblock_region,
            screenblock_memory: screenblocks,
            palette_bank: pal_bank_number,
        };

        loaded_bg.write_charblocks();
        loaded_bg.write_screenblocks();
        loaded_bg.enable_background();

        loaded_bg
    }

    fn write_charblocks(&mut self) {
        // Just need to write each tile sequentially.
        let charblocks = self.charblock_memory.as_vol_region();

        for (i, tile) in self.background.tileset.iter().enumerate() {
            charblocks.get(i).unwrap().write(*tile);
        }
    }

    fn write_screenblocks(&mut self) {
        let num_screenblocks = self.background.size.required_screenblocks();
        let screenblock_width = self.background.size.screenblock_width();
        let charblock_start: u16 = self.charblock_memory.get_start().try_into().unwrap();

        for i in 0..num_screenblocks {
            // First figure out the starting row for this screenblock.
            let row_offset = i / screenblock_width * SCREENBLOCK_SIZE;

            // And the starting column within each of those rows.
            let col_offset = i % screenblock_width * SCREENBLOCK_SIZE;

            // Get the grid for this screenblock.
            let screenblock = self.screenblock_memory.get_frame(i);

            for row in 0..SCREENBLOCK_SIZE {
                let row_mem = screenblock.get_row(row).unwrap();

                let row_start = row * SCREENBLOCK_SIZE + row_offset;

                let col_start = row_start + col_offset;

                for j in 0..SCREENBLOCK_SIZE {
                    let tile = col_start + j;

                    // The tilemap is indexed from zero. Each value needs to be offset
                    // according to the starting address of the Charblock region allocated for this BG.

                    let text_entry = self.background.tilemap[tile].with_palbank(self.palette_bank);
                    let text_entry = text_entry.with_tile(text_entry.tile() + charblock_start);

                    row_mem.index(j).write(text_entry);
                }
            }
        }
    }

    fn enable_background(&mut self) {
        // Set the BGCNT register to use the screenblock and charblock set by this BG.
        let screenblock_index: u16 = self.screenblock_memory.get_start().try_into().unwrap();

        let bg_control = BackgroundControl::new()
            .with_bpp8(false)
            .with_screenblock(screenblock_index)
            .with_charblock(constants::CHARBLOCK_BASE)
            .with_priority(self.layer.get_priority());

        self.layer.get_display_control_register().write(bg_control);

        let disp_control = self.layer.enable();
        DISPCNT.write(disp_control);

        // Clear the scroll registers.
        self.layer.get_horizontal_scroll_register().write(0);
        self.layer.get_vertical_scroll_register().write(0);
    }
}

impl<'a> Drop for LoadedBackground<'a> {
    fn drop(&mut self) {
        let disp_control = self.layer.disable();
        DISPCNT.write(disp_control);
    }
}

include!(concat!(env!("OUT_DIR"), "/background_data.rs"));
