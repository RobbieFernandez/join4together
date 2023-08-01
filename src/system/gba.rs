use super::memory::block::MemoryBlockManager;
use super::memory::series::MemorySeriesManager;
use gba::prelude::*;
use voladdress::Safe;

pub use super::memory::block::ClaimedVolRegion;
pub use super::memory::series::ClaimedVolAddress;

type PaletteMemory = MemoryBlockManager<Color, Safe, Safe, 256>;
type ObjAttrMemory = MemorySeriesManager<ObjAttr, Safe, Safe, 128, 8>;
type ObjTileMemory = MemoryBlockManager<Tile4, Safe, Safe, 1024>;

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

        GBA {
            obj_palette_memory: PaletteMemory::new(OBJ_PALETTE),
            bg_palette_memory: PaletteMemory::new(BG_PALETTE),
            obj_attr_memory: ObjAttrMemory::new(OBJ_ATTR_ALL),
            obj_tile_memory: ObjTileMemory::new(OBJ_TILES),
        }
    }
}
