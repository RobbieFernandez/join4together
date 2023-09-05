use super::memory::block::MemoryBlockManager;
use super::memory::series::MemorySeriesManager;
use gba::prelude::*;
use voladdress::Safe;

pub use super::memory::block::ClaimedVolRegion;
pub use super::memory::series::ClaimedVolAddress;

pub type PaletteMemory = MemoryBlockManager<Color, Safe, Safe, 256>;
pub type ObjAttrMemory = MemorySeriesManager<ObjAttr, Safe, Safe, 128, 8>;
pub type ObjTileMemory = MemoryBlockManager<Tile4, Safe, Safe, 1024>;
pub type BgTilesetMemory = MemoryBlockManager<Tile4, Safe, Safe, 512>;

static GBA_TAKEN: GbaCell<bool> = GbaCell::new(false);
static PREV_INPUT_STATE: GbaCell<KeyInput> = GbaCell::new(KeyInput::new());
static CURRENT_INPUT_STATE: GbaCell<KeyInput> = GbaCell::new(KeyInput::new());

pub struct GBA {
    pub obj_palette_memory: PaletteMemory,
    pub bg_palette_memory: PaletteMemory,
    pub obj_attr_memory: ObjAttrMemory,
    pub obj_tile_memory: ObjTileMemory,
    // pub bg_tileset_memory: BgTilesetMemory,
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

extern "C" fn update_input(irq: IrqBits) {
    if irq.vblank() {
        let keystate = KEYINPUT.read();

        PREV_INPUT_STATE.write(CURRENT_INPUT_STATE.read());
        CURRENT_INPUT_STATE.write(keystate);
    }
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
            obj_attr_memory: ObjAttrMemory::new(OBJ_ATTR_ALL),
            obj_tile_memory: ObjTileMemory::new(OBJ_TILES),
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
        IE.write(IrqBits::VBLANK);
        IME.write(true);

        RUST_IRQ_HANDLER.write(Some(update_input));
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
