use gba::prelude::ObjDisplayStyle;

use crate::{
    audio::noise::play_menu_move_noise,
    graphics::{
        background::{
            BackgroundLayer, LoadedBackground, SCROLLER_BACKGROUND, TITLE_SCREEN_BACKGROUND,
        },
        effects::{background_scroller::BackgroundScroller, blinker::Blinker},
        sprite::{
            AnimationController, LoadedAnimation, LoadedObjectEntry, LoadedSprite,
            MENU_CURSOR_ANIMATION, PRESS_TEXT_SPRITE, START_TEXT_SPRITE, VS_CPU_TEXT_SPRITE,
            VS_PLAYER_TEXT_SPRITE,
        },
    },
    system::{
        constants::SCREEN_WIDTH,
        gba::{GbaKey, GBA},
    },
};

use super::{
    game_screen::cpu_face::{CpuEmotion, CpuFace, CpuSprites},
    Screen, ScreenState,
};

const MENU_TEXT_Y: u16 = 140;
const MENU_TEXT_HORIZ_MARGIN: u16 = 12;
const CURSOR_X_OFFSET: u16 = 10;

const BLINK_TIME_ON: u32 = 40;
const BLINK_TIME_OFF: u32 = 10;

const CPU_HEAD_POS: (u16, u16) = (140, 52);
const GAME_TRANSITION_TIME: u16 = 40;

#[derive(Clone, Debug)]
enum MenuEntry {
    VsCpu,
    VsPlayer,
}

#[derive(Clone)]
struct PressStartState {
    blinker: Blinker,
}

#[derive(Clone)]
struct MenuState {
    cursor_position: MenuEntry,
}

#[derive(Clone)]
struct TransitionState {
    game_mode: MenuEntry,
    timer: u16,
}

#[derive(Clone)]
enum TitleScreenState {
    PressStart(PressStartState),
    Menu(MenuState),
    GameTransition(TransitionState),
}

pub struct TitleScreen<'a> {
    gba: &'a GBA,
    press_text_object: LoadedObjectEntry<'a>,
    start_text_object: LoadedObjectEntry<'a>,
    vs_cpu_text_object: LoadedObjectEntry<'a>,
    vs_player_text_object: LoadedObjectEntry<'a>,
    cursor_animation_controller: AnimationController<'a, 5>,
    scrolling_background: LoadedBackground<'a>,
    _logo_background: LoadedBackground<'a>,
    state: TitleScreenState,
    cpu_face: CpuFace<'a>,
    background_scroller: BackgroundScroller,
}

pub struct TitleScreenLoadedData<'a> {
    press_text_sprite: LoadedSprite<'a>,
    start_text_sprite: LoadedSprite<'a>,
    vs_cpu_text_sprite: LoadedSprite<'a>,
    vs_player_text_sprite: LoadedSprite<'a>,
    cursor_animation: LoadedAnimation<'a, 5>,
    cpu_sprites: CpuSprites<'a>,
}

impl MenuEntry {
    fn next(&self) -> Self {
        match self {
            Self::VsCpu => Self::VsPlayer,
            Self::VsPlayer => Self::VsCpu,
        }
    }
}

impl<'a> TitleScreenLoadedData<'a> {
    pub fn new(gba: &'a GBA) -> Self {
        let press_text_sprite = PRESS_TEXT_SPRITE.load(gba);
        let start_text_sprite = START_TEXT_SPRITE.load(gba);

        let vs_player_text_sprite = VS_PLAYER_TEXT_SPRITE.load(gba);
        let vs_cpu_text_sprite = VS_CPU_TEXT_SPRITE.load(gba);
        let cursor_animation = MENU_CURSOR_ANIMATION.load(gba);
        let cpu_sprites = CpuSprites::new(gba);

        Self {
            press_text_sprite,
            start_text_sprite,
            vs_player_text_sprite,
            vs_cpu_text_sprite,
            cursor_animation,
            cpu_sprites,
        }
    }
}

