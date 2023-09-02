use crate::{graphics::sprite::AnimationController, system::gba::GBA};

use super::{cpu_face::CpuFace, game_board, Player};

pub trait Turn {
    fn update(
        &mut self,
        gba: &GBA,
        animation_controller: &mut AnimationController<4>,
        game_board: &mut game_board::GameBoard,
        cpu_face: &mut CpuFace,
    ) -> Option<usize>;

    fn get_player(&self) -> Player;
}
