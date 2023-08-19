#![no_std]
#![no_main]

use gba::video::Color;

use system::gba::GBA;
use graphics::sprite::{PALETTE, HOODMAN_SPRITE};

pub mod graphics;
pub mod system;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn main() -> ! {
    let gba = GBA::take();

    let mut palette_mem = gba.obj_palette_memory.request_memory(
        PALETTE.len()
    );
    let palette_mem_region = palette_mem.as_vol_region();

    for i in 0..PALETTE.len() {
        let mut col = Color::new();
        col.0 = PALETTE[i];
        palette_mem_region.index(i).write(Color::from(col));
    }

    let loaded_sprite = HOODMAN_SPRITE.load(&gba);
    let obj_attr = loaded_sprite.create_obj_attr_entry();
    
    // Now construct OAM entry for this sprite. 
    let mut oa_slot = gba.obj_attr_memory.request_slot();
    let oa_addr = oa_slot.as_vol_address();

    oa_addr.write(obj_attr);

    loop {}
}
