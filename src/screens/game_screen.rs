use core::cmp::min;

use self::cpu_face::CpuFace;

use super::{Screen, ScreenState};
use crate::graphics::background::{LoadedBackground, BOARD_BACKGROUND};
use crate::graphics::sprite::{
    AnimationController, LoadedAnimation, LoadedObjectEntry, LoadedSprite,
};
use crate::system::{constants::BOARD_SLOTS, gba::GBA};
use cpu_turn::CpuTurn;
use player_turn::PlayerTurn;

pub mod cpu_face;
mod cpu_turn;
mod cursor;
mod game_board;
mod player_turn;

const TOKEN_DROP_TOP_SPEED: i16 = 15;
const TOKEN_DROP_SPEED_GRADIENT: i16 = 1;
const TOKEN_DROP_STARTING_SPEED: i16 = 1;

const TOKEN_BOUNCE_SPEED_DECAY: i16 = 2;

pub enum Agent<'a> {
    Human(TokenColor, PlayerTurn),
    Cpu(TokenColor, CpuFace<'a>, CpuTurn),
}

#[derive(Clone, Copy, PartialEq)]
pub enum TokenColor {
    Red,
    Yellow,
}

#[derive(Clone)]
struct TokenDroppingState {
    token_color: TokenColor,
    column: usize,
    current_y: u16,
    target_y: u16,
    row: usize,
    obj_index: usize,
    speed: i16,
    num_bounces: i16,
}

#[derive(Clone)]
enum GameState {
    TurnState(TokenColor),
    TokenDropping(TokenDroppingState),
}

pub struct GameScreen<'a> {
    gba: &'a GBA,
    red_token_animation_controller: AnimationController<'a, 4>,
    yellow_token_animation_controller: AnimationController<'a, 4>,
    _board_slot_objects: [LoadedObjectEntry<'a>; BOARD_SLOTS],
    game_state: GameState,
    game_board: game_board::GameBoard<'a>,
    _background: LoadedBackground<'a>,
    red_agent: Agent<'a>,
    yellow_agent: Agent<'a>,
}

// The agent enum just proxies the update call to the appropriate Turn struct.
impl<'a> Agent<'a> {
    pub fn get_color(&self) -> TokenColor {
        match self {
            Self::Cpu(token_color, _, _) => token_color.clone(),
            Self::Human(token_color, _) => token_color.clone(),
        }
    }

    pub fn update(
        &mut self,
        gba: &GBA,
        animation_controller: &mut AnimationController<4>,
        game_board: &mut game_board::GameBoard,
        opponent: &mut Agent,
    ) -> Option<usize> {
        match self {
            Self::Cpu(_, ref mut face, ref mut turn) => {
                turn.update(animation_controller, game_board, face)
            }
            Self::Human(_, ref mut turn) => {
                let cpu_face = match opponent {
                    Agent::Human(_, _) => None,
                    Agent::Cpu(_, ref mut face, _) => Some(face),
                };

                turn.update(gba, animation_controller, game_board, cpu_face)
            }
        }
    }

    pub fn new_human_agent(color: TokenColor) -> Self {
        Self::Human(color, PlayerTurn::new(color))
    }

    pub fn new_cpu_agent(color: TokenColor, cpu_face: CpuFace<'a>) -> Self {
        Self::Cpu(color, cpu_face, CpuTurn::new(color))
    }
}

