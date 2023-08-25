use crate::graphics::sprite::{
    AnimationController, LoadedAnimation, LoadedObjectEntry, LoadedSprite, BOARD_SLOT_SPRITE,
    RED_TOKEN_FRAME_0_SPRITE,
};
use crate::system::gba::{GbaKey, GBA};

const BOARD_COLUMNS: u8 = 7;
const BOARD_ROWS: u8 = 6;
const BOARD_SLOTS: usize = (BOARD_COLUMNS * BOARD_ROWS) as usize;

const SCREEN_WIDTH: u16 = 240;
const SCREEN_HEIGHT: u16 = 160;

enum Player {
    Red,
    Yellow,
}

struct PlayerTurnState {
    player: Player,
    cursor_position: u8,
}

enum GameState {
    PlayerTurn(PlayerTurnState),
    TokenDropping,
}

pub struct GameScreen<'a> {
    gba: &'a GBA,
    red_token_animation_controller: AnimationController<'a, 4>,
    yellow_token_animation_controller: AnimationController<'a, 4>,
    _board_slot_objects: [LoadedObjectEntry<'a>; BOARD_SLOTS],
    game_state: GameState,
}

impl<'a> GameScreen<'a> {
    pub fn new(
        gba: &'a GBA,
        red_token_animation: &'a LoadedAnimation<4>,
        yellow_token_animation: &'a LoadedAnimation<4>,
        board_slot_sprite: &'a LoadedSprite<'a>,
    ) -> Self {
        let red_token_animation_controller = red_token_animation.create_controller(gba);
        let yellow_token_animation_controller = yellow_token_animation.create_controller(gba);

        // Create an Object entry for each slot that makes up the board.
        // We need to keep ownership of these in order to keep them in OBJRAM, so store them in an array.
        let _board_slot_objects: [LoadedObjectEntry<'a>; BOARD_SLOTS] =
            create_board(board_slot_sprite, gba);

        let game_state = GameState::PlayerTurn(PlayerTurnState {
            player: Player::Red,
            cursor_position: 0,
        });

        Self {
            gba,
            red_token_animation_controller,
            yellow_token_animation_controller,
            _board_slot_objects,
            game_state,
        }
    }

    pub fn update(&mut self) {
        if let GameState::PlayerTurn(ref mut s) = self.game_state {
            s.update(
                self.gba,
                &mut self.yellow_token_animation_controller,
                &mut self.red_token_animation_controller,
            );
        };
    }
}

impl PlayerTurnState {
    pub fn update<'a>(
        &mut self,
        gba: &GBA,
        yellow_token_animation_controller: &mut AnimationController<'a, 4>,
        red_token_animation_controller: &mut AnimationController<'a, 4>,
    ) {
        let anim_controller = match self.player {
            Player::Yellow => yellow_token_animation_controller,
            Player::Red => red_token_animation_controller,
        };

        if gba.key_was_pressed(GbaKey::LEFT) {
            self.move_left();
        } else if gba.key_was_pressed(GbaKey::RIGHT) {
            self.move_right();
        }

        let oa = anim_controller.get_obj_attr_entry().get_obj_attr_data();

        let (start_x, start_y) = board_top_left_corner();
        let token_height: u16 = RED_TOKEN_FRAME_0_SPRITE.height().try_into().unwrap();
        let token_width: u16 = RED_TOKEN_FRAME_0_SPRITE.width().try_into().unwrap();
        let board_slot_width: u16 = BOARD_SLOT_SPRITE.width().try_into().unwrap();
        let padding = (board_slot_width - token_width) / 2;

        let xpos = start_x + (self.cursor_position as u16) * board_slot_width + padding;

        let ypos = start_y / 2 - token_height / 2;

        oa.1 = oa.1.with_x(xpos);
        oa.0 = oa.0.with_y(ypos);

        anim_controller.tick();
    }

    fn move_left(&mut self) {
        if self.cursor_position == 0 {
            self.cursor_position = BOARD_COLUMNS;
        }

        self.cursor_position -= 1;
    }

    fn move_right(&mut self) {
        self.cursor_position = (self.cursor_position + 1) % BOARD_COLUMNS;
    }
}

fn board_top_left_corner() -> (u16, u16) {
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

fn create_board<'a>(
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
