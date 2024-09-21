use core::mem::size_of;

use crate::audio::mixer;
use crate::audio::noise::enable_noise;

use super::irq::init_irq;
use super::memory::block::MemoryBlockManager;
use super::memory::series::MemorySeriesManager;
use super::memory::shadow_oam::ShadowOAM;
use super::memory::strided_grid::MemoryStridedGridManager;
use gba::prelude::*;
use voladdress::Safe;

pub use super::memory::block::ClaimedVolRegion;
pub use super::memory::series::ClaimedVolAddress;
pub use super::memory::shadow_oam::OAMEntry;
pub use super::memory::strided_grid::ClaimedGridFrames;

pub type PaletteMemory = MemoryBlockManager<Color, Safe, Safe, 256>;
pub type ObjAttrMemory = MemorySeriesManager<ObjAttr, Safe, Safe, 128, 8>;
pub type ObjTileMemory = MemoryBlockManager<Tile4, Safe, Safe, 1024>;
pub type CharblockMemory = MemoryBlockManager<Tile4, Safe, Safe, 512>;
pub type ScreenblockMemory =
    MemoryStridedGridManager<TextEntry, Safe, Safe, 32, 32, 32, SCREENBLOCK_INDEX_OFFSET>;
pub type AffineObjectMatrixMemory =
    MemorySeriesManager<i16fx8, Safe, Safe, 32, { size_of::<[u16; 16]>() }>;

static GBA_TAKEN: GbaCell<bool> = GbaCell::new(false);
static PREV_INPUT_STATE: GbaCell<KeyInput> = GbaCell::new(KeyInput::new());
static CURRENT_INPUT_STATE: GbaCell<KeyInput> = GbaCell::new(KeyInput::new());

pub const CHARBLOCK_BASE: u16 = 3;

pub struct GBA {
    pub obj_palette_memory: PaletteMemory,
    pub bg_palette_memory: PaletteMemory,
    pub obj_tile_memory: ObjTileMemory,
    pub charblock_memory: CharblockMemory,
    pub screenblock_memory: ScreenblockMemory,
    pub affine_object_matrix_memory: AffineObjectMatrixMemory,
    pub shadow_oam: ShadowOAM,
}

pub enum GbaKey {
    A,
    B,
    SELECT,
    START,
    UP,
    DOWN,
    LEFT,
    RIGHT,
    L,
    R,
}

pub fn update_input() {
    let keystate = KEYINPUT.read();

    PREV_INPUT_STATE.write(CURRENT_INPUT_STATE.read());
    CURRENT_INPUT_STATE.write(keystate);
}

impl GBA {
    pub fn take() -> Self {
        if GBA_TAKEN.read() {
            panic!("GBA struct can only be taken once.");
        }

        GBA_TAKEN.write(true);

        let mut gba = GBA {
            bg_palette_memory: PaletteMemory::new(BG_PALETTE),
            obj_palette_memory: PaletteMemory::new(OBJ_PALETTE),
            obj_tile_memory: ObjTileMemory::new(OBJ_TILES),
            shadow_oam: ShadowOAM::new(),

            // Screenblocks and charblocks occupy the same region in memory, so we need to make sure we don't
            // try to allocate overlapping addressed for Tilemaps/Tilesets.
            // The memory managers assume that they own the memory they allocate, so this breaks that assumption.
            // As a sort of hack, we will just use charblock 3, so that they won't overlap unless we allocate
            // too many screenblocks (which we don't for this game).
            charblock_memory: CharblockMemory::new(CHARBLOCK3_4BPP),
            screenblock_memory: ScreenblockMemory::new(TEXT_SCREENBLOCKS),

            // The gba crate has separate entries for each of the matrix params.
            // To make this memory still be dynamic, we'll just have param a be managed
            // and then whenver we own param A, we will manually get the corresponding B, C and D entries.
            affine_object_matrix_memory: AffineObjectMatrixMemory::new(AFFINE_PARAM_A),
        };
        gba.init();
        gba
    }

    pub fn input_state(&self) -> KeyInput {
        CURRENT_INPUT_STATE.read()
    }

    pub fn key_was_pressed(&self, key: GbaKey) -> bool {
        read_key(&key, CURRENT_INPUT_STATE.read()) && !read_key(&key, PREV_INPUT_STATE.read())
    }

    pub fn key_was_released(&self, key: GbaKey) -> bool {
        read_key(&key, PREV_INPUT_STATE.read()) && !read_key(&key, CURRENT_INPUT_STATE.read())
    }

    fn set_display_mode(&mut self, display_mode: DisplayControl) {
        DISPCNT.write(display_mode);
    }

    fn init(&mut self) {
        self.hide_all_objects();
        self.set_display_mode(
            DisplayControl::new()
                .with_video_mode(VideoMode::_0)
                .with_obj_vram_1d(true)
                .with_show_obj(true),
        );

        // Set up the VBLANK IRQ
        DISPSTAT.write(DisplayStatus::new().with_irq_vblank(true));

        // We will start TIMER 3 to be used only for seeding RNG
        TIMER3_CONTROL.write(TimerControl::new().with_enabled(true));
        TIMER3_RELOAD.write(0x0000);

        // Turn on the sound chip.
        SOUND_ENABLED.write(SoundEnable::new().with_enabled(true));
        enable_noise();

        mixer::init_mixer();

        init_irq();
    }

    fn hide_all_objects(&mut self) {
        let hidden_obj = ObjAttr0::new().with_style(ObjDisplayStyle::NotDisplayed);

        for addr in OBJ_ATTR0.iter() {
            addr.write(hidden_obj);
        }
    }
}

fn read_key(key: &GbaKey, input_state: KeyInput) -> bool {
    match key {
        GbaKey::A => input_state.a(),
        GbaKey::B => input_state.b(),
        GbaKey::SELECT => input_state.select(),
        GbaKey::START => input_state.start(),
        GbaKey::UP => input_state.up(),
        GbaKey::DOWN => input_state.down(),
        GbaKey::LEFT => input_state.left(),
        GbaKey::RIGHT => input_state.right(),
        GbaKey::L => input_state.l(),
        GbaKey::R => input_state.r(),
    }
}
