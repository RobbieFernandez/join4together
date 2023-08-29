use crate::graphics::sprite::{
    AnimationController, LoadedAnimation, LoadedObjectEntry, LoadedSprite,
};
use crate::system::{constants::BOARD_SLOTS, gba::GBA};
use cpu_turn::CpuTurn;
use gba::prelude::ObjDisplayStyle;
use player_turn::PlayerTurn;
use turn::{Turn, TurnOutcome};

mod cpu_turn;
mod game_board;
mod player_turn;
mod turn;

#[derive(Clone, Copy, PartialEq)]
pub enum Player {
    Red,
    Yellow,
}

pub enum GameState {
    PlayerTurnState(PlayerTurn),
    CpuTurnState(CpuTurn),
    // TokenDropping(Player, usize),
}

pub struct GameScreen<'a> {
    gba: &'a GBA,
    red_token_animation_controller: AnimationController<'a, 4>,
    yellow_token_animation_controller: AnimationController<'a, 4>,
    _board_slot_objects: [LoadedObjectEntry<'a>; BOARD_SLOTS],
    game_state: GameState,
    game_board: game_board::GameBoard<'a>,
}

impl<'a> GameScreen<'a> {
    pub fn new(
        gba: &'a GBA,
        red_token_animation: &'a LoadedAnimation<4>,
        yellow_token_animation: &'a LoadedAnimation<4>,
        board_slot_sprite: &'a LoadedSprite<'a>,
    ) -> Self {
        let red_token_animation_controller = red_token_animation.create_controller(gba);
        let yellow_token_animation_controller = yellow_token_animation.create_controller(gba);

        // Create an Object entry for each slot that makes up the board.
        // We need to keep ownership of these in order to keep them in OBJRAM, so store them in an array.
        let _board_slot_objects = game_board::create_board_object_entries(board_slot_sprite, gba);

        // For now hardcode player is red, CPU is yellow and player goes first.
        let game_state = GameState::PlayerTurnState(PlayerTurn::new(Player::Red));

        let game_board = game_board::GameBoard::new(
            gba,
            red_token_animation.get_frame(0),
            yellow_token_animation.get_frame(0),
        );

        Self {
            gba,
            red_token_animation_controller,
            yellow_token_animation_controller,
            _board_slot_objects,
            game_state,
            game_board,
        }
    }

    pub fn update(&mut self) {
        let (current_player, turn_outcome) = match self.game_state {
            GameState::PlayerTurnState(ref mut player_turn) => take_turn(
                player_turn,
                self.gba,
                &mut self.yellow_token_animation_controller,
                &mut self.red_token_animation_controller,
                &mut self.game_board,
            ),
            GameState::CpuTurnState(ref mut cpu_turn) => take_turn(
                cpu_turn,
                self.gba,
                &mut self.yellow_token_animation_controller,
                &mut self.red_token_animation_controller,
                &mut self.game_board,
            ),
        };

        match turn_outcome {
            TurnOutcome::NextTurn => {
                let anim_controller = match current_player {
                    Player::Red => &mut self.red_token_animation_controller,
                    Player::Yellow => &mut self.yellow_token_animation_controller,
                };

                let oa_entry = anim_controller.get_obj_attr_entry();
                let oa_attr = oa_entry.get_obj_attr_data();

                oa_attr.0 = oa_attr.0.with_style(ObjDisplayStyle::NotDisplayed);
                oa_entry.commit_to_memory();

                let next_player = current_player.opposite();

                self.game_state = match self.game_state {
                    GameState::PlayerTurnState(_) => {
                        let turn = CpuTurn::new(next_player);
                        GameState::CpuTurnState(turn)
                    }
                    GameState::CpuTurnState(_) => {
                        let turn = PlayerTurn::new(next_player);
                        GameState::PlayerTurnState(turn)
                    }
                }
            }
            // TODO - Handle rest.
            TurnOutcome::Continue => {}
            TurnOutcome::Victory => {
                panic!("Done")
            }
        }
    }
}

fn take_turn<'a, T: Turn>(
    turn: &mut T,
    gba: &GBA,
    yellow_token_animation_controller: &mut AnimationController<'a, 4>,
    red_token_animation_controller: &mut AnimationController<'a, 4>,
    game_board: &mut game_board::GameBoard,
) -> (Player, TurnOutcome) {
    let outcome = turn.update(
        gba,
        yellow_token_animation_controller,
        red_token_animation_controller,
        game_board,
    );
    let player = turn.get_player();
    (player, outcome)
}

impl Player {
    pub fn opposite(&self) -> Player {
        match self {
            Player::Red => Player::Yellow,
            Player::Yellow => Player::Red,
        }
    }
}
