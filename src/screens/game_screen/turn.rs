use gba::prelude::ObjDisplayStyle;

use crate::{
    graphics::sprite::{AnimationController, BOARD_SLOT_SPRITE, RED_TOKEN_FRAME_0_SPRITE},
    system::gba::GBA,
};

use super::{cpu_face::CpuFace, game_board, Player};

pub enum TurnOutcome {
    Victory,
    NextTurn,
    Continue,
}

pub trait Turn {
    fn update(
        &mut self,
        gba: &GBA,
        animation_controller: &mut AnimationController<4>,
        game_board: &mut game_board::GameBoard,
        cpu_face: &mut CpuFace,
    ) -> TurnOutcome;

    fn get_player(&self) -> Player;

    fn draw_cursor(&self, column_number: u16, animation_controller: &mut AnimationController<4>) {
        let oa = animation_controller
            .get_obj_attr_entry()
            .get_obj_attr_data();

        let (start_x, start_y) = game_board::board_top_left_corner();
        let token_height: u16 = RED_TOKEN_FRAME_0_SPRITE.height().try_into().unwrap();
        let token_width: u16 = RED_TOKEN_FRAME_0_SPRITE.width().try_into().unwrap();
        let board_slot_width: u16 = BOARD_SLOT_SPRITE.width().try_into().unwrap();
        let padding = (board_slot_width - token_width) / 2;

        let xpos = start_x + column_number * board_slot_width + padding;

        let ypos = start_y / 2 - token_height / 2;

        oa.1 = oa.1.with_x(xpos);
        oa.0 = oa.0.with_y(ypos).with_style(ObjDisplayStyle::Normal);

        animation_controller.tick();
    }
}
