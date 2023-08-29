#![no_std]
#![no_main]

use gba::{
    prelude::{MgbaBufferedLogger, MgbaMessageLevel},
    video::Color,
};
use graphics::sprite::{BOARD_SLOT_SPRITE, PALETTE, RED_TOKEN_ANIMATION, YELLOW_TOKEN_ANIMATION};
use screens::game_screen::GameScreen;
use system::gba::GBA;

pub mod graphics;
pub mod screens;
pub mod system;

#[panic_handler]
fn panic_handler(i: &core::panic::PanicInfo) -> ! {
    use core::fmt::Write;
    let log_level = MgbaMessageLevel::Error;
    if let Ok(mut logger) = MgbaBufferedLogger::try_new(log_level) {
        writeln!(logger, "Panic, {}", i).ok();
    }

    loop {}
}

enum Screen<'a> {
    Game(GameScreen<'a>),
}

#[no_mangle]
extern "C" fn main() -> ! {
    let gba = GBA::take();

    let mut palette_mem = gba
        .obj_palette_memory
        .request_memory(PALETTE.len())
        .expect("Object palette cannot fit in memory.");

    let palette_mem_region = palette_mem.as_vol_region();

    for (i, pal_color) in PALETTE.iter().enumerate() {
        let mut col = Color::new();
        col.0 = *pal_color;
        palette_mem_region.index(i).write(col);
    }

    let yellow_token_animation = YELLOW_TOKEN_ANIMATION.load(&gba);
    let red_token_animation = RED_TOKEN_ANIMATION.load(&gba);
    let board_slot_sprite = BOARD_SLOT_SPRITE.load(&gba);

    let mut screen = Screen::Game(GameScreen::new(
        &gba,
        &red_token_animation,
        &yellow_token_animation,
        &board_slot_sprite,
    ));

    loop {
        gba::bios::VBlankIntrWait();

        match screen {
            Screen::Game(ref mut g) => g.update(),
        };
    }
}