impl<'a> GameScreen<'a> {
    pub fn new(
        gba: &'a GBA,
        red_token_animation: &'a LoadedAnimation<4>,
        yellow_token_animation: &'a LoadedAnimation<4>,
        board_slot_sprite: &'a LoadedSprite<'a>,
        red_agent: Agent<'a>,
        yellow_agent: Agent<'a>,
    ) -> Self {
        let red_token_animation_controller = red_token_animation.create_controller(gba);
        let yellow_token_animation_controller = yellow_token_animation.create_controller(gba);

        // Create an Object entry for each slot that makes up the board.
        // We need to keep ownership of these in order to keep them in OBJRAM, so store them in an array.
        let _board_slot_objects = game_board::create_board_object_entries(board_slot_sprite, gba);

        // For now hardcode red player goes first.
        let game_state = GameState::TurnState(TokenColor::Red);

        let game_board = game_board::GameBoard::new(
            gba,
            red_token_animation.get_frame(0),
            yellow_token_animation.get_frame(0),
        );

        let _background = BOARD_BACKGROUND.load(gba);

        Self {
            gba,
            red_token_animation_controller,
            yellow_token_animation_controller,
            _board_slot_objects,
            game_state,
            game_board,
            _background,
            red_agent,
            yellow_agent,
        }
    }

    fn get_state(&self) -> GameState {
        self.game_state.clone()
    }

    fn update_turn(&mut self, token_color: TokenColor) -> Option<GameState> {
        let (animation_controller, agent, opponent) = match token_color {
            TokenColor::Red => (
                &mut self.red_token_animation_controller,
                &mut self.red_agent,
                &mut self.yellow_agent,
            ),
            TokenColor::Yellow => (
                &mut self.yellow_token_animation_controller,
                &mut self.yellow_agent,
                &mut self.red_agent,
            ),
        };

        let column = agent.update(
            self.gba,
            animation_controller,
            &mut self.game_board,
            opponent,
        );

        if let Some(column) = column {
            let row = self.game_board.get_next_free_row(column);

            match row {
                Some(row) => {
                    let obj_index = self.game_board.set_cell(token_color, column, row);
                    let y_pos = game_board::get_token_y_position();

                    let drop_state = TokenDroppingState {
                        token_color,
                        column,
                        row,
                        obj_index,
                        current_y: y_pos,
                        speed: TOKEN_DROP_STARTING_SPEED,
                        target_y: self.game_board.get_token_ypos_for_row(row),
                        num_bounces: 0,
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
        let i_current_y: i16 = state.current_y.try_into().unwrap();
        let new_y = i_current_y + state.speed;

        state.current_y = new_y.try_into().unwrap();
        state.current_y = min(state.current_y, state.target_y);

        state.speed = min(
            state.speed + TOKEN_DROP_SPEED_GRADIENT,
            TOKEN_DROP_TOP_SPEED,
        );

        self.update_token_dropping_obj(state);

        if state.current_y == state.target_y {
            let bounce_speed = -(state.speed / TOKEN_BOUNCE_SPEED_DECAY);

            if bounce_speed.abs() == 1 {
                // Bouncing animation has
                // Turn is over now.
                // Check victory conditions, otherwise move to next player's turn.
                if self
                    .game_board
                    .is_winning_token(state.column, state.row, state.token_color)
                {
                    // TODO - Transition to game over screen.
                    // if state.token_color == TokenColor::Red {
                    //     self.cpu_face.set_emotion(cpu_face::CpuEmotion::Sad);
                    // }

                    panic!("Game's over");
                } else {
                    Some(GameState::TurnState(state.token_color.opposite()))
                }
            } else {
                state.num_bounces += 1;
                state.speed = bounce_speed;

                None
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

impl TokenColor {
    pub fn opposite(&self) -> TokenColor {
        match self {
            TokenColor::Red => TokenColor::Yellow,
            TokenColor::Yellow => TokenColor::Red,
        }
    }
}

impl<'a> Screen for GameScreen<'a> {
    fn update(&mut self) -> Option<ScreenState> {
        let mut state = self.get_state();

        let new_state = match state {
            GameState::TurnState(token_color) => self.update_turn(token_color),
            GameState::TokenDropping(ref mut token_state) => {
                self.update_token_dropping(token_state)
            }
        };

        if let Some(new_state) = new_state {
            self.game_state = new_state;
        } else {
            self.game_state = state;
        }

        None
    }
}
