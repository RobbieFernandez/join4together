use gba::prelude::ObjDisplayStyle;

use crate::{
    graphics::{
        background::{LoadedBackground, TITLE_SCREEN_BACKGROUND},
        sprite::{LoadedObjectEntry, LoadedSprite, PRESS_TEXT_SPRITE, START_TEXT_SPRITE},
    },
    system::gba::{GbaKey, GBA},
};

use super::ScreenState;

const PRESS_START_Y: u16 = 140;

const BLINK_TIME_ON: u32 = 40;
const BLINK_TIME_OFF: u32 = 20;

enum BlinkState {
    On(u32),
    Off(u32),
}

pub struct TitleScreen<'a> {
    gba: &'a GBA,
    press_text_object: LoadedObjectEntry<'a>,
    start_text_object: LoadedObjectEntry<'a>,
    _background: LoadedBackground<'a>,
    blink_state: BlinkState,
}

impl<'a> TitleScreen<'a> {
    pub fn new(
        gba: &'a GBA,
        press_text_sprite: &'a LoadedSprite<'a>,
        start_text_sprite: &'a LoadedSprite<'a>,
    ) -> Self {
        let background = TITLE_SCREEN_BACKGROUND.load(gba);
        let mut press_text_object = press_text_sprite.create_obj_attr_entry(gba);

        let press_oa = press_text_object.get_obj_attr_data();
        press_oa.0 = press_oa.0.with_y(PRESS_START_Y);
        press_oa.1 = press_oa.1.with_x(85);
        press_text_object.commit_to_memory();

        let mut start_text_object = start_text_sprite.create_obj_attr_entry(gba);

        let start_oa = start_text_object.get_obj_attr_data();
        start_oa.0 = start_oa.0.with_y(PRESS_START_Y);
        start_oa.1 = start_oa.1.with_x(120);
        start_text_object.commit_to_memory();

        Self {
            gba,
            press_text_object,
            start_text_object,
            blink_state: BlinkState::On(0),
            _background: background,
        }
    }

    pub fn update(&mut self) -> bool {
        self.update_blinking_text();
        self.gba.key_was_pressed(GbaKey::START)
    }

    pub fn update_blinking_text(&mut self) {
        self.blink_state = match self.blink_state {
            BlinkState::On(t) => {
                if t >= BLINK_TIME_ON {
                    self.hide_text();
                    BlinkState::Off(0)
                } else {
                    BlinkState::On(t + 1)
                }
            }
            BlinkState::Off(t) => {
                if t >= BLINK_TIME_OFF {
                    self.show_text();
                    BlinkState::On(0)
                } else {
                    BlinkState::Off(t + 1)
                }
            }
        };
    }

    fn hide_text(&mut self) {
        for obj in [&mut self.press_text_object, &mut self.start_text_object] {
            let oa = obj.get_obj_attr_data();
            oa.0 = oa.0.with_style(ObjDisplayStyle::NotDisplayed);
            obj.commit_to_memory();
        }
    }

    fn show_text(&mut self) {
        for obj in [&mut self.press_text_object, &mut self.start_text_object] {
            let oa = obj.get_obj_attr_data();
            oa.0 = oa.0.with_style(ObjDisplayStyle::Normal);
            obj.commit_to_memory();
        }
    }
}

pub fn game_loop(gba: &GBA) -> ScreenState {
    let press_text_sprite = PRESS_TEXT_SPRITE.load(gba);
    let start_text_sprite = START_TEXT_SPRITE.load(gba);

    let mut screen = TitleScreen::new(gba, &press_text_sprite, &start_text_sprite);

    loop {
        gba::bios::VBlankIntrWait();
        if screen.update() {
            return ScreenState::GameScreen;
        }
    }
}
