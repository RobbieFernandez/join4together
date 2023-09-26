use bitfrob::u8x2;
use gba::{
    prelude::{BLDALPHA, BLDCNT},
    video::BlendControl,
};

// Just wraps the gba crate's BlendControl struct but makes it cancel all blending once it's dropped.
pub struct BlendController {}

impl BlendController {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, blend_control: BlendControl, blend_weights: u8x2) {
        BLDCNT.write(blend_control);
        BLDALPHA.write(blend_weights);
    }
}

impl Drop for BlendController {
    fn drop(&mut self) {
        BLDCNT.write(BlendControl::new());
        BLDALPHA.write([0u8, 0u8].into());
    }
}
