use core::cmp::min;

use self::cpu_face::CpuFace;

use super::{Screen, ScreenState};
use crate::graphics::background::{LoadedBackground, BOARD_BACKGROUND};
use crate::graphics::effects::Blinker;
use crate::graphics::sprite::{
    AnimationController, LoadedAnimation, LoadedObjectEntry, LoadedSprite, BOARD_SLOT_SPRITE,
    RED_TOKEN_ANIMATION, YELLOW_TOKEN_ANIMATION,
};
use crate::system::{constants::BOARD_SLOTS, gba::GBA};
use cpu_turn::CpuTurn;
use game_board::WinningPositions;
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

const WINNING_TOKEN_BLINK_TIME_ON: u32 = 22;
const WINNING_TOKEN_BLINK_TIME_OFF: u32 = 8;

pub enum Agent<'a> {
    Human(PlayerTurn),
    Cpu(CpuFace<'a>, CpuTurn),
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
struct GameOverState {
    winning_color: Option<TokenColor>,
    winning_token_positions: Option<WinningPositions>,
    blinker: Blinker,
}

#[derive(Clone)]
enum GameState {
    TurnState(TokenColor),
    TokenDropping(TokenDroppingState),
    GameOver(GameOverState),
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

pub struct GameScreenLoadedData<'a> {
    red_token_animation: LoadedAnimation<'a, 4>,
    yellow_token_animation: LoadedAnimation<'a, 4>,
    board_slot_sprite: LoadedSprite<'a>,
}

impl<'a> GameScreenLoadedData<'a> {
    pub fn new(gba: &'a GBA) -> Self {
        let yellow_token_animation = YELLOW_TOKEN_ANIMATION.load(gba);
        let red_token_animation = RED_TOKEN_ANIMATION.load(gba);
        let board_slot_sprite = BOARD_SLOT_SPRITE.load(gba);

        Self {
            yellow_token_animation,
            red_token_animation,
            board_slot_sprite,
        }
    }
}

// The agent enum just proxies the update call to the appropriate Turn struct.
impl<'a> Agent<'a> {
    pub fn update(
        &mut self,
        gba: &GBA,
        token_color: TokenColor,
        animation_controller: &mut AnimationController<4>,
        game_board: &mut game_board::GameBoard,
        opponent: &mut Agent,
    ) -> Option<usize> {
        match self {
            Self::Cpu(ref mut face, ref mut turn) => {
                turn.update(token_color, animation_controller, game_board, face)
            }
            Self::Human(ref mut turn) => {
                let cpu_face = match opponent {
                    Agent::Human(_) => None,
                    Agent::Cpu(ref mut face, _) => Some(face),
                };

                turn.update(gba, token_color, animation_controller, game_board, cpu_face)
            }
        }
    }

    pub fn new_human_agent() -> Self {
        Self::Human(PlayerTurn::new())
    }

    pub fn new_cpu_agent(cpu_face: CpuFace<'a>) -> Self {
        Self::Cpu(cpu_face, CpuTurn::new())
    }
}

impl<'a> GameScreen<'a> {
    pub fn new(
        gba: &'a GBA,
        loaded_data: &'a GameScreenLoadedData<'a>,
        red_agent: Agent<'a>,
        yellow_agent: Agent<'a>,
    ) -> Self {
        let red_token_animation_controller = loaded_data.red_token_animation.create_controller(gba);
        let yellow_token_animation_controller =
            loaded_data.yellow_token_animation.create_controller(gba);

        // Create an Object entry for each slot that makes up the board.
        // We need to keep ownership of these in order to keep them in OBJRAM, so store them in an array.
        let _board_slot_objects =
            game_board::create_board_object_entries(&loaded_data.board_slot_sprite, gba);

        // For now hardcode red player goes first.
        let game_state = GameState::TurnState(TokenColor::Red);

        let game_board = game_board::GameBoard::new(
            gba,
            loaded_data.red_token_animation.get_frame(0),
            loaded_data.yellow_token_animation.get_frame(0),
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
            token_color,
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
                let winning_positions = self.game_board.get_winning_token_positions(
                    state.column,
                    state.row,
                    state.token_color,
                );

                match winning_positions {
                    Some(winning_positions) => {
                        let new_state =
                            self.get_player_winning_state(state.token_color, winning_positions);
                        Some(new_state)
                    }
                    None => Some(GameState::TurnState(state.token_color.opposite())),
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

    fn update_game_over(&mut self, state: &mut GameOverState) -> Option<GameState> {
        state.blinker.update();

        if let Some(winning_positions) = state.winning_token_positions {
            for i in winning_positions {
                let mut token_obj = self.game_board.get_token_obj_entry_mut(i).as_mut().unwrap();
                state.blinker.apply_to_object(&mut token_obj);
                token_obj.commit_to_memory();
            }
        }

        None
    }

    fn get_agent<'b>(&'b mut self, token_color: TokenColor) -> &'b mut Agent<'a> {
        match token_color {
            TokenColor::Red => &mut self.red_agent,
            TokenColor::Yellow => &mut self.yellow_agent,
        }
    }

    fn get_player_winning_state(
        &mut self,
        winning_color: TokenColor,
        winning_token_positions: WinningPositions,
    ) -> GameState {
        let blinker = Blinker::new(
            WINNING_TOKEN_BLINK_TIME_ON,
            WINNING_TOKEN_BLINK_TIME_OFF,
            false,
        );

        let game_over_state = GameOverState {
            blinker,
            winning_color: Some(winning_color),
            winning_token_positions: Some(winning_token_positions),
        };

        // If the losing player is a CPU, then he becomes sad :(
        let losing_color = winning_color.opposite();
        let losing_agent = self.get_agent(losing_color);
        if let Agent::Cpu(ref mut cpu_face, _) = losing_agent {
            cpu_face.set_emotion(cpu_face::CpuEmotion::Sad);
        }

        GameState::GameOver(game_over_state)
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
            GameState::GameOver(ref mut game_over_state) => self.update_game_over(game_over_state),
        };

        if let Some(new_state) = new_state {
            self.game_state = new_state;
        } else {
            self.game_state = state;
        }

        None
    }
}
