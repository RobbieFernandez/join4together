use gba::prelude::{i16fx8, AFFINE_PARAM_B, AFFINE_PARAM_C, AFFINE_PARAM_D};
use voladdress::Safe;

use crate::system::gba::{ClaimedVolAddress, GBA};

pub struct AffineMatrix<'a> {
    pub param_a: i16fx8,
    pub param_b: i16fx8,
    pub param_c: i16fx8,
    pub param_d: i16fx8,
    memory: ClaimedVolAddress<'a, i16fx8, Safe, Safe, 32>,
}

impl<'a> AffineMatrix<'a> {
    pub fn new(gba: &'a GBA) -> Self {
        let memory = gba
            .affine_object_matrix_memory
            .request_slot()
            .expect("Out of affine matrix memory.");

        Self {
            param_a: i16fx8::wrapping_from(1),
            param_b: i16fx8::wrapping_from(0),
            param_c: i16fx8::wrapping_from(0),
            param_d: i16fx8::wrapping_from(1),
            memory,
        }
    }

    pub fn commit_to_memory(&mut self) {
        // This involves writing to memory outside of the GBA manager.
        // That's because the gba crate has separate mmio definitions for each affine matrix parameter.
        // But we need to use params with the same indexed, so really they are logically a series of 4-tuples
        // So by claiming param A, we can then use the other params with the same index
        // And so long as we only use B, C and D when A is owned, it is safe.
        let addr = self.memory.get_index();

        // Write A
        self.memory.as_vol_address().write(self.param_a);

        // Write the others with the same index.
        AFFINE_PARAM_B.index(addr).write(self.param_b);
        AFFINE_PARAM_C.index(addr).write(self.param_c);
        AFFINE_PARAM_D.index(addr).write(self.param_d);
    }

    pub fn index(&self) -> u16 {
        self.memory.get_index().try_into().unwrap()
    }
}
