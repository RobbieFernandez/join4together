use core::cmp::min;

use crate::graphics::sprite::{
    AnimationController, LoadedAnimation, LoadedObjectEntry, LoadedSprite,
};
use crate::system::constants::SCREEN_WIDTH;
use crate::system::{constants::BOARD_SLOTS, constants::SCREEN_HEIGHT, gba::GBA};
use cpu_turn::CpuTurn;
use player_turn::PlayerTurn;
use turn::Turn;

use self::cpu_face::CpuSprites;

pub mod cpu_face;
mod cpu_turn;
mod cursor;
mod game_board;
mod player_turn;
mod turn;

const TOKEN_DROP_TOP_SPEED: u16 = 15;
const TOKEN_DROP_SPEED_GRADIENT: u16 = 1;
const TOKEN_DROP_STARTING_SPEED: u16 = 1;

#[derive(Clone, Copy, PartialEq)]
pub enum Player {
    Red,
    Yellow,
}

#[derive(Clone)]
struct TokenDroppingState {
    player: Player,
    column: usize,
    current_y: u16,
    target_y: u16,
    row: usize,
    obj_index: usize,
    speed: u16,
}

#[derive(Clone)]
enum GameState {
    PlayerTurnState(PlayerTurn),
    CpuTurnState(CpuTurn),
    TokenDropping(TokenDroppingState),
}

pub struct GameScreen<'a> {
    gba: &'a GBA,
    red_token_animation_controller: AnimationController<'a, 4>,
    yellow_token_animation_controller: AnimationController<'a, 4>,
    _board_slot_objects: [LoadedObjectEntry<'a>; BOARD_SLOTS],
    game_state: GameState,
    game_board: game_board::GameBoard<'a>,
    cpu_face: cpu_face::CpuFace<'a>,
}

impl<'a> GameScreen<'a> {
    pub fn new(
        gba: &'a GBA,
        red_token_animation: &'a LoadedAnimation<4>,
        yellow_token_animation: &'a LoadedAnimation<4>,
        board_slot_sprite: &'a LoadedSprite<'a>,
        cpu_sprites: &'a CpuSprites<'a>,
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

        let cpu_head_sprite = cpu_sprites.get_head_sprite().sprite();

        let cpu_head_height: u16 = cpu_head_sprite.height().try_into().unwrap();
        let cpu_head_width: u16 = cpu_head_sprite.width().try_into().unwrap();

        let cpu_head_ypos = SCREEN_HEIGHT - cpu_head_height;
        let cpu_head_xpos = SCREEN_WIDTH - cpu_head_width - 5;

        let cpu_face = cpu_face::CpuFace::new(gba, cpu_head_xpos, cpu_head_ypos, cpu_sprites);

        Self {
            gba,
            red_token_animation_controller,
            yellow_token_animation_controller,
            _board_slot_objects,
            game_state,
            game_board,
            cpu_face,
        }
    }

    pub fn update(&mut self) {
        let mut state = self.get_state();

        let new_state = match state {
            GameState::PlayerTurnState(ref mut player_turn) => self.update_turn(player_turn),
            GameState::CpuTurnState(ref mut cpu_turn) => self.update_turn(cpu_turn),
            GameState::TokenDropping(ref mut token_state) => {
                self.update_token_dropping(token_state)
            }
        };

        if let Some(new_state) = new_state {
            self.game_state = new_state;
        } else {
            self.game_state = state;
        }
    }

    fn get_state(&self) -> GameState {
        self.game_state.clone()
    }

    fn update_turn<T>(&mut self, turn: &mut T) -> Option<GameState>
    where
        T: Turn,
    {
        let (player, column) = take_turn(
            turn,
            self.gba,
            &mut self.yellow_token_animation_controller,
            &mut self.red_token_animation_controller,
            &mut self.game_board,
            &mut self.cpu_face,
        );

        if let Some(column) = column {
            let row = self.game_board.get_next_free_row(column);

            match row {
                Some(row) => {
                    let obj_index = self.game_board.set_cell(player, column, row);
                    let y_pos = game_board::get_token_y_position();

                    let drop_state = TokenDroppingState {
                        player,
                        column,
                        row,
                        obj_index,
                        current_y: y_pos,
                        speed: TOKEN_DROP_STARTING_SPEED,
                        target_y: self.game_board.get_token_ypos_for_row(row),
                    };

                    Some(GameState::TokenDropping(drop_state))
                }
                None => {
                    panic!("No more rows!");
                }
            }
        } else {
            None
        }
    }

    fn update_token_dropping(&mut self, state: &mut TokenDroppingState) -> Option<GameState> {
        state.current_y = min(state.current_y + state.speed, state.target_y);
        state.speed = min(
            state.speed + TOKEN_DROP_SPEED_GRADIENT,
            TOKEN_DROP_TOP_SPEED,
        );

        self.update_token_dropping_obj(state);

        if state.current_y == state.target_y {
            // Turn is over now.
            // Check victory conditions, otherwise move to next player's turn.
            if self
                .game_board
                .is_winning_token(state.column, state.row, state.player)
            {
                // TODO - Transition to game over screen.
                if state.player == Player::Red {
                    self.cpu_face.set_emotion(cpu_face::CpuEmotion::Sad);
                }

                panic!("Game's over");
            } else {
                // TODO - Don't hardcode CPU/Player.
                match state.player {
                    Player::Red => Some(GameState::CpuTurnState(CpuTurn::new(
                        state.player.opposite(),
                    ))),
                    Player::Yellow => Some(GameState::PlayerTurnState(PlayerTurn::new(
                        state.player.opposite(),
                    ))),
                }
            }
        } else {
            None
        }
    }

    fn update_token_dropping_obj(&mut self, state: &TokenDroppingState) {
        let y_pos = state.current_y;
        let obj = self.game_board.get_token_obj_entry_mut(state.obj_index);

        match obj {
            Some(obj) => {
                let attr = obj.get_obj_attr_data();
                attr.0 = attr.0.with_y(y_pos);
                obj.commit_to_memory();
            }
            None => {}
        }
    }
}

fn take_turn<'a, T: Turn>(
    turn: &mut T,
    gba: &GBA,
    yellow_token_animation_controller: &mut AnimationController<'a, 4>,
    red_token_animation_controller: &mut AnimationController<'a, 4>,
    game_board: &mut game_board::GameBoard,
    cpu_face: &mut cpu_face::CpuFace,
) -> (Player, Option<usize>) {
    let player = turn.get_player();

    let animation_controller = match player {
        Player::Red => red_token_animation_controller,
        Player::Yellow => yellow_token_animation_controller,
    };

    let column = turn.update(gba, animation_controller, game_board, cpu_face);
    (player, column)
}

impl Player {
    pub fn opposite(&self) -> Player {
        match self {
            Player::Red => Player::Yellow,
            Player::Yellow => Player::Red,
        }
    }
}
