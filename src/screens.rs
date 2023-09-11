use crate::system::gba::GBA;

pub mod game_screen;
pub mod title_screen;

pub enum ScreenState {
    TitleScreen,
    GameScreen,
}

impl ScreenState {
    pub fn game_loop(&self, gba: &GBA) -> Self {
        match self {
            ScreenState::TitleScreen => title_screen::game_loop(gba),
            ScreenState::GameScreen => game_screen::game_loop(gba),
        }
    }
}
