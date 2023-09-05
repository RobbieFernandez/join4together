use crate::{
    graphics::background::{LoadedBackground, TITLE_SCREEN_BACKGROUND},
    system::gba::{GbaKey, GBA},
};

pub struct TitleScreen<'a> {
    gba: &'a GBA,
    _background: LoadedBackground<'a>,
}

impl<'a> TitleScreen<'a> {
    pub fn new(gba: &'a GBA) -> Self {
        let background = TITLE_SCREEN_BACKGROUND.load(gba);
        Self {
            gba,
            _background: background,
        }
    }

    pub fn update(&mut self) -> bool {
        self.gba.key_was_pressed(GbaKey::START)
    }
}
