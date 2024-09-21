use core::cmp::min;

use self::cpu_face::CpuFace;

use super::{Screen, ScreenState};
use crate::audio::assets::BOUNCE_NOISE;
use crate::audio::mixer::{self, AudioSource, AudioVolume};
use crate::graphics::background::{
    BackgroundLayer, LoadedBackground, BOARD_BACKGROUND, CLOUDS_CLOSE_BACKGROUND,
    CLOUDS_FAR_BACKGROUND,
};
use crate::graphics::effects::background_scroller::BackgroundScroller;
use crate::graphics::effects::blending::BlendController;
use crate::graphics::effects::blinker::Blinker;
use crate::graphics::sprite::{
    AnimationController, LoadedAnimation, LoadedObjectEntry, LoadedSprite, BOARD_SLOT_SPRITE,
    CPU_TEXT_SPRITE, DRAW_TEXT_SPRITE, MENU_CURSOR_ANIMATION, MENU_CURSOR_FRAME_0_SPRITE,
    P1_TEXT_SPRITE, P2_TEXT_SPRITE, QUIT_TEXT_SPRITE, RED_TOKEN_ANIMATION, REMATCH_TEXT_SPRITE,
    WINS_TEXT_RED_SPRITE, WINS_TEXT_YELLOW_SPRITE, YELLOW_TOKEN_ANIMATION,
};
use crate::system::constants::{SCREEN_CENTER, SCREEN_WIDTH};
use crate::system::gba::GbaKey;
use crate::system::{constants::BOARD_SLOTS, gba::GBA};
use cpu_turn::CpuTurn;
use game_board::WinningPositions;
use gba::video::{BlendControl, ColorEffectMode};
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

const WIN_TEXT_WORD_SPACING: u16 = 4;
const WIN_TEXT_YPOS: u16 = 5;

const GAME_OVER_MENU_YPOS: u16 = 26;
const MENU_TEXT_HORIZ_MARGIN: u16 = 10;
const CURSOR_X_OFFSET: u16 = 10;

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
struct Winner {
    token_positions: WinningPositions,
    blinker: Blinker,
}

#[derive(Clone)]
enum GameOutcome {
    Winner(Winner),
    Draw,
}

#[derive(Clone)]
enum CursorPosition {
    Rematch,
    Quit,
}

#[derive(Clone)]
struct GameOverState {
    outcome: GameOutcome,
    cursor_position: CursorPosition,
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
    p1_text_object: LoadedObjectEntry<'a>,
    p2_text_object: LoadedObjectEntry<'a>,
    cpu_text_object: LoadedObjectEntry<'a>,
    red_wins_text_object: LoadedObjectEntry<'a>,
    yellow_wins_text_object: LoadedObjectEntry<'a>,
    draw_text_object: LoadedObjectEntry<'a>,
    clouds_background_close: LoadedBackground<'a>,
    clouds_background_far: LoadedBackground<'a>,
    _blend_controller: BlendController,
    cloud_scroller_close: BackgroundScroller,
    cloud_scroller_far: BackgroundScroller,
    rematch_text_object: LoadedObjectEntry<'a>,
    quit_text_object: LoadedObjectEntry<'a>,
    menu_cursor_animation_controller: AnimationController<'a, 5>,
}

pub struct GameScreenLoadedData<'a> {
    red_token_animation: LoadedAnimation<'a, 4>,
    yellow_token_animation: LoadedAnimation<'a, 4>,
    board_slot_sprite: LoadedSprite<'a>,
    p1_text_sprite: LoadedSprite<'a>,
    p2_text_sprite: LoadedSprite<'a>,
    cpu_text_sprite: LoadedSprite<'a>,
    red_wins_text_sprite: LoadedSprite<'a>,
    yellow_wins_text_sprite: LoadedSprite<'a>,
    draw_text_sprite: LoadedSprite<'a>,
    rematch_text_sprite: LoadedSprite<'a>,
    quit_text_sprite: LoadedSprite<'a>,
    menu_cursor_animation: LoadedAnimation<'a, 5>,
}

