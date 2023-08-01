#![no_std]
#![no_main]

use gba::prelude::*;
use system::gba::{ClaimedVolAddress, ClaimedVolRegion, GBA};
use voladdress::Safe;

pub mod system;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn main() -> ! {
    let gba = GBA::take();

    let pal_bank_1 = gba.bg_palette_memory.request_aligned_memory(16, 1);
    let pal_bank_2 = gba.bg_palette_memory.request_aligned_memory(16, 1);

    let oam1 = gba.obj_attr_memory.request_slot();
    let oam2 = gba.obj_attr_memory.request_slot();

    // Sprite:
    // Owns tile data in OBJ_TILES
    // Owns OAM
    // Owns a palette bank

    loop {}
}
