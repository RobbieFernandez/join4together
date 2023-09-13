use gba::prelude::VBlankIntrWait;

use crate::system::gba::GBA;

use self::game_screen::cpu_face::{CpuFace, CpuSprites};

pub mod game_screen;
pub mod title_screen;

pub enum ScreenState {
    TitleScreen,
    VsCpuScreen,
    VsPlayerScreen,
}

pub trait Screen {
    fn update(&mut self) -> Option<ScreenState>;
}

impl ScreenState {
    /// Run the game loop for the currently active screen.
    /// If a screen transition occurs, then the next screen ScreenState will be returned.
    pub fn exec_screen(&self, gba: &GBA) -> Self {
        // Construct the required screen and run its loop until it transitions.
        match self {
            ScreenState::TitleScreen => {
                let loaded_data = title_screen::TitleScreenLoadedData::new(gba);
                let mut screen = title_screen::TitleScreen::new(gba, &loaded_data);

                self.screen_loop(&mut screen)
            }
            ScreenState::VsCpuScreen => {
                let cpu_sprites = CpuSprites::new(gba);
                let cpu_face = CpuFace::new(gba, &cpu_sprites);

                let red_agent = game_screen::Agent::new_human_agent();
                let yellow_agent = game_screen::Agent::new_cpu_agent(cpu_face);

                self.exec_game_screen(gba, red_agent, yellow_agent)
            }
            ScreenState::VsPlayerScreen => {
                let red_agent = game_screen::Agent::new_human_agent();
                let yellow_agent = game_screen::Agent::new_human_agent();

                self.exec_game_screen(gba, red_agent, yellow_agent)
            }
        }
    }

    pub fn exec_game_screen(
        &self,
        gba: &GBA,
        red_agent: game_screen::Agent,
        yellow_agent: game_screen::Agent,
    ) -> ScreenState {
        let loaded_data = game_screen::GameScreenLoadedData::new(gba);
        let mut screen = game_screen::GameScreen::new(gba, &loaded_data, red_agent, yellow_agent);

        self.screen_loop(&mut screen)
    }

    fn screen_loop<S: Screen>(&self, screen: &mut S) -> ScreenState {
        loop {
            VBlankIntrWait();

            // Break out of the loop when a transition happens.
            if let Some(state) = screen.update() {
                return state;
            }
        }
    }
}