impl<'a> TitleScreen<'a> {
    pub fn new(gba: &'a GBA, loaded_data: &'a TitleScreenLoadedData<'a>) -> Self {
        let logo_background = TITLE_SCREEN_BACKGROUND.load(gba, BackgroundLayer::Bg1);
        let scrolling_background = SCROLLER_BACKGROUND.load(gba, BackgroundLayer::Bg0);

        let mut press_text_object = loaded_data.press_text_sprite.create_obj_attr_entry(gba);

        let press_oa = press_text_object.get_obj_attr_data();
        press_oa.set_x(85);
        press_oa.set_y(MENU_TEXT_Y);

        let mut start_text_object = loaded_data.start_text_sprite.create_obj_attr_entry(gba);

        let start_oa = start_text_object.get_obj_attr_data();
        start_oa.set_x(120);
        start_oa.set_y(MENU_TEXT_Y);

        let mut vs_cpu_text_object = loaded_data.vs_cpu_text_sprite.create_obj_attr_entry(gba);
        let vs_cpu_sprite_width: u16 = loaded_data
            .vs_cpu_text_sprite
            .sprite()
            .width()
            .try_into()
            .unwrap();

        let vs_cpu_oa = vs_cpu_text_object.get_obj_attr_data();
        vs_cpu_oa.set_style(ObjDisplayStyle::NotDisplayed);
        vs_cpu_oa.set_x(SCREEN_WIDTH / 4 - vs_cpu_sprite_width / 2 + MENU_TEXT_HORIZ_MARGIN);
        vs_cpu_oa.set_y(MENU_TEXT_Y);

        let mut vs_player_text_object =
            loaded_data.vs_player_text_sprite.create_obj_attr_entry(gba);
        let vs_player_sprite_width: u16 = loaded_data
            .vs_player_text_sprite
            .sprite()
            .width()
            .try_into()
            .unwrap();
        let vs_player_oa = vs_player_text_object.get_obj_attr_data();
        vs_player_oa.set_style(ObjDisplayStyle::NotDisplayed);
        vs_player_oa.set_x(
            SCREEN_WIDTH / 4 - vs_player_sprite_width / 2 + SCREEN_WIDTH / 2
                - MENU_TEXT_HORIZ_MARGIN,
        );
        vs_player_oa.set_y(MENU_TEXT_Y);

        let mut cursor_animation_controller = loaded_data.cursor_animation.create_controller(gba);
        cursor_animation_controller.set_hidden();
        let cursor_obj = cursor_animation_controller.get_obj_attr_entry();
        let cursor_oa = cursor_obj.get_obj_attr_data();
        cursor_oa.set_y(MENU_TEXT_Y + 2);

        let state = TitleScreenState::PressStart(PressStartState {
            blinker: Blinker::new(BLINK_TIME_ON, BLINK_TIME_OFF, true),
        });

        let mut cpu_face = CpuFace::new(gba, &loaded_data.cpu_sprites);
        cpu_face.set_x(CPU_HEAD_POS.0);
        cpu_face.set_y(CPU_HEAD_POS.1);

        let background_scroller = BackgroundScroller::new(0, 1).with_divisor(2);

        Self {
            gba,
            press_text_object,
            start_text_object,
            state,
            vs_cpu_text_object,
            vs_player_text_object,
            cursor_animation_controller,
            cpu_face,
            scrolling_background,
            background_scroller,
            _logo_background: logo_background,
        }
    }

    fn update_press_start(&mut self, mut press_start_state: PressStartState) {
        press_start_state.blinker.update();
        press_start_state
            .blinker
            .apply_to_object(&mut self.press_text_object);

        press_start_state
            .blinker
            .apply_to_object(&mut self.start_text_object);

        self.state = TitleScreenState::PressStart(press_start_state);

        if self.gba.key_was_pressed(GbaKey::START) {
            self.enter_menu();
        }
    }

    fn update_menu(&mut self, mut menu_state: MenuState) {
        if self.gba.key_was_pressed(GbaKey::LEFT) || self.gba.key_was_pressed(GbaKey::RIGHT) {
            play_menu_move_noise();
            menu_state.cursor_position = menu_state.cursor_position.next();
        };

        self.update_cursor_object(&menu_state);
        self.update_cpu_expression(&menu_state);

        if self.gba.key_was_pressed(GbaKey::START) || self.gba.key_was_pressed(GbaKey::A) {
            play_menu_move_noise();
            self.enter_transition(menu_state.cursor_position);
        } else {
            self.state = TitleScreenState::Menu(menu_state);
        }
    }

