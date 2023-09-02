use super::cpu_face::{CpuEmotion, CpuFace};
use super::cursor::Cursor;
use super::game_board;
use super::turn::Turn;
use super::Player;
use crate::graphics::sprite::AnimationController;
use crate::system::gba::{GbaKey, GBA};

#[derive(Clone)]
pub struct PlayerTurn {
    player: Player,
    cursor: Cursor,
}

impl PlayerTurn {
    pub fn new(player: Player) -> Self {
        let cursor = Cursor::new();
        Self { player, cursor }
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
        if self.cursor.is_moving() {
            self.cursor.update_movement();
        } else if gba.key_was_pressed(GbaKey::LEFT) {
            self.cursor.move_left();
        } else if gba.key_was_pressed(GbaKey::RIGHT) {
            self.cursor.move_right();
        } else if gba.key_was_pressed(GbaKey::A) {
            let col = self.cursor.get_column();
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

        self.cursor.draw(anim_controller);

        None
    }

    fn get_player(&self) -> Player {
        self.player
    }
}
