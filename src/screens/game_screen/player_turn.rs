use super::game_board;
use super::turn::{Turn, TurnOutcome};
use super::Player;
use crate::graphics::sprite::{AnimationController, BOARD_SLOT_SPRITE, RED_TOKEN_FRAME_0_SPRITE};
use crate::system::{
    constants::BOARD_COLUMNS,
    gba::{GbaKey, GBA},
};
use gba::video::obj::ObjDisplayStyle;

pub struct PlayerTurn {
    player: Player,
    cursor_position: u8,
}

impl PlayerTurn {
    pub fn new(player: Player) -> Self {
        Self {
            player,
            cursor_position: 0,
        }
    }

    pub fn get_player(&self) -> Player {
        self.player
    }

    fn move_left(&mut self) {
        if self.cursor_position == 0 {
            self.cursor_position = BOARD_COLUMNS;
        }

        self.cursor_position -= 1;
    }

    fn move_right(&mut self) {
        self.cursor_position = (self.cursor_position + 1) % BOARD_COLUMNS;
    }
}

impl Turn for PlayerTurn {
    fn update<'a>(
        &mut self,
        gba: &GBA,
        yellow_token_animation_controller: &mut AnimationController<'a, 4>,
        red_token_animation_controller: &mut AnimationController<'a, 4>,
        game_board: &mut game_board::GameBoard,
    ) -> TurnOutcome {
        let anim_controller = match self.player {
            Player::Yellow => yellow_token_animation_controller,
            Player::Red => red_token_animation_controller,
        };

        if gba.key_was_pressed(GbaKey::LEFT) {
            self.move_left();
        } else if gba.key_was_pressed(GbaKey::RIGHT) {
            self.move_right();
        } else if gba.key_was_pressed(GbaKey::A) {
            let col_number: usize = self.cursor_position.try_into().unwrap();
            let token_position = game_board.drop_token(col_number, self.player);

            if let Some((col, row)) = token_position {
                return if game_board.is_winning_token(col, row, self.player) {
                    TurnOutcome::Victory
                } else {
                    TurnOutcome::NextTurn
                };
            }
        }

        let oa = anim_controller.get_obj_attr_entry().get_obj_attr_data();

        let (start_x, start_y) = game_board::board_top_left_corner();
        let token_height: u16 = RED_TOKEN_FRAME_0_SPRITE.height().try_into().unwrap();
        let token_width: u16 = RED_TOKEN_FRAME_0_SPRITE.width().try_into().unwrap();
        let board_slot_width: u16 = BOARD_SLOT_SPRITE.width().try_into().unwrap();
        let padding = (board_slot_width - token_width) / 2;

        let xpos = start_x + (self.cursor_position as u16) * board_slot_width + padding;

        let ypos = start_y / 2 - token_height / 2;

        oa.1 = oa.1.with_x(xpos);
        oa.0 = oa.0.with_y(ypos).with_style(ObjDisplayStyle::Normal);

        anim_controller.tick();

        TurnOutcome::Continue
    }

    fn get_player(&self) -> Player {
        self.player
    }
}
