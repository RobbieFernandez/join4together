use super::memory::block::MemoryBlockManager;
use super::memory::series::MemorySeriesManager;
use gba::prelude::*;
use voladdress::Safe;

pub use super::memory::block::ClaimedVolRegion;
pub use super::memory::series::ClaimedVolAddress;

pub type PaletteMemory = MemoryBlockManager<Color, Safe, Safe, 256>;
pub type ObjAttrMemory = MemorySeriesManager<ObjAttr, Safe, Safe, 128, 8>;
pub type ObjTileMemory = MemoryBlockManager<Tile4, Safe, Safe, 1024>;

static GBA_TAKEN: GbaCell<bool> = GbaCell::new(false);

pub struct GBA {
    pub obj_palette_memory: PaletteMemory,
    pub bg_palette_memory: PaletteMemory,
    pub obj_attr_memory: ObjAttrMemory,
    pub obj_tile_memory: ObjTileMemory,
}

impl GBA {
    pub fn take() -> Self {
        if GBA_TAKEN.read() {
            panic!("GBA struct can only be taken once.");
        }

        GBA_TAKEN.write(true);

        let mut gba = GBA {
            bg_palette_memory: PaletteMemory::new(BG_PALETTE),
            obj_palette_memory: PaletteMemory::new(OBJ_PALETTE),
            obj_attr_memory: ObjAttrMemory::new(OBJ_ATTR_ALL),
            obj_tile_memory: ObjTileMemory::new(OBJ_TILES),
        };
        gba.init();
        gba
    }

    pub fn set_display_mode(&mut self, display_mode: DisplayControl) {
        DISPCNT.write(display_mode);
    }

    pub fn init(&mut self) {
        self.hide_all_objects();
        self.set_display_mode(
            DisplayControl::new()
                .with_video_mode(VideoMode::_0)
                .with_obj_vram_1d(true)
                .with_show_obj(true),
        );

        DISPSTAT.write(DisplayStatus::new().with_irq_vblank(true));
        IE.write(IrqBits::VBLANK);
        IME.write(true);
    }

    fn hide_all_objects(&mut self) {
        let hidden_obj = ObjAttr0::new().with_style(ObjDisplayStyle::NotDisplayed);

        for addr in OBJ_ATTR0.iter() {
            addr.write(hidden_obj);
        }
    }
}
