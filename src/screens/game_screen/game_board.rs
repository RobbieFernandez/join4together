use super::Player;

use crate::system::{
    constants::{BOARD_COLUMNS, BOARD_ROWS, BOARD_SLOTS, SCREEN_HEIGHT, SCREEN_WIDTH},
    gba::GBA,
};

use crate::graphics::sprite::{
    LoadedObjectEntry, LoadedSprite, BOARD_SLOT_SPRITE, RED_TOKEN_FRAME_0_SPRITE,
};

pub enum Direction {
    North,
    East,
    South,
    West,
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
}

pub static DIRECTIONS: [Direction; 8] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
    Direction::NorthEast,
    Direction::SouthEast,
    Direction::SouthWest,
    Direction::NorthWest,
];

pub struct GameBoard<'a> {
    matrix: [Option<Player>; BOARD_SLOTS],
    gba: &'a GBA,
    red_token_sprite: &'a LoadedSprite<'a>,
    yellow_token_sprite: &'a LoadedSprite<'a>,
    token_objects: [Option<LoadedObjectEntry<'a>>; BOARD_SLOTS],
}

impl<'a> GameBoard<'a> {
    pub fn new(
        gba: &'a GBA,
        red_token_sprite: &'a LoadedSprite,
        yellow_token_sprite: &'a LoadedSprite,
    ) -> Self {
        let matrix: [Option<Player>; BOARD_SLOTS] = core::array::from_fn(|_| None);
        let token_objects: [Option<LoadedObjectEntry<'a>>; BOARD_SLOTS] =
            core::array::from_fn(|_| None);

        Self {
            matrix,
            gba,
            red_token_sprite,
            yellow_token_sprite,
            token_objects,
        }
    }

    pub fn drop_token(&mut self, column_number: usize, player: Player) -> Option<(usize, usize)> {
        let num_columns: usize = BOARD_COLUMNS.try_into().unwrap();
        assert!(column_number < num_columns);

        let row_index = self.get_next_free_row(column_number);

        let num_rows: usize = BOARD_ROWS.try_into().unwrap();
        let column_start = column_number * num_rows;

        let (start_x, _) = board_top_left_corner();
        let token_width: u16 = RED_TOKEN_FRAME_0_SPRITE.width().try_into().unwrap();
        let board_slot_width: u16 = BOARD_SLOT_SPRITE.width().try_into().unwrap();
        let xpadding = (board_slot_width - token_width) / 2;

        let board_slot_height: u16 = BOARD_SLOT_SPRITE.height().try_into().unwrap();

        if let Some(i) = row_index {
            let cell_index = column_start + i;
            let cell = &mut self.matrix[cell_index];
            let obj = &mut self.token_objects[cell_index];

            // Mark this cell as occupied by the player.
            cell.replace(player);

            // Add an obj entry to draw this player's token here.
            let sprite = match player {
                Player::Red => self.red_token_sprite,
                Player::Yellow => self.yellow_token_sprite,
            };

            let mut loaded_token_obj = sprite.create_obj_attr_entry(self.gba);
            let obj_attr = loaded_token_obj.get_obj_attr_data();

            let xpos = start_x + (column_number as u16) * board_slot_width + xpadding;

            let ypos = SCREEN_HEIGHT - (i as u16 + 1) * board_slot_height;

            obj_attr.0 = obj_attr.0.with_y(ypos);
            obj_attr.1 = obj_attr.1.with_x(xpos);

            loaded_token_obj.commit_to_memory();

            obj.replace(loaded_token_obj);

            Some((column_number, i))
        } else {
            None
        }
    }

    pub fn is_winning_token(&self, column: usize, row: usize, player: Player) -> bool {
        for direction in &DIRECTIONS {
            let coords = self.move_index_in_direction(column, row, direction);

            if let Some((next_col, next_row)) = coords {
                if self.is_connected(direction, player, next_col, next_row, 1) {
                    return true;
                }
            }
        }

        false
    }

    pub fn player_can_win(&self, column: usize, player: Player) -> bool {
        let row = self.get_next_free_row(column);

        match row {
            Some(row) => self.is_winning_token(column, row, player),
            None => false,
        }
    }

    pub fn get_board_after_move(&self, column: usize, player: Player) -> Self {
        // matrix: [Option<Player>; BOARD_SLOTS],
        // gba: &'a GBA,
        // red_token_sprite: &'a LoadedSprite<'a>,
        // yellow_token_sprite: &'a LoadedSprite<'a>,
        // token_objects: [Option<LoadedObjectEntry<'a>>; BOARD_SLOTS],

        let mut new_matrix = self.matrix.clone();

        let row = self.get_next_free_row(column);

        if let Some(row) = row {
            let num_rows: usize = BOARD_ROWS.try_into().unwrap();
            let column_start = column * num_rows;
            let cell_index = column_start + row;
            new_matrix[cell_index] = Some(player);
        }

        let cloned_board = Self {
            matrix: new_matrix,
            gba: self.gba,
            red_token_sprite: self.red_token_sprite,
            yellow_token_sprite: self.yellow_token_sprite,
            token_objects: core::array::from_fn(|_| None),
        };

        cloned_board
    }

    pub fn get_neighbour(
        &self,
        column: usize,
        row: usize,
        direction: &Direction,
    ) -> Option<Player> {
        let coords = self.move_index_in_direction(column, row, &direction);

        if let Some((col, row)) = coords {
            self.check_token(col, row)
        } else {
            None
        }
    }

    pub fn get_next_free_row(&self, column_number: usize) -> Option<usize> {
        let num_rows: usize = BOARD_ROWS.try_into().unwrap();
        let column_start = column_number * num_rows;
        (0..num_rows).find(|i| (&self.matrix[column_start + i]).is_none())
    }

    fn move_index_in_direction(
        &self,
        current_column: usize,
        current_row: usize,
        direction: &Direction,
    ) -> Option<(usize, usize)> {
        let num_rows: usize = BOARD_ROWS.try_into().unwrap();
        let num_cols: usize = BOARD_COLUMNS.try_into().unwrap();

        let is_bottom = current_row == 0;
        let is_top = current_row == num_rows - 1;
        let is_left = current_column == 0;
        let is_right = current_column == num_cols - 1;

        match direction {
            Direction::North => {
                if is_top {
                    None
                } else {
                    Some((current_column, current_row + 1))
                }
            }
            Direction::NorthEast => {
                if is_top || is_right {
                    None
                } else {
                    Some((current_column + 1, current_row + 1))
                }
            }
            Direction::East => {
                if is_right {
                    None
                } else {
                    Some((current_column + 1, current_row))
                }
            }
            Direction::SouthEast => {
                if is_bottom || is_right {
                    None
                } else {
                    Some((current_column + 1, current_row - 1))
                }
            }
            Direction::South => {
                if is_bottom {
                    None
                } else {
                    Some((current_column, current_row - 1))
                }
            }
            Direction::SouthWest => {
                if is_bottom || is_left {
                    None
                } else {
                    Some((current_column - 1, current_row - 1))
                }
            }
            Direction::West => {
                if is_left {
                    None
                } else {
                    Some((current_column - 1, current_row))
                }
            }
            Direction::NorthWest => {
                if is_left || is_top {
                    None
                } else {
                    Some((current_column - 1, current_row + 1))
                }
            }
        }
    }

    fn check_token(&self, column: usize, row: usize) -> Option<Player> {
        let num_rows: usize = BOARD_ROWS.try_into().unwrap();
        let index = column * num_rows + row;

        if index < self.matrix.len() {
            self.matrix[index]
        } else {
            None
        }
    }

    fn is_connected(
        &self,
        direction: &Direction,
        player: Player,
        column: usize,
        row: usize,
        count: u32,
    ) -> bool {
        if self.check_token(column, row) == Some(player) {
            let count = count + 1;

            if count == 4 {
                true
            } else {
                let coords = self.move_index_in_direction(column, row, &direction);

                if let Some((next_column, next_row)) = coords {
                    self.is_connected(direction, player, next_column, next_row, count)
                } else {
                    false
                }
            }
        } else {
            false
        }
    }
}

pub fn create_board_object_entries<'a>(
    board_slot_sprite: &'a LoadedSprite,
    gba: &'a GBA,
) -> [LoadedObjectEntry<'a>; BOARD_SLOTS] {
    let (start_x, start_y) = board_top_left_corner();

    let sprite = board_slot_sprite.sprite();
    let board_slot_width: u16 = sprite.width().try_into().unwrap();
    let board_slot_height: u16 = sprite.height().try_into().unwrap();

    let columns: u16 = BOARD_COLUMNS.into();

    core::array::from_fn(|i| {
        let mut obj_entry = board_slot_sprite.create_obj_attr_entry(gba);

        let i: u16 = i.try_into().unwrap();
        let col: u16 = i % columns;
        let row: u16 = i / columns;

        let obj_attrs = obj_entry.get_obj_attr_data();
        obj_attrs.0 = obj_attrs.0.with_y(start_y + row * board_slot_height);
        obj_attrs.1 = obj_attrs.1.with_x(start_x + col * board_slot_width);
        obj_attrs.2 = obj_attrs.2.with_priority(0);

        obj_entry.commit_to_memory();

        obj_entry
    })
}

