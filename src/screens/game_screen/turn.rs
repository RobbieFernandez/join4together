use crate::{graphics::sprite::AnimationController, system::gba::GBA};

use super::{game_board, Player};

pub enum TurnOutcome {
    Victory,
    NextTurn,
    Continue,
}

pub trait Turn {
    fn update<'a>(
        &mut self,
        gba: &GBA,
        yellow_token_animation_controller: &mut AnimationController<'a, 4>,
        red_token_animation_controller: &mut AnimationController<'a, 4>,
        game_board: &mut game_board::GameBoard,
    ) -> TurnOutcome;

    fn get_player(&self) -> Player;
}
