#![no_std]
#![no_main]

use gba::video::Color;
use graphics::sprite::{BOARD_SLOT_SPRITE, PALETTE, RED_TOKEN_ANIMATION, YELLOW_TOKEN_ANIMATION};
use screens::game_screen::GameScreen;
use system::gba::GBA;

pub mod graphics;
pub mod screens;
pub mod system;

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

    for (i, pal_color) in PALETTE.iter().enumerate() {
        let mut col = Color::new();
        col.0 = *pal_color;
        palette_mem_region.index(i).write(col);
    }

    let yellow_token_animation = YELLOW_TOKEN_ANIMATION.load(&gba);
    let red_token_animation = RED_TOKEN_ANIMATION.load(&gba);
    let board_slot_sprite = BOARD_SLOT_SPRITE.load(&gba);

    let mut game_screen = GameScreen::new(
        &gba,
        &red_token_animation,
        &yellow_token_animation,
        &board_slot_sprite,
    );

    loop {
        gba::bios::VBlankIntrWait();

        game_screen.update();

        // match game_state {
        //     GameState::Turn(ref state) => {
        //         let player_anim_controller = match state.player {
        //             Player::Red => &mut red_token_animation_controller,
        //             Player::Yellow => &mut yellow_token_animation_controller,
        //         };

        //         // Handle key presses
        //         if gba.key_was_pressed(system::gba::GbaKey::LEFT) {
        //             // let red_obj = red_token_animation_controller
        //             //     .get_obj_attr_entry()
        //             //     .get_obj_attr_data();

        //             // red_obj.1 = red_obj.1.with_x(red_obj.1.x() - board_slot_width)
        //         }

        //         if gba.key_was_pressed(system::gba::GbaKey::RIGHT) {
        //             // let red_obj = red_token_animation_controller
        //             //     .get_obj_attr_entry()
        //             //     .get_obj_attr_data();

        //             // red_obj.1 = red_obj.1.with_x(red_obj.1.x() + board_slot_width)
        //         }

        //         let mut oa = player_anim_controller
        //             .get_obj_attr_entry()
        //             .get_obj_attr_data();

        //         oa.0 = oa.0.with_style(ObjDisplayStyle::Normal);

        //         player_anim_controller.tick();
        //     }
        //     _ => {}
        // }
    }
}