impl<'a> GameScreenLoadedData<'a> {
    pub fn new(gba: &'a GBA) -> Self {
        let yellow_token_animation = YELLOW_TOKEN_ANIMATION.load(gba);
        let red_token_animation = RED_TOKEN_ANIMATION.load(gba);
        let board_slot_sprite = BOARD_SLOT_SPRITE.load(gba);

        let p1_text_sprite = P1_TEXT_SPRITE.load(gba);
        let p2_text_sprite = P2_TEXT_SPRITE.load(gba);
        let cpu_text_sprite = CPU_TEXT_SPRITE.load(gba);
        let red_wins_text_sprite = WINS_TEXT_RED_SPRITE.load(gba);
        let yellow_wins_text_sprite = WINS_TEXT_YELLOW_SPRITE.load(gba);

        let draw_text_sprite = DRAW_TEXT_SPRITE.load(gba);

        let rematch_text_sprite = REMATCH_TEXT_SPRITE.load(gba);
        let quit_text_sprite = QUIT_TEXT_SPRITE.load(gba);
        let menu_cursor_animation = MENU_CURSOR_ANIMATION.load(gba);

        Self {
            yellow_token_animation,
            red_token_animation,
            board_slot_sprite,
            p1_text_sprite,
            p2_text_sprite,
            cpu_text_sprite,
            red_wins_text_sprite,
            yellow_wins_text_sprite,
            draw_text_sprite,
            rematch_text_sprite,
            quit_text_sprite,
            menu_cursor_animation,
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
        starting_color: TokenColor,
    ) -> Self {
        let mut red_token_animation_controller =
            loaded_data.red_token_animation.create_controller(gba);
        let mut yellow_token_animation_controller =
            loaded_data.yellow_token_animation.create_controller(gba);

        red_token_animation_controller.set_hidden();
        yellow_token_animation_controller.set_hidden();

        // Create an Object entry for each slot that makes up the board.
        // We need to keep ownership of these in order to keep them in OBJRAM, so store them in an array.
        let _board_slot_objects =
            game_board::create_board_object_entries(&loaded_data.board_slot_sprite, gba);

        let game_state = GameState::TurnState(starting_color);

        let game_board = game_board::GameBoard::new(
            gba,
            loaded_data.red_token_animation.get_frame(0),
            loaded_data.yellow_token_animation.get_frame(0),
        );

        let p1_text_object = loaded_data
            .p1_text_sprite
            .create_obj_attr_entry(gba)
            .with_hidden();
        let p2_text_object = loaded_data
            .p2_text_sprite
            .create_obj_attr_entry(gba)
            .with_hidden();
        let cpu_text_object = loaded_data
            .cpu_text_sprite
            .create_obj_attr_entry(gba)
            .with_hidden();
        let red_wins_text_object = loaded_data
            .red_wins_text_sprite
            .create_obj_attr_entry(gba)
            .with_hidden();
        let yellow_wins_text_object = loaded_data
            .yellow_wins_text_sprite
            .create_obj_attr_entry(gba)
            .with_hidden();

        let draw_text_object = loaded_data
            .draw_text_sprite
            .create_obj_attr_entry(gba)
            .with_hidden();

        let _background = BOARD_BACKGROUND.load(gba, BackgroundLayer::Bg0);
        let clouds_background_far = CLOUDS_FAR_BACKGROUND.load(gba, BackgroundLayer::Bg1);
        let clouds_background_close = CLOUDS_CLOSE_BACKGROUND.load(gba, BackgroundLayer::Bg2);
        let mut blend_controller = BlendController::new();

        // Target 1 is on top of Target 2
        blend_controller.update(
            BlendControl::new()
                .with_mode(ColorEffectMode::AlphaBlend)
                .with_target2_bg0(true)
                .with_target1_bg1(true)
                .with_target1_bg2(true),
            [25, 7].into(),
        );

        let cloud_scroller_close = BackgroundScroller::new(1, 0).with_divisor(5);
        let cloud_scroller_far = BackgroundScroller::new(1, 0).with_divisor(8);

        let rematch_text_object = loaded_data
            .rematch_text_sprite
            .create_obj_attr_entry(gba)
            .with_hidden();
        let quit_text_object = loaded_data
            .quit_text_sprite
            .create_obj_attr_entry(gba)
            .with_hidden();
        let mut menu_cursor_animation_controller =
            loaded_data.menu_cursor_animation.create_controller(gba);

        menu_cursor_animation_controller.set_hidden();

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
            p1_text_object,
            p2_text_object,
            cpu_text_object,
            red_wins_text_object,
            yellow_wins_text_object,
            draw_text_object,
            clouds_background_close,
            clouds_background_far,
            cloud_scroller_close,
            cloud_scroller_far,
            rematch_text_object,
            quit_text_object,
            menu_cursor_animation_controller,
            _blend_controller: blend_controller,
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
                    None => {
                        if self.game_board.is_full() {
                            Some(self.get_draw_game_state())
                        } else {
                            Some(GameState::TurnState(state.token_color.opposite()))
                        }
                    }
                }
            } else {
                // noise::play_impact_noise();
                let sound = AudioSource::new(BOUNCE_NOISE, AudioVolume::new(10), false);
                mixer::set_channel_2(sound);

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
            }
            None => {}
        }
    }

    fn update_game_over(&mut self, game_over_state: &mut GameOverState) -> Option<ScreenState> {
        if let GameOutcome::Winner(ref mut winner) = &mut game_over_state.outcome {
            winner.blinker.update();
            for i in winner.token_positions {
                let mut token_obj = self.game_board.get_token_obj_entry_mut(i).as_mut().unwrap();
                winner.blinker.apply_to_object(&mut token_obj);
            }
        }

        if self.gba.key_was_pressed(GbaKey::A) || self.gba.key_was_pressed(GbaKey::START) {
            return match game_over_state.cursor_position {
                CursorPosition::Quit => Some(ScreenState::TitleScreen),
                CursorPosition::Rematch => match self.yellow_agent {
                    Agent::Cpu(_, _) => Some(ScreenState::VsCpuSpinnerScreen),
                    Agent::Human(_) => Some(ScreenState::VsPlayerSpinnerScreen),
                },
            };
        }

        // Update the menu position.
        if self.gba.key_was_pressed(GbaKey::LEFT) || self.gba.key_was_pressed(GbaKey::RIGHT) {
            game_over_state.cursor_position = game_over_state.cursor_position.next();
        }

        self.update_cursor_object(&game_over_state.cursor_position);

        None
    }

    fn update_cursor_object(&mut self, position: &CursorPosition) {
        let target_obj = match position {
            CursorPosition::Rematch => self.rematch_text_object.get_obj_attr_data(),
            CursorPosition::Quit => self.quit_text_object.get_obj_attr_data(),
        };

        let target_obj_x = target_obj.1.x();
        let cursor_x = target_obj_x - CURSOR_X_OFFSET;

        let cursor_obj = self.menu_cursor_animation_controller.get_obj_attr_entry();
        cursor_obj.set_visible();
        let cursor_oa = cursor_obj.get_obj_attr_data();
        cursor_oa.set_x(cursor_x);
        self.menu_cursor_animation_controller.tick();
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

        let outcome = GameOutcome::Winner(Winner {
            token_positions: winning_token_positions,
            blinker,
        });

        // If the losing player is a CPU, then he becomes sad :(
        let losing_color = winning_color.opposite();
        let losing_agent = self.get_agent(losing_color);
        if let Agent::Cpu(ref mut cpu_face, _) = losing_agent {
            cpu_face.set_emotion(cpu_face::CpuEmotion::Sad);
        }

        // Add the "{Player} Wins" banner.
        let winning_player_obj = if winning_color == TokenColor::Red {
            &mut self.p1_text_object
        } else {
            let winning_agent = self.get_agent(winning_color);
            if let Agent::Human(_) = winning_agent {
                &mut self.p2_text_object
            } else {
                &mut self.cpu_text_object
            }
        };

        let wins_text_obj = match winning_color {
            TokenColor::Red => &mut self.red_wins_text_object,
            TokenColor::Yellow => &mut self.yellow_wins_text_object,
        };

        let player_name_width: u16 = winning_player_obj
            .loaded_sprite()
            .sprite()
            .width()
            .try_into()
            .unwrap();
        let wins_text_width: u16 = wins_text_obj
            .loaded_sprite()
            .sprite()
            .width()
            .try_into()
            .unwrap();
        let total_width = player_name_width + wins_text_width + WIN_TEXT_WORD_SPACING;

        let player_text_xpos = SCREEN_WIDTH / 2 - total_width / 2;
        let oa = winning_player_obj.get_obj_attr_data();
        oa.set_x(player_text_xpos);
        oa.set_y(WIN_TEXT_YPOS);
        winning_player_obj.set_visible();

        let wins_text_xpos = player_text_xpos + player_name_width + WIN_TEXT_WORD_SPACING;
        let oa = wins_text_obj.get_obj_attr_data();
        oa.set_x(wins_text_xpos);
        oa.set_y(WIN_TEXT_YPOS);
        wins_text_obj.set_visible();

        let game_over_state = GameOverState {
            outcome,
            cursor_position: CursorPosition::Rematch,
        };

        self.init_game_over_menu();

        GameState::GameOver(game_over_state)
    }

    fn get_draw_game_state(&mut self) -> GameState {
        let outcome = GameOutcome::Draw;

        let draw_text_width: u16 = self
            .draw_text_object
            .loaded_sprite()
            .sprite()
            .width()
            .try_into()
            .unwrap();

        for color in [TokenColor::Yellow, TokenColor::Red] {
            let agent = self.get_agent(color);
            if let Agent::Cpu(ref mut face, _) = agent {
                face.set_emotion(cpu_face::CpuEmotion::Surprised)
            }
        }

        let x_pos = (SCREEN_WIDTH - draw_text_width) / 2;
        let oa = self.draw_text_object.get_obj_attr_data();
        oa.set_x(x_pos);
        oa.set_y(WIN_TEXT_YPOS);
        self.draw_text_object.set_visible();

        let game_over_state = GameOverState {
            outcome,
            cursor_position: CursorPosition::Quit,
        };

        self.init_game_over_menu();

        GameState::GameOver(game_over_state)
    }

    fn init_game_over_menu(&mut self) {
        let rematch_oa = self.rematch_text_object.get_obj_attr_data();
        let rematch_text_width: u16 = REMATCH_TEXT_SPRITE.width().try_into().unwrap();
        let rematch_text_height: u16 = REMATCH_TEXT_SPRITE.height().try_into().unwrap();
        let rematch_text_xpos = SCREEN_CENTER.0 - rematch_text_width - MENU_TEXT_HORIZ_MARGIN;

        rematch_oa.set_x(rematch_text_xpos);
        rematch_oa.set_y(GAME_OVER_MENU_YPOS);
        self.rematch_text_object.set_visible();

        let rematch_x_center = rematch_text_xpos + (rematch_text_width / 2);
        let rematch_center_offset = SCREEN_CENTER.0 - rematch_x_center;

        let quit_oa = self.quit_text_object.get_obj_attr_data();
        let quit_text_width: u16 = QUIT_TEXT_SPRITE.width().try_into().unwrap();
        let quit_text_xpos = (SCREEN_CENTER.0 + rematch_center_offset) - (quit_text_width / 2);

        quit_oa.set_x(quit_text_xpos);
        quit_oa.set_y(GAME_OVER_MENU_YPOS);
        self.quit_text_object.set_visible();

        let cursor_height: u16 = MENU_CURSOR_FRAME_0_SPRITE.height().try_into().unwrap();
        let menu_y_center = GAME_OVER_MENU_YPOS + (rematch_text_height / 2);
        let cursor_ypos = menu_y_center - (cursor_height / 2);

        let cursor_obj = self.menu_cursor_animation_controller.get_obj_attr_entry();
        let cursor_oa = cursor_obj.get_obj_attr_data();
        cursor_oa.set_y(cursor_ypos);
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
        self.cloud_scroller_close.update();
        self.cloud_scroller_close
            .apply_to_background(&self.clouds_background_close);

        self.cloud_scroller_far.update();
        self.cloud_scroller_far
            .apply_to_background(&self.clouds_background_far);

        let mut state = self.get_state();

        let new_state = match state {
            GameState::TurnState(token_color) => self.update_turn(token_color),
            GameState::TokenDropping(ref mut token_state) => {
                self.update_token_dropping(token_state)
            }
            GameState::GameOver(ref mut game_over_state) => {
                let next_screen = self.update_game_over(game_over_state);
                if next_screen.is_some() {
                    return next_screen;
                }
                None
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

impl CursorPosition {
    pub fn next(&self) -> Self {
        match self {
            Self::Quit => Self::Rematch,
            Self::Rematch => Self::Quit,
        }
    }
}
