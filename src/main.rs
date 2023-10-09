#![no_std]
#![no_main]

use gba::prelude::{MgbaBufferedLogger, MgbaMessageLevel};
use graphics::sprite::OBJ_PALETTE;

use screens::ScreenState;
use system::gba::GBA;

pub mod audio;
pub mod graphics;
pub mod math;
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

#[allow(clippy::large_enum_variant)]
#[no_mangle]
extern "C" fn main() -> ! {
    let gba = GBA::take();

    let mut palette_mem = gba
        .obj_palette_memory
        .request_memory(OBJ_PALETTE.len())
        .expect("Object palette cannot fit in memory.");

    let palette_mem_region = palette_mem.as_vol_region();

    for (i, color) in OBJ_PALETTE.iter().enumerate() {
        palette_mem_region.index(i).write(*color);
    }

    let mut screen_state = ScreenState::TitleScreen;

    let mut music_player = audio::music::MusicPlayer::take();

    // Top-level game loop just runs the currently active screen until it transitions.
    loop {
        screen_state = screen_state.exec_screen(&gba, &mut music_player);
    }
}
