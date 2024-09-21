use gba::{prelude::TIMER3_COUNT, random::Lcg32};

use crate::{
    audio::drum_roll::DrumRoll,
    graphics::effects::spinner::Spinner,
    graphics::{
        background::{BackgroundLayer, LoadedBackground, SCROLLER_BACKGROUND, SPINNER_BACKGROUND},
        effects::{background_scroller::BackgroundScroller, blinker::Blinker},
        sprite::{
            AffineLoadedObjectEntry, AnimationController, LoadedAnimation, LoadedObjectEntry,
            LoadedSprite, CPU_TEXT_SPRITE, P1_TEXT_SPRITE, P2_TEXT_SPRITE, PRESS_A_ANIMATION,
            PRESS_A_FRAME_0_SPRITE, SPINNER_ARROW_SPRITE,
        },
    },
    system::{
        constants::{SCREEN_HEIGHT, SCREEN_WIDTH},
        gba::{GbaKey, GBA},
    },
};

use super::{game_screen::TokenColor, Screen, ScreenState};

const ARROW_POSITION: (u16, u16) = (56, 32);
const PRESS_A_OFFSET: u16 = 4;
const PLAYER_TEXT_X_OFFSET: u16 = 70;
const PLAYER_TEXT_Y_POSITION: u16 = 85;
const FINISHED_STATE_TIME: u32 = 100;
const BLINK_TIME_ON: u32 = 22;
const BLINK_TIME_OFF: u32 = 8;

pub enum SpinnerMode {
    VsCpu,
    VsPlayer,
}

#[derive(Clone)]
struct FinishedState {
    timer: u32,
    color: TokenColor,
}

#[derive(Clone)]
enum SpinnerScreenState {
    PressA,                  // Waiting for player to press a to begin the spinner.
    Spinning,                // Waiting for spinner to finish spinning
    Finished(FinishedState), // Wait time before game begins, flash the player icon that won the spin.
}

pub struct SpinnerScreenLoadedData<'a> {
    loaded_sprite: LoadedSprite<'a>,
    press_a_animation: LoadedAnimation<'a, 2>,
    red_player_icon: LoadedSprite<'a>,
    yellow_player_icon: LoadedSprite<'a>,
}

pub struct SpinnerScreen<'a> {
    arrow_sprite: AffineLoadedObjectEntry<'a>,
    spinner: Spinner,
    _spinner_background: LoadedBackground<'a>,
    scrolling_background: LoadedBackground<'a>,
    state: SpinnerScreenState,
    press_a_animation_controller: AnimationController<'a, 2>,
    mode: SpinnerMode,
    gba: &'a GBA,
    red_player_obj: LoadedObjectEntry<'a>,
    yellow_player_obj: LoadedObjectEntry<'a>,
    blinker: Blinker,
    background_scroller: BackgroundScroller,
    drum_roll: DrumRoll,
}

impl<'a> SpinnerScreenLoadedData<'a> {
    pub fn new(gba: &'a GBA, mode: &SpinnerMode) -> Self {
        let loaded_sprite = SPINNER_ARROW_SPRITE.load(gba);
        let press_a_animation = PRESS_A_ANIMATION.load(gba);

        let red_player_icon = P1_TEXT_SPRITE.load(gba);
        let yellow_player_icon = match mode {
            SpinnerMode::VsCpu => &CPU_TEXT_SPRITE,
            SpinnerMode::VsPlayer => &P2_TEXT_SPRITE,
        }
        .load(gba);

        Self {
            loaded_sprite,
            press_a_animation,
            red_player_icon,
            yellow_player_icon,
        }
    }
}

impl<'a> Screen for SpinnerScreen<'a> {
    fn update(&mut self) -> Option<super::ScreenState> {
        self.background_scroller.update();

        self.background_scroller
            .apply_to_background(&self.scrolling_background);

        let state = self.state.clone();

        match state {
            SpinnerScreenState::PressA => {
                self.update_press_a();
                None
            }
            SpinnerScreenState::Spinning => {
                self.update_spinning();
                None
            }
            SpinnerScreenState::Finished(state_struct) => self.update_finished_state(state_struct),
        }
    }
}