pub fn board_top_left_corner() -> (u16, u16) {
    let sprite = &BOARD_SLOT_SPRITE;

    let board_slot_width: u16 = sprite.width().try_into().unwrap();
    let board_slot_height: u16 = sprite.height().try_into().unwrap();

    let columns: u16 = BOARD_COLUMNS.into();
    let rows: u16 = BOARD_ROWS.into();

    let board_width_pixels: u16 = board_slot_width * columns;
    let board_height_pixels: u16 = board_slot_height * rows;

    let start_y: u16 = SCREEN_HEIGHT - board_height_pixels;
    let start_x: u16 = (SCREEN_WIDTH - board_width_pixels) / 2;

    (start_x, start_y)
}

impl Direction {
    pub fn row_movement(&self) -> i32 {
        match self {
            Direction::North => 1,
            Direction::East => 0,
            Direction::South => -1,
            Direction::West => 0,
            Direction::NorthEast => 1,
            Direction::SouthEast => -1,
            Direction::SouthWest => -1,
            Direction::NorthWest => 1,
        }
    }

    pub fn col_movement(&self) -> i32 {
        match self {
            Direction::North => 0,
            Direction::East => 1,
            Direction::South => 0,
            Direction::West => -1,
            Direction::NorthEast => 1,
            Direction::SouthEast => 1,
            Direction::SouthWest => -1,
            Direction::NorthWest => -1,
        }
    }
}
