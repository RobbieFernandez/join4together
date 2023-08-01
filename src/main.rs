#![no_std]
#![no_main]

use gba::video::Color;
use system::gba::GBA;

pub mod system;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn main() -> ! {
    let mut gba = GBA::take();

    let bg_palette_manager = gba.bg_palette_memory();
    let white = Color(0b0_11111_11111_11111);

    let mut bg_pal_slots = bg_palette_manager.request_memory(2);

    bg_pal_slots
        .as_vol_region()
        .write_from_slice(&[white, white]);

    loop {}
}
