#![no_std]
#![no_main]

use gba::video::DisplayControl;
use system::gba::GBA;

pub mod system;
pub mod graphics;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn main() -> ! {
    let mut gba = GBA::take();

    let _pal_bank_1 = gba.bg_palette_memory.request_aligned_memory(16, 1);
    let _pal_bank_2 = gba.bg_palette_memory.request_aligned_memory(16, 1);

    let _oam1 = gba.obj_attr_memory.request_slot();
    let _oam2 = gba.obj_attr_memory.request_slot();

    // Sprite:
    // Owns tile data in OBJ_TILES
    // Store number of tiles.
    // Owns OAM slot
    // Owns a palette bank
    // Owns ObjectAttr struct
    //
    // Commit method:
    //      Flushes ObjectAttr to OAM
    // Load method:
    //      Claims relevant memory and takes ownership
    //      Writes palette to PALRAM

    loop {}
}
