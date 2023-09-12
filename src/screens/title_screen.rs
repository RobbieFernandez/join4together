use gba::prelude::ObjDisplayStyle;

use crate::{
    graphics::{
        background::{LoadedBackground, TITLE_SCREEN_BACKGROUND},
        sprite::{AnimationController, LoadedAnimation, LoadedObjectEntry, LoadedSprite},
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
const MENU_TEXT_HORIZ_MARGIN: u16 = 8;
const CURSOR_X_OFFSET: u16 = 10;

const BLINK_TIME_ON: u32 = 40;
const BLINK_TIME_OFF: u32 = 20;

const CPU_HEAD_POS: (u16, u16) = (133, 49);
const GAME_TRANSITION_TIME: u16 = 40;

#[derive(Clone)]
enum BlinkState {
    On(u32),
    Off(u32),
}

#[derive(Clone, Debug)]
enum MenuEntry {
    VsCpu,
    VsPlayer,
}

#[derive(Clone)]
struct PressStartState {
    blink_state: BlinkState,
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
    _background: LoadedBackground<'a>,
    state: TitleScreenState,
    cpu_face: CpuFace<'a>,
}

impl MenuEntry {
    fn next(&self) -> Self {
        match self {
            Self::VsCpu => Self::VsPlayer,
            Self::VsPlayer => Self::VsCpu,
        }
    }
}

impl<'a> TitleScreen<'a> {
    pub fn new(
        gba: &'a GBA,
        press_text_sprite: &'a LoadedSprite<'a>,
        start_text_sprite: &'a LoadedSprite<'a>,
        vs_cpu_text_sprite: &'a LoadedSprite<'a>,
        vs_player_text_sprite: &'a LoadedSprite<'a>,
        cursor_animation: &'a LoadedAnimation<'a, 5>,
        cpu_sprites: &'a CpuSprites<'a>,
    ) -> Self {
        let background = TITLE_SCREEN_BACKGROUND.load(gba);
        let mut press_text_object = press_text_sprite.create_obj_attr_entry(gba);

        let press_oa = press_text_object.get_obj_attr_data();
        press_oa.set_x(85);
        press_oa.set_y(MENU_TEXT_Y);
        press_text_object.commit_to_memory();

        let mut start_text_object = start_text_sprite.create_obj_attr_entry(gba);

        let start_oa = start_text_object.get_obj_attr_data();
        start_oa.set_x(120);
        start_oa.set_y(MENU_TEXT_Y);
        start_text_object.commit_to_memory();

        let mut vs_cpu_text_object = vs_cpu_text_sprite.create_obj_attr_entry(gba);
        let vs_cpu_sprite_width: u16 = vs_cpu_text_sprite.sprite().width().try_into().unwrap();

        let vs_cpu_oa = vs_cpu_text_object.get_obj_attr_data();
        vs_cpu_oa.set_style(ObjDisplayStyle::NotDisplayed);
        vs_cpu_oa.set_x(SCREEN_WIDTH / 4 - vs_cpu_sprite_width / 2 + MENU_TEXT_HORIZ_MARGIN);
        vs_cpu_oa.set_y(MENU_TEXT_Y);
        vs_cpu_text_object.commit_to_memory();

        let mut vs_player_text_object = vs_player_text_sprite.create_obj_attr_entry(gba);
        let vs_player_sprite_width: u16 =
            vs_player_text_sprite.sprite().width().try_into().unwrap();
        let vs_player_oa = vs_player_text_object.get_obj_attr_data();
        vs_player_oa.set_style(ObjDisplayStyle::NotDisplayed);
        vs_player_oa.set_x(
            SCREEN_WIDTH / 4 - vs_player_sprite_width / 2 + SCREEN_WIDTH / 2
                - MENU_TEXT_HORIZ_MARGIN,
        );
        vs_player_oa.set_y(MENU_TEXT_Y);

        vs_player_text_object.commit_to_memory();

        let mut cursor_animation_controller = cursor_animation.create_controller(gba);
        cursor_animation_controller.set_hidden();
        let cursor_obj = cursor_animation_controller.get_obj_attr_entry();
        let cursor_oa = cursor_obj.get_obj_attr_data();
        cursor_oa.set_y(MENU_TEXT_Y + 2);

        cursor_animation_controller
            .get_obj_attr_entry()
            .commit_to_memory();

        let state = TitleScreenState::PressStart(PressStartState {
            blink_state: BlinkState::On(0),
        });

        let mut cpu_face = CpuFace::new(gba, cpu_sprites);
        cpu_face.set_x(CPU_HEAD_POS.0);
        cpu_face.set_y(CPU_HEAD_POS.1);

        Self {
            gba,
            press_text_object,
            start_text_object,
            state,
            vs_cpu_text_object,
            vs_player_text_object,
            cursor_animation_controller,
            cpu_face,
            _background: background,
        }
    }

    fn update_press_start(&mut self, mut press_start_state: PressStartState) {
        press_start_state.blink_state = match press_start_state.blink_state {
            BlinkState::On(t) => {
                if t >= BLINK_TIME_ON {
                    self.hide_press_start_text();
                    BlinkState::Off(0)
                } else {
                    BlinkState::On(t + 1)
                }
            }
            BlinkState::Off(t) => {
                if t >= BLINK_TIME_OFF {
                    self.show_press_start_text();
                    BlinkState::On(0)
                } else {
                    BlinkState::Off(t + 1)
                }
            }
        };

        self.state = TitleScreenState::PressStart(press_start_state);

        if self.gba.key_was_pressed(GbaKey::START) {
            self.enter_menu();
        }
    }

    fn update_menu(&mut self, mut menu_state: MenuState) {
        if self.gba.key_was_pressed(GbaKey::LEFT) || self.gba.key_was_pressed(GbaKey::RIGHT) {
            menu_state.cursor_position = menu_state.cursor_position.next();
        };

        self.update_cursor_object(&menu_state);
        self.update_cpu_expression(&menu_state);

        if self.gba.key_was_pressed(GbaKey::START) || self.gba.key_was_pressed(GbaKey::A) {
            self.enter_transition(menu_state.cursor_position);
        } else {
            self.state = TitleScreenState::Menu(menu_state);
        }
    }

    fn update_transition(&mut self, mut transition_state: TransitionState) -> Option<ScreenState> {
        transition_state.timer -= 1;

        if transition_state.timer == 0 {
            match transition_state.game_mode {
                MenuEntry::VsCpu => Some(ScreenState::VsCpuScreen),
                MenuEntry::VsPlayer => Some(ScreenState::VsPlayerScreen),
            }
        } else {
            self.state = TitleScreenState::GameTransition(transition_state);
            None
        }
    }

    fn enter_menu(&mut self) {
        self.hide_press_start_text();

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
            obj.commit_to_memory();
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
            obj.commit_to_memory();
        }
    }

    fn show_press_start_text(&mut self) {
        for obj in [&mut self.press_text_object, &mut self.start_text_object] {
            let oa = obj.get_obj_attr_data();
            oa.set_style(ObjDisplayStyle::Normal);
            obj.commit_to_memory();
        }
    }

    fn get_state(&self) -> TitleScreenState {
        self.state.clone()
    }
}

impl<'a> Screen for TitleScreen<'a> {
    fn update(&mut self) -> Option<ScreenState> {
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
