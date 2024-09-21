use core::cmp::{max, min};

use super::game_board::{self, get_token_x_position};
use crate::{graphics::sprite::AnimationController, system::constants::BOARD_COLUMNS};

const CURSOR_MOVEMENT_SPEED: u16 = 5;
const CURSOR_MOVEMENT_SPEED_FAST: u16 = 20;

#[derive(Clone)]
pub struct Cursor {
    column: usize,
    x_position: u16,
    target_x_position: u16,
    moving: bool,
    speed: u16,
}

impl Cursor {
    pub fn new() -> Self {
        let column: usize = 0;
        let x_position = get_token_x_position(column);
        Self {
            column,
            x_position,
            target_x_position: x_position,
            moving: false,
            speed: CURSOR_MOVEMENT_SPEED,
        }
    }

    pub fn draw(&self, animation_controller: &mut AnimationController<4>) {
        animation_controller.set_visible();

        let oa = animation_controller
            .get_obj_attr_entry()
            .get_obj_attr_data();

        let ypos = game_board::get_token_y_position();

        oa.1 = oa.1.with_x(self.x_position);
        oa.0 = oa.0.with_y(ypos);

        animation_controller.tick();
    }

    pub fn update_movement(&mut self) {
        if self.moving {
            if self.x_position > self.target_x_position {
                self.x_position = max(self.x_position - self.speed, self.target_x_position)
            } else {
                self.x_position = min(self.x_position + self.speed, self.target_x_position)
            }

            self.moving = self.x_position != self.target_x_position;

            // Reset the speed once the movement is done.
            if !self.moving {
                self.speed = CURSOR_MOVEMENT_SPEED;
            }
        }
    }

    pub fn move_to_column(&mut self, target_column: usize) {
        self.moving = true;
        self.target_x_position = game_board::get_token_x_position(target_column);
        self.column = target_column;
    }

    pub fn move_left(&mut self) {
        let target_col: usize = if self.column == 0 {
            let num_col: usize = BOARD_COLUMNS.into();
            self.speed = CURSOR_MOVEMENT_SPEED_FAST;
            num_col - 1
        } else {
            self.column - 1
        };

        self.move_to_column(target_col);
    }

    pub fn move_right(&mut self) {
        let num_columns: usize = BOARD_COLUMNS.into();
        let target_col: usize = (self.column + 1) % num_columns;

        if target_col < self.column {
            self.speed = CURSOR_MOVEMENT_SPEED_FAST;
        }

        self.move_to_column(target_col);
    }

    pub fn is_moving(&self) -> bool {
        self.moving
    }

    pub fn get_column(&self) -> usize {
        self.column
    }
}
