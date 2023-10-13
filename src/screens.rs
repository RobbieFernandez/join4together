use gba::prelude::VBlankIntrWait;

use crate::{audio::mixer, system::gba::GBA};

use self::{
    game_screen::{
        cpu_face::{CpuFace, CpuSprites},
        TokenColor,
    },
    spinner_screen::{SpinnerScreen, SpinnerScreenLoadedData},
};

pub mod game_screen;
pub mod spinner_screen;
pub mod title_screen;

pub enum ScreenState {
    TitleScreen,
    VsCpuScreen(TokenColor),
    VsCpuSpinnerScreen,
    VsPlayerScreen(TokenColor),
    VsPlayerSpinnerScreen,
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
            ScreenState::VsCpuScreen(starting_color) => {
                let cpu_sprites = CpuSprites::new(gba);
                let cpu_face = CpuFace::new(gba, &cpu_sprites);

                let red_agent = game_screen::Agent::new_human_agent();
                let yellow_agent = game_screen::Agent::new_cpu_agent(cpu_face);

                self.exec_game_screen(gba, red_agent, yellow_agent, *starting_color)
            }
            ScreenState::VsPlayerScreen(starting_color) => {
                let red_agent = game_screen::Agent::new_human_agent();
                let yellow_agent = game_screen::Agent::new_human_agent();

                self.exec_game_screen(gba, red_agent, yellow_agent, *starting_color)
            }
            ScreenState::VsCpuSpinnerScreen => {
                self.exec_spinner_screen(gba, spinner_screen::SpinnerMode::VsCpu)
            }
            ScreenState::VsPlayerSpinnerScreen => {
                self.exec_spinner_screen(gba, spinner_screen::SpinnerMode::VsPlayer)
            }
        }
    }

    pub fn exec_spinner_screen(&self, gba: &GBA, mode: spinner_screen::SpinnerMode) -> ScreenState {
        let loaded_data = SpinnerScreenLoadedData::new(gba, &mode);
        let mut screen = SpinnerScreen::new(gba, &loaded_data, mode);
        self.screen_loop(&mut screen)
    }

    pub fn exec_game_screen(
        &self,
        gba: &GBA,
        red_agent: game_screen::Agent,
        yellow_agent: game_screen::Agent,
        starting_color: TokenColor,
    ) -> ScreenState {
        let loaded_data = game_screen::GameScreenLoadedData::new(gba);
        let mut screen = game_screen::GameScreen::new(
            gba,
            &loaded_data,
            red_agent,
            yellow_agent,
            starting_color,
        );

        self.screen_loop(&mut screen)
    }

    fn screen_loop<S: Screen>(&self, screen: &mut S) -> ScreenState {
        loop {
            VBlankIntrWait();
            let next_state = screen.update();
            mixer::fill_next_buffer();

            // Break out of the loop when a transition happens.
            if let Some(state) = next_state {
                return state;
            }
        }
    }
}
