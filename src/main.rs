#![no_std]
#![no_main]

use gba::video::Color;

use graphics::sprite::{LoadedObjectEntry, BOARD_SLOT_SPRITE, PALETTE};
use system::gba::GBA;

pub mod graphics;
pub mod system;

const BOARD_COLUMNS: u16 = 7;
const BOARD_ROWS: u16 = 6;
const BOARD_SLOTS: usize = (BOARD_COLUMNS * BOARD_ROWS) as usize;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn main() -> ! {
    let gba = GBA::take();

    let mut palette_mem = gba
        .obj_palette_memory
        .request_memory(PALETTE.len())
        .expect("Object palette cannot fit in memory.");

    let palette_mem_region = palette_mem.as_vol_region();

    for i in 0..PALETTE.len() {
        let mut col = Color::new();
        col.0 = PALETTE[i];
        palette_mem_region.index(i).write(Color::from(col));
    }

    let tile_sprite = BOARD_SLOT_SPRITE.load(&gba);

    let board_slot_width: u16 = BOARD_SLOT_SPRITE.width().try_into().unwrap();
    let board_slot_height: u16 = BOARD_SLOT_SPRITE.height().try_into().unwrap();

    let board_width_pixels: u16 = board_slot_width * BOARD_COLUMNS;
    let board_height_pixels: u16 = board_slot_height * BOARD_ROWS;

    let start_y: u16 = 160 - board_height_pixels;
    let start_x: u16 = (240 - board_width_pixels) / 2;

    let _tile_slot_objs: [LoadedObjectEntry; BOARD_SLOTS] = core::array::from_fn(|i| {
        let mut obj_entry = tile_sprite.create_obj_attr_entry(&gba);

        let i: u16 = i.try_into().unwrap();
        let col: u16 = i % BOARD_COLUMNS;
        let row: u16 = i / BOARD_COLUMNS;

        let obj_attrs = obj_entry.get_obj_attr_data();
        obj_attrs.0 = obj_attrs.0.with_y(start_y + row * board_slot_height);
        obj_attrs.1 = obj_attrs.1.with_x(start_x + col * board_slot_width);

        obj_entry.commit_to_memory();

        obj_entry
    });

    loop {
        gba::bios::VBlankIntrWait();
    }
}
