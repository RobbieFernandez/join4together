use core::{cell::RefCell, marker::PhantomData};

use super::error::OutOfMemoryError;

pub struct ClaimedMemorySlot<'a> {
    index: usize,
    alloc_status_ptr: *mut bool,
    phantom: PhantomData<&'a ()>,
}

struct FreeMemorySlot<'a> {
    index: usize,
    alloc_status_ptr: *mut bool,
    phantom: PhantomData<&'a ()>,
}

pub struct MemorySlotTracker<const C: usize> {
    slot_allocation_status: RefCell<[bool; C]>,
}

impl<'a> ClaimedMemorySlot<'a> {
    unsafe fn new(index: usize, alloc_status_ptr: *mut bool) -> Self {
        alloc_status_ptr.write(true);
        Self {
            index,
            alloc_status_ptr,
            phantom: PhantomData,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

impl<'a> Drop for ClaimedMemorySlot<'a> {
    fn drop(&mut self) {
        unsafe {
            self.alloc_status_ptr.write(false);
        }
    }
}

impl<'a> FreeMemorySlot<'a> {
    unsafe fn into_claimed(self) -> ClaimedMemorySlot<'a> {
        ClaimedMemorySlot::new(self.index, self.alloc_status_ptr)
    }
}

impl<'a, const C: usize> MemorySlotTracker<C> {
    pub fn new() -> Self {
        Self {
            slot_allocation_status: RefCell::new([false; C]),
        }
    }

    pub fn request_slot(&self) -> Result<ClaimedMemorySlot<'a>, OutOfMemoryError> {
        let free_slot = self.find_free_slot()?;

        unsafe { Ok(free_slot.into_claimed()) }
    }

    fn find_free_slot(&self) -> Result<FreeMemorySlot<'a>, OutOfMemoryError> {
        let alloc_arr = self.slot_allocation_status.borrow();
        let index = alloc_arr
            .iter()
            .enumerate()
            .filter_map(|(i, claimed)| if *claimed { None } else { Some(i) })
            .next()
            .ok_or(OutOfMemoryError)?;

        drop(alloc_arr);
        let mut alloc_arr = self.slot_allocation_status.borrow_mut();
        let alloc_status_ptr = unsafe { alloc_arr.as_mut_ptr().add(index) };

        Ok(FreeMemorySlot {
            index,
            alloc_status_ptr,
            phantom: PhantomData,
        })
    }
}
