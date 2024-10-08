#![no_std]
#![no_main]

use audio::mixer;
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
    use gba::prelude::{MgbaBufferedLogger, MgbaMessageLevel};
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
    palette_mem_region.write_from_slice(&OBJ_PALETTE);

    let bgm = audio::mixer::AudioSource::new(
        audio::assets::BACKGROUND_MUSIC,
        audio::mixer::AudioVolume::new(45),
        true,
    );

    mixer::set_channel_1(bgm);

    // Top-level game loop just runs the currently active screen until it transitions.
    let mut screen_state = ScreenState::TitleScreen;
    loop {
        screen_state = screen_state.exec_screen(&gba);
    }
}