impl<'a> SpinnerScreen<'a> {
    pub fn new(gba: &'a GBA, loaded_data: &'a SpinnerScreenLoadedData, mode: SpinnerMode) -> Self {
        let mut arrow_sprite = loaded_data
            .loaded_sprite
            .create_obj_attr_entry(gba)
            .into_affine(gba);

        let oa = arrow_sprite.get_obj_attr_data();
        oa.set_x(ARROW_POSITION.0);
        oa.set_y(ARROW_POSITION.1);

        arrow_sprite.get_affine_matrix().commit_to_memory();

        let spinner = Spinner::new(0xFFFF, 0x000F);

        let spinner_background = SPINNER_BACKGROUND.load(gba, BackgroundLayer::Bg1);
        let scrolling_background = SCROLLER_BACKGROUND.load(gba, BackgroundLayer::Bg0);

        let mut press_a_animation_controller = loaded_data.press_a_animation.create_controller(gba);
        let press_a_height: u16 = PRESS_A_FRAME_0_SPRITE.height().try_into().unwrap();
        let press_a_width: u16 = PRESS_A_FRAME_0_SPRITE.width().try_into().unwrap();

        let press_a_obj = press_a_animation_controller.get_obj_attr_entry();
        let press_a_oa = press_a_obj.get_obj_attr_data();
        press_a_oa.set_x(SCREEN_WIDTH - press_a_width - PRESS_A_OFFSET);
        press_a_oa.set_y(SCREEN_HEIGHT - press_a_height - PRESS_A_OFFSET);

        let mut red_player_obj = loaded_data.red_player_icon.create_obj_attr_entry(gba);
        let red_player_text_width: u16 = red_player_obj
            .loaded_sprite()
            .sprite()
            .width()
            .try_into()
            .unwrap();

        let red_player_oa = red_player_obj.get_obj_attr_data();
        red_player_oa.set_x((SCREEN_WIDTH / 2) - PLAYER_TEXT_X_OFFSET - red_player_text_width);
        red_player_oa.set_y(PLAYER_TEXT_Y_POSITION);

        let mut yellow_player_obj = loaded_data.yellow_player_icon.create_obj_attr_entry(gba);
        let yellow_player_oa = yellow_player_obj.get_obj_attr_data();

        yellow_player_oa.set_x((SCREEN_WIDTH / 2) + PLAYER_TEXT_X_OFFSET);
        yellow_player_oa.set_y(PLAYER_TEXT_Y_POSITION);

        let blinker = Blinker::new(BLINK_TIME_ON, BLINK_TIME_OFF, false);
        let background_scroller = BackgroundScroller::new(0, 1).with_divisor(2);

        let drum_roll = DrumRoll::new(1);

        Self {
            gba,
            arrow_sprite,
            spinner,
            press_a_animation_controller,
            mode,
            red_player_obj,
            yellow_player_obj,
            blinker,
            scrolling_background,
            background_scroller,
            drum_roll,
            state: SpinnerScreenState::PressA,
            _spinner_background: spinner_background,
        }
    }

    fn update_press_a(&mut self) {
        self.press_a_animation_controller.tick();
        if self.gba.key_was_pressed(GbaKey::A) {
            self.enter_spinning_state();
        }
    }

    fn enter_spinning_state(&mut self) {
        // Hide the press a indicator.
        self.press_a_animation_controller.set_hidden();

        let seed: u32 = TIMER3_COUNT.read().into();
        let mut rng = Lcg32::new(seed);
        let starting_speed = (rng.next_u32() & 0x00FF) as u16 | 0x0F00;
        self.spinner.set_speed(starting_speed);

        self.state = SpinnerScreenState::Spinning;
    }

    fn update_spinning(&mut self) {
        self.spinner.update();
        self.spinner.apply_to_object(&mut self.arrow_sprite);
        self.arrow_sprite.get_affine_matrix().commit_to_memory();

        // The max spinning speed is 0x0FFF.
        // We need to decrease the speed of the drums linearly as the speed drops down to zero.
        // The drum delay counts frames so we don't want it to grow very high.
        // The shift by 8 slows down the decay.
        self.drum_roll.update();
        let drum_delay = 2 + ((0x1000 - self.spinner.speed()) >> 8);

        self.drum_roll.set_delay(drum_delay.into());

        if self.spinner.finished() {
            self.enter_finished_state();
        }
    }

    fn enter_finished_state(&mut self) {
        let rot = self.spinner.rotation();
        let quarter_turn = 0xFFFF / 4;
        let three_quarter_turn = quarter_turn * 3;

        let color = if rot > quarter_turn && rot < three_quarter_turn {
            TokenColor::Red
        } else {
            TokenColor::Yellow
        };

        let state = FinishedState {
            timer: FINISHED_STATE_TIME,
            color,
        };

        self.state = SpinnerScreenState::Finished(state);
    }

    fn update_finished_state(&mut self, mut state: FinishedState) -> Option<ScreenState> {
        state.timer -= 1;

        // apply blinker.
        let target_obj = match state.color {
            TokenColor::Red => &mut self.red_player_obj,
            TokenColor::Yellow => &mut self.yellow_player_obj,
        };

        self.blinker.update();
        self.blinker.apply_to_object(target_obj);

        let should_transition = state.timer == 0;

        let starting_color = state.color;

        self.state = SpinnerScreenState::Finished(state);

        if should_transition {
            let next_screen = match self.mode {
                SpinnerMode::VsCpu => ScreenState::VsCpuScreen(starting_color),
                SpinnerMode::VsPlayer => ScreenState::VsPlayerScreen(starting_color),
            };
            Some(next_screen)
        } else {
            None
        }
    }
}