    fn update_transition(&mut self, mut transition_state: TransitionState) -> Option<ScreenState> {
        transition_state.timer -= 1;

        if transition_state.timer == 0 {
            match transition_state.game_mode {
                MenuEntry::VsCpu => Some(ScreenState::VsCpuSpinnerScreen),
                MenuEntry::VsPlayer => Some(ScreenState::VsPlayerSpinnerScreen),
            }
        } else {
            self.state = TitleScreenState::GameTransition(transition_state);
            None
        }
    }

    fn enter_menu(&mut self) {
        self.hide_press_start_text();

        play_menu_move_noise();

        let menu_state = MenuState {
            cursor_position: MenuEntry::VsCpu,
        };

        self.update_cursor_object(&menu_state);

        // Show menu items and cursor.
        for obj in [
            &mut self.vs_cpu_text_object,
            &mut self.vs_player_text_object,
            &mut self.cursor_animation_controller.get_obj_attr_entry(),
        ] {
            let oa = obj.get_obj_attr_data();
            oa.set_style(ObjDisplayStyle::Normal);
        }

        self.state = TitleScreenState::Menu(menu_state)
    }

    fn enter_transition(&mut self, game_mode: MenuEntry) {
        // Hide cursor.
        let cursor_obj = self.cursor_animation_controller.get_obj_attr_entry();
        let cursor_oa = cursor_obj.get_obj_attr_data();
        cursor_oa.set_style(ObjDisplayStyle::NotDisplayed);
        self.cursor_animation_controller.tick();

        // Set CPU emotion.
        let cpu_emotion = match game_mode {
            MenuEntry::VsCpu => CpuEmotion::Surprised,
            MenuEntry::VsPlayer => CpuEmotion::Sad,
        };

        self.cpu_face.set_emotion(cpu_emotion);

        let transition_state = TransitionState {
            game_mode,
            timer: GAME_TRANSITION_TIME,
        };
        self.state = TitleScreenState::GameTransition(transition_state);
    }

    fn update_cursor_object(&mut self, menu_state: &MenuState) {
        let target_obj = match menu_state.cursor_position {
            MenuEntry::VsCpu => self.vs_cpu_text_object.get_obj_attr_data(),
            MenuEntry::VsPlayer => self.vs_player_text_object.get_obj_attr_data(),
        };
        let target_obj_x = target_obj.1.x();
        let cursor_x = target_obj_x - CURSOR_X_OFFSET;

        let cursor_obj = self.cursor_animation_controller.get_obj_attr_entry();
        let cursor_oa = cursor_obj.get_obj_attr_data();
        cursor_oa.set_x(cursor_x);
        self.cursor_animation_controller.tick();
    }

    fn update_cpu_expression(&mut self, menu_state: &MenuState) {
        let cpu_emotion = match menu_state.cursor_position {
            MenuEntry::VsCpu => CpuEmotion::Happy,
            MenuEntry::VsPlayer => CpuEmotion::Mad,
        };

        self.cpu_face.set_emotion(cpu_emotion);
    }

    fn hide_press_start_text(&mut self) {
        for obj in [&mut self.press_text_object, &mut self.start_text_object] {
            let oa = obj.get_obj_attr_data();
            oa.set_style(ObjDisplayStyle::NotDisplayed);
        }
    }

    fn get_state(&self) -> TitleScreenState {
        self.state.clone()
    }
}

impl<'a> Screen for TitleScreen<'a> {
    fn update(&mut self) -> Option<ScreenState> {
        self.background_scroller.update();

        self.background_scroller
            .apply_to_background(&self.scrolling_background);

        match self.get_state() {
            TitleScreenState::PressStart(state) => {
                self.update_press_start(state);
                None
            }
            TitleScreenState::Menu(state) => {
                self.update_menu(state);
                None
            }
            TitleScreenState::GameTransition(state) => self.update_transition(state),
        }
    }
}
