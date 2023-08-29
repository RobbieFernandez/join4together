use crate::system::constants::BOARD_COLUMNS;
use crate::system::gba::GBA;

use super::game_board;
use super::turn::{Turn, TurnOutcome};
use super::Player;
use crate::graphics::sprite::AnimationController;

pub struct CpuTurn {
    player: Player,
}

impl CpuTurn {
    pub fn new(player: Player) -> Self {
        Self { player }
    }

    fn choose_column(&self, game_board: &mut game_board::GameBoard) -> usize {
        let num_columns: usize = BOARD_COLUMNS.try_into().unwrap();
        (0..num_columns)
            .max_by_key(|c| self.score_column(game_board, *c))
            .unwrap_or_default()
    }

    fn score_column(&self, game_board: &mut game_board::GameBoard, column_number: usize) -> i32 {
        let row = game_board.get_next_free_row(column_number);

        if row.is_none() {
            return i32::MIN;
        }

        let row = row.unwrap();

        let opponent = self.player.opposite();

        // See what the board will look like after this move is made.
        let candidate_board = game_board.get_board_after_move(column_number, self.player);

        // First priority is to choose a winning move.
        if game_board.player_can_win(column_number, self.player) {
            return i32::MAX;
        };

        // Next priority is to block opponent's winning move.
        if game_board.player_can_win(column_number, opponent) {
            return i32::MAX - 1;
        }

        // Don't make a move that sets up a winning move for the opponent.
        if self.player_has_winning_move(&candidate_board, opponent) {
            // + 1 makes sure this is chosen over a move that is not allowed.
            return i32::MIN + 1;
        }

        // Then see if you can set up a winning move.
        if self.player_has_winning_move(&candidate_board, self.player) {
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
                        if color == self.player {
                            1
                        } else {
                            2
                        }
                    }
                }
        })
    }

    fn player_has_winning_move(&self, game_board: &game_board::GameBoard, player: Player) -> bool {
        let num_columns: usize = BOARD_COLUMNS.try_into().unwrap();
        (0..num_columns).any(|column| game_board.player_can_win(column, player))
    }
}

impl Turn for CpuTurn {
    fn update<'a>(
        &mut self,
        gba: &GBA,
        yellow_token_animation_controller: &mut AnimationController<'a, 4>,
        red_token_animation_controller: &mut AnimationController<'a, 4>,
        game_board: &mut game_board::GameBoard,
    ) -> TurnOutcome {
        let column = self.choose_column(game_board);

        let token_position = game_board.drop_token(column, self.player);

        if let Some((col, row)) = token_position {
            if game_board.is_winning_token(col, row, self.player) {
                TurnOutcome::Victory
            } else {
                TurnOutcome::NextTurn
            }
        } else {
            TurnOutcome::Continue
        }
    }

    fn get_player(&self) -> Player {
        self.player
    }
}
