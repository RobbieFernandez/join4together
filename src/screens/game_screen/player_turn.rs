use super::cpu_face::{CpuEmotion, CpuFace};
use super::game_board;
use super::turn::Turn;
use super::Player;
use crate::graphics::sprite::AnimationController;
use crate::system::{
    constants::BOARD_COLUMNS,
    gba::{GbaKey, GBA},
};

#[derive(Clone)]
pub struct PlayerTurn {
    player: Player,
    pub cursor_position: u8,
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
    ) -> Option<usize> {
        use core::fmt::Write;
        use gba::prelude::{MgbaBufferedLogger, MgbaMessageLevel};
        let log_level = MgbaMessageLevel::Debug;
        if let Ok(mut logger) = MgbaBufferedLogger::try_new(log_level) {
            writeln!(logger, "Cursor (turn): {}", self.cursor_position).ok();
        }

        if gba.key_was_pressed(GbaKey::LEFT) {
            self.move_left();
        } else if gba.key_was_pressed(GbaKey::RIGHT) {
            self.move_right();
        } else if gba.key_was_pressed(GbaKey::A) {
            let col: usize = self.cursor_position.try_into().unwrap();
            let row = game_board.get_next_free_row(col);
            anim_controller.set_hidden();
            anim_controller.get_obj_attr_entry().commit_to_memory();

            if let Some(row) = row {
                // If the player blocks the CPU, then he should be angry.
                if game_board.is_winning_token(col, row, self.player.opposite()) {
                    cpu_face.set_emotion(CpuEmotion::Mad);
                }

                return Some(col);
            }
        }

        self.draw_cursor(self.cursor_position as u16, anim_controller);

        None
    }

    fn get_player(&self) -> Player {
        self.player
    }
}
