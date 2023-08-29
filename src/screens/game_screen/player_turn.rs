use super::cpu_face::{CpuEmotion, CpuFace};
use super::game_board;
use super::turn::{Turn, TurnOutcome};
use super::Player;
use crate::graphics::sprite::AnimationController;
use crate::system::{
    constants::BOARD_COLUMNS,
    gba::{GbaKey, GBA},
};

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
    fn update(
        &mut self,
        gba: &GBA,
        anim_controller: &mut AnimationController<4>,
        game_board: &mut game_board::GameBoard,
        cpu_face: &mut CpuFace,
    ) -> TurnOutcome {
        if gba.key_was_pressed(GbaKey::LEFT) {
            self.move_left();
        } else if gba.key_was_pressed(GbaKey::RIGHT) {
            self.move_right();
        } else if gba.key_was_pressed(GbaKey::A) {
            let col_number: usize = self.cursor_position.try_into().unwrap();
            let token_position = game_board.drop_token(col_number, self.player);

            if let Some((col, row)) = token_position {
                // If the player blocks the CPU, then he should be angry.
                if game_board.is_winning_token(col, row, self.player.opposite()) {
                    cpu_face.set_emotion(CpuEmotion::Mad);
                }

                return if game_board.is_winning_token(col, row, self.player) {
                    TurnOutcome::Victory
                } else {
                    TurnOutcome::NextTurn
                };
            }
        }

        self.draw_cursor(self.cursor_position as u16, anim_controller);

        TurnOutcome::Continue
    }

    fn get_player(&self) -> Player {
        self.player
    }
}
