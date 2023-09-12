use gba::prelude::VBlankIntrWait;

use crate::{
    graphics::sprite::{
        BOARD_SLOT_SPRITE, PRESS_TEXT_SPRITE, RED_TOKEN_ANIMATION, START_TEXT_SPRITE,
        YELLOW_TOKEN_ANIMATION,
    },
    system::gba::GBA,
};

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
                let press_text_sprite = PRESS_TEXT_SPRITE.load(gba);
                let start_text_sprite = START_TEXT_SPRITE.load(gba);

                let mut screen =
                    title_screen::TitleScreen::new(gba, &press_text_sprite, &start_text_sprite);

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
        let yellow_token_animation = YELLOW_TOKEN_ANIMATION.load(gba);
        let red_token_animation = RED_TOKEN_ANIMATION.load(gba);
        let board_slot_sprite = BOARD_SLOT_SPRITE.load(gba);

        let mut screen = game_screen::GameScreen::new(
            gba,
            &red_token_animation,
            &yellow_token_animation,
            &board_slot_sprite,
            red_agent,
            yellow_agent,
        );

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
