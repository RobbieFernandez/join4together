use super::TokenColor;

use crate::system::{
    constants::{BOARD_COLUMNS, BOARD_ROWS, BOARD_SLOTS, SCREEN_HEIGHT, SCREEN_WIDTH},
    gba::GBA,
};

use crate::graphics::sprite::{
    LoadedObjectEntry, LoadedSprite, BOARD_SLOT_SPRITE, RED_TOKEN_FRAME_0_SPRITE,
};

pub type WinningPositions = [usize; 4];

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
    matrix: [Option<TokenColor>; BOARD_SLOTS],
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
        let matrix: [Option<TokenColor>; BOARD_SLOTS] = core::array::from_fn(|_| None);
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

    pub fn get_token_ypos_for_row(&self, row_number: usize) -> u16 {
        let board_slot_height: u16 = BOARD_SLOT_SPRITE.height().try_into().unwrap();

        let row_number: u16 = row_number.try_into().unwrap();
        SCREEN_HEIGHT - (row_number + 1) * board_slot_height
    }

    pub fn set_cell(
        &mut self,
        token_color: TokenColor,
        column_number: usize,
        row_number: usize,
    ) -> usize {
        let num_columns: usize = BOARD_COLUMNS.try_into().unwrap();
        assert!(column_number < num_columns);

        let num_rows: usize = BOARD_ROWS.try_into().unwrap();
        let column_start = column_number * num_rows;

        let cell_index = column_start + row_number;
        let cell = &mut self.matrix[cell_index];

        // Mark this cell as occupied by the player.
        cell.replace(token_color);

        // Add an obj entry to draw this player's token here.
        self.add_token_obj(token_color, column_number, row_number)
    }

    pub fn get_winning_token_positions(
        &self,
        column: usize,
        row: usize,
        token_color: TokenColor,
    ) -> Option<WinningPositions> {
        DIRECTIONS
            .iter()
            .map(|direction| self.get_connected_line(direction, token_color, column, row))
            .filter(|l| l.is_some())
            .map(|l| l.unwrap())
            .next()
    }

    pub fn is_winning_token(&self, column: usize, row: usize, token_color: TokenColor) -> bool {
        self.get_winning_token_positions(column, row, token_color)
            .is_some()
    }

    pub fn player_can_win(&self, column: usize, token_color: TokenColor) -> bool {
        let row = self.get_next_free_row(column);

        match row {
            Some(row) => self
                .get_winning_token_positions(column, row, token_color)
                .is_some(),
            None => false,
        }
    }

    pub fn get_board_after_move(&self, column: usize, token_color: TokenColor) -> Self {
        let mut new_matrix = self.matrix;

        let row = self.get_next_free_row(column);

        if let Some(row) = row {
            let num_rows: usize = BOARD_ROWS.try_into().unwrap();
            let column_start = column * num_rows;
            let cell_index = column_start + row;
            new_matrix[cell_index] = Some(token_color);
        }

        Self {
            matrix: new_matrix,
            gba: self.gba,
            red_token_sprite: self.red_token_sprite,
            yellow_token_sprite: self.yellow_token_sprite,
            token_objects: core::array::from_fn(|_| None),
        }
    }

    pub fn get_neighbour(
        &self,
        column: usize,
        row: usize,
        direction: &Direction,
    ) -> Option<TokenColor> {
        let coords = self.move_index_in_direction(column, row, direction);

        if let Some((col, row)) = coords {
            self.check_token(col, row)
        } else {
            None
        }
    }

    pub fn get_next_free_row(&self, column_number: usize) -> Option<usize> {
        let num_rows: usize = BOARD_ROWS.try_into().unwrap();
        let column_start = column_number * num_rows;
        (0..num_rows).find(|i| self.matrix[column_start + i].is_none())
    }

    pub fn get_token_obj_entry_mut(&mut self, index: usize) -> &mut Option<LoadedObjectEntry<'a>> {
        &mut self.token_objects[index]
    }

    pub fn is_full(&self) -> bool {
        self.matrix.iter().all(|i| i.is_some())
    }

    fn add_token_obj(&mut self, token_color: TokenColor, col: usize, row: usize) -> usize {
        let cell_index = self.calculate_index(col, row);

        let x_pos = get_token_x_position(col);
        let y_pos = get_token_y_position();

        let sprite = match token_color {
            TokenColor::Red => self.red_token_sprite,
            TokenColor::Yellow => self.yellow_token_sprite,
        };

        let obj_slot = &mut self.token_objects[cell_index];
        let mut obj = sprite.create_obj_attr_entry(self.gba);

        let attr = obj.get_obj_attr_data();
        attr.1 = attr.1.with_x(x_pos);
        attr.0 = attr.0.with_y(y_pos);

        obj_slot.replace(obj);

        cell_index
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

    fn check_token(&self, column: usize, row: usize) -> Option<TokenColor> {
        let num_rows: usize = BOARD_ROWS.try_into().unwrap();
        let index = column * num_rows + row;

        if index < self.matrix.len() {
            self.matrix[index]
        } else {
            None
        }
    }

    pub fn get_connected_positions(
        &self,
        starting_column: usize,
        starting_row: usize,
        direction: &Direction,
        token_color: TokenColor,
    ) -> ([usize; 3], usize) {
        let mut current_col = starting_column;
        let mut current_row = starting_row;

        let mut length: usize = 0;
        let mut token_positions = [0; 3];

        loop {
            let new_coords = self.move_index_in_direction(current_col, current_row, direction);

            if let Some((new_col, new_row)) = new_coords {
                if self.check_token(new_col, new_row) == Some(token_color) {
                    token_positions[length] = self.calculate_index(new_col, new_row);
                    length += 1;

                    current_col = new_col;
                    current_row = new_row;

                    if length == 3 {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        (token_positions, length)
    }

    fn get_connected_line(
        &self,
        direction: &Direction,
        token_color: TokenColor,
        column: usize,
        row: usize,
    ) -> Option<WinningPositions> {
        // An array that will hold 4 co-ordinates. Initially it will just be the starting token,
        // But we will add in adjacent tokens until we have filled it.
        let mut connected_positions = [(0); 4];
        let mut connected_length = 1;
        connected_positions[0] = self.calculate_index(column, row);

        let (positive_connections, positive_distance) =
            self.get_connected_positions(column, row, direction, token_color);

        for i in 0..positive_distance {
            connected_positions[i + 1] = positive_connections[i];
        }

        connected_length += positive_distance;

        if connected_length == 4 {
            Some(connected_positions)
        } else {
            let opposite_direction = direction.opposite();

            let (negative_connections, negative_distance) =
                self.get_connected_positions(column, row, &opposite_direction, token_color);

            if connected_length + negative_distance < 4 {
                // Still not enough connected tokens, so return early
                None
            } else {
                // There are enough new tokens to make this a connected line. Add these new elements to the array and return it
                let remaining_tokens = 4 - connected_length;
                for i in 0..remaining_tokens {
                    connected_positions[connected_length + i] = negative_connections[i];
                }
                Some(connected_positions)
            }
        }
    }

    fn calculate_index(&self, col: usize, row: usize) -> usize {
        let num_rows: usize = BOARD_ROWS.try_into().unwrap();
        let column_start = col * num_rows;

        column_start + row
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
    pub fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::NorthEast => Direction::SouthWest,
            Direction::East => Direction::West,
            Direction::SouthEast => Direction::NorthWest,
            Direction::South => Direction::North,
            Direction::SouthWest => Direction::NorthEast,
            Direction::West => Direction::East,
            Direction::NorthWest => Direction::SouthEast,
        }
    }
}

pub fn get_token_y_position() -> u16 {
    let (_, start_y) = board_top_left_corner();
    let token_height: u16 = RED_TOKEN_FRAME_0_SPRITE.height().try_into().unwrap();

    start_y / 2 - token_height / 2
}

pub fn get_token_x_position(column_number: usize) -> u16 {
    let (start_x, _) = board_top_left_corner();
    let token_width: u16 = RED_TOKEN_FRAME_0_SPRITE.width().try_into().unwrap();
    let board_slot_width: u16 = BOARD_SLOT_SPRITE.width().try_into().unwrap();
    let padding = (board_slot_width - token_width) / 2;

    let column_number: u16 = column_number.try_into().unwrap();
    start_x + column_number * board_slot_width + padding
}
