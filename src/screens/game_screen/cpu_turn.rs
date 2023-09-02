use core::cmp::Ordering;

use crate::system::constants::BOARD_COLUMNS;
use crate::system::gba::GBA;

use super::cpu_face::{CpuEmotion, CpuFace};
use super::cursor::Cursor;
use super::game_board;
use super::turn::Turn;
use super::Player;
use crate::graphics::sprite::AnimationController;

const NUM_COLUMNS: usize = BOARD_COLUMNS as usize;
const MOVEMENT_DELAY: u32 = 6;

#[derive(Clone)]
struct DecidingState {
    col_scores: [Option<i32>; NUM_COLUMNS],
    scored_columns: usize,
}

#[derive(Clone)]
struct MovingState {
    target_column: usize,
    move_delay_timer: u32,
}

#[derive(Clone)]
enum CpuState {
    Deciding(DecidingState),
    Moving(MovingState),
}

#[derive(Clone)]
pub struct CpuTurn {
    player: Player,
    state: CpuState,
    cursor: Cursor,
}

impl CpuTurn {
    pub fn new(player: Player) -> Self {
        let deciding_state = DecidingState {
            col_scores: [None; NUM_COLUMNS],
            scored_columns: 0,
        };

        Self {
            player,
            state: CpuState::Deciding(deciding_state),
            cursor: Cursor::new(),
        }
    }
}

impl Turn for CpuTurn {
    fn update(
        &mut self,
        _gba: &GBA,
        animation_controller: &mut AnimationController<4>,
        game_board: &mut game_board::GameBoard,
        cpu_face: &mut CpuFace,
    ) -> Option<usize> {
        let player = self.get_player();

        match self.state {
            CpuState::Deciding(ref mut deciding) => {
                let best_column = deciding.get_best_column();

                if let Some(best_column) = best_column {
                    let moving_state = MovingState::new(best_column);
                    self.state = CpuState::Moving(moving_state);
                } else {
                    deciding.score_next_column(player, game_board, cpu_face);
                }
            }
            CpuState::Moving(ref mut moving) => {
                let finished_moving = moving.update(&mut self.cursor);

                if finished_moving {
                    let column = self.cursor.get_column();
                    let row = game_board.get_next_free_row(column);

                    if let Some(row) = row {
                        if !game_board.is_winning_token(column, row, self.player) {
                            cpu_face.set_emotion(CpuEmotion::Neutral);
                        }

                        animation_controller.set_hidden();
                        animation_controller.get_obj_attr_entry().commit_to_memory();

                        return Some(column);
                    } else {
                        panic!("CPU chose invalid best move.")
                    }
                }
            }
        };

        self.cursor.draw(animation_controller);

        None
    }

    fn get_player(&self) -> Player {
        self.player
    }
}

impl DecidingState {
    pub fn score_next_column(
        &mut self,
        player: Player,
        game_board: &mut game_board::GameBoard,
        cpu_face: &mut CpuFace,
    ) {
        let score = self.score_column(player, game_board, self.scored_columns, cpu_face);

        self.col_scores[self.scored_columns] = Some(score);
        self.scored_columns += 1;
    }

    pub fn get_best_column(&self) -> Option<usize> {
        if self.scored_columns == self.col_scores.len() {
            let best = (0..NUM_COLUMNS).max_by_key(|i| self.col_scores[*i].unwrap());
            Some(best.unwrap())
        } else {
            None
        }
    }

    fn score_column(
        &self,
        player: Player,
        game_board: &mut game_board::GameBoard,
        column_number: usize,
        cpu_face: &mut CpuFace,
    ) -> i32 {
        let row = game_board.get_next_free_row(column_number);

        if row.is_none() {
            return i32::MIN;
        }

        let row = row.unwrap();

        let opponent = player.opposite();

        // See what the board will look like after this move is made.
        let candidate_board = game_board.get_board_after_move(column_number, player);

        // First priority is to choose a winning move.
        if game_board.player_can_win(column_number, player) {
            cpu_face.set_emotion(CpuEmotion::Happy);
            return i32::MAX;
        };

        // Next priority is to block opponent's winning move.
        if game_board.player_can_win(column_number, opponent) {
            cpu_face.set_emotion(CpuEmotion::Surprised);
            return i32::MAX - 1;
        }

        // Don't make a move that sets up a winning move for the opponent.
        if self.player_has_winning_move(&candidate_board, opponent) {
            // + 1 makes sure this is chosen over a move that is not allowed.
            return i32::MIN + 1;
        }

        // Then see if you can set up a winning move.
        if self.player_has_winning_move(&candidate_board, player) {
            return i32::MAX - 2;
        }

        // Otherwise fall back to heuristic:
        //  Go through each neighbour. +1 if my token, 0 for unoccupied, +2 if opponent's token.
        game_board::DIRECTIONS.iter().fold(0, |score, direction| {
            let neighbour = game_board.get_neighbour(column_number, row, direction);

            score
                + match neighbour {
                    None => 0,
                    Some(color) => {
                        if color == player {
                            1
                        } else {
                            2
                        }
                    }
                }
        })
    }

    fn player_has_winning_move(&self, game_board: &game_board::GameBoard, player: Player) -> bool {
        (0..NUM_COLUMNS).any(|column| game_board.player_can_win(column, player))
    }
}

impl MovingState {
    pub fn new(target_column: usize) -> Self {
        Self {
            target_column,
            move_delay_timer: MOVEMENT_DELAY,
        }
    }

    pub fn update(&mut self, cursor: &mut Cursor) -> bool {
        cursor.update_movement();

        if !cursor.is_moving() && self.update_timer() {
            match cursor.get_column().cmp(&self.target_column) {
                Ordering::Greater => {
                    cursor.move_left();
                }
                Ordering::Less => {
                    cursor.move_right();
                }
                Ordering::Equal => {
                    return true;
                }
            }
        }

        false
    }

    fn update_timer(&mut self) -> bool {
        self.move_delay_timer -= 1;

        if self.move_delay_timer == 0 {
            self.move_delay_timer = MOVEMENT_DELAY;
            true
        } else {
            false
        }
    }
}
