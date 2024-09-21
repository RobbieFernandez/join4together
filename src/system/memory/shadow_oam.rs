use core::cell::UnsafeCell;

use gba::prelude::*;

use super::{
    error::OutOfMemoryError,
    slot::{ClaimedMemorySlot, MemorySlotTracker},
};

const OAM_SIZE: usize = 128;

pub struct ShadowOAM {
    mem: [UnsafeCell<ObjAttr>; OAM_SIZE],
    tracker: MemorySlotTracker<OAM_SIZE>,
}

pub struct OAMEntry<'a> {
    obj_attr: &'a mut ObjAttr,
    _slot: ClaimedMemorySlot<'a>,
}

impl<'a> OAMEntry<'a> {
    pub fn get_obj_attr(&mut self) -> &mut ObjAttr {
        self.obj_attr
    }
}

impl<'a> Drop for OAMEntry<'a> {
    fn drop(&mut self) {
        self.obj_attr.set_style(ObjDisplayStyle::NotDisplayed);
    }
}

impl ShadowOAM {
    pub fn new() -> Self {
        let mem: [UnsafeCell<ObjAttr>; OAM_SIZE] = core::array::from_fn(|_| {
            let mut oa = ObjAttr::new();
            oa.set_style(ObjDisplayStyle::NotDisplayed);
            UnsafeCell::new(oa)
        });

        Self {
            mem,
            tracker: MemorySlotTracker::new(),
        }
    }

    pub fn request_memory<'a>(&'a self) -> Result<OAMEntry<'a>, OutOfMemoryError> {
        let slot = self.tracker.request_slot()?;
        let obj_attr = &self.mem[slot.index()];
        unsafe {
            *obj_attr.get() = ObjAttr::new();
            let obj_attr = &mut *obj_attr.get();
            obj_attr.set_style(ObjDisplayStyle::Normal);
            Ok(OAMEntry {
                obj_attr,
                _slot: slot,
            })
        }
    }

    /// Sync the shadow OAM to the real OAM
    ///
    /// # Safety
    /// Must be called during VBLANK
    pub unsafe fn sync(&self) {
        for i in 0..OAM_SIZE {
            let oa = &*self.mem[i].get();
            OBJ_ATTR_ALL.index(i).write(*oa);
        }
    }
}
