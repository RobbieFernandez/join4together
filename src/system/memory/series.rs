// use core::ops::Range;
use voladdress::{VolAddress, VolSeries};

struct FreeMemorySlot {
    alloc_ptr: *mut bool,
    index: usize,
}

struct ClaimedMemorySlot {
    alloc_ptr: *mut bool,
    index: usize,
}

pub struct ClaimedVolAddress<T, R, W> {
    memory_slot: ClaimedMemorySlot,
    vol_address: VolAddress<T, R, W>,
}

pub struct MemorySeriesManager<T, R, W, const C: usize, const S: usize> {
    series: VolSeries<T, R, W, C, S>,
    allocation_arr: [bool; C],
}

impl FreeMemorySlot {
    fn into_claimed(self) -> ClaimedMemorySlot {
        unsafe { ClaimedMemorySlot::new(self.alloc_ptr, self.index) }
    }
}

impl ClaimedMemorySlot {
    unsafe fn new(alloc_ptr: *mut bool, index: usize) -> ClaimedMemorySlot {
        unsafe {
            assert!(*alloc_ptr == false);
            *alloc_ptr = true;
        }
        ClaimedMemorySlot { alloc_ptr, index }
    }

    fn into_claimed_vol_address<T, R, W, const C: usize, const S: usize>(
        self,
        series: VolSeries<T, R, W, C, S>,
    ) -> ClaimedVolAddress<T, R, W> {
        let vol_address = series.index(self.index);
        ClaimedVolAddress {
            memory_slot: self,
            vol_address,
        }
    }
}

impl Drop for ClaimedMemorySlot {
    fn drop(&mut self) {
        unsafe {
            assert!(*self.alloc_ptr == true);
            *self.alloc_ptr = false;
        }
    }
}

impl<T, R, W> ClaimedVolAddress<T, R, W> {
    pub fn as_vol_address(&mut self) -> &VolAddress<T, R, W> {
        &self.vol_address
    }

    pub fn get_index(&self) -> usize {
        self.memory_slot.index
    }
}

impl<T, R, W, const C: usize, const S: usize> MemorySeriesManager<T, R, W, C, S> {
    pub fn new(series: VolSeries<T, R, W, C, S>) -> Self {
        Self {
            series,
            allocation_arr: [false; C],
        }
    }

    unsafe fn find_available_memory_slot(&mut self) -> FreeMemorySlot {
        let iter = &mut self.allocation_arr.iter();

        let next_slot = match iter.position(|e| !e) {
            Some(n) => n,
            _ => panic!("Out of memory"),
        };

        let alloc_ptr: *const bool = &self.allocation_arr[next_slot];

        FreeMemorySlot {
            alloc_ptr: alloc_ptr.cast_mut(),
            index: next_slot,
        }
    }

    pub fn request_slot(&mut self) -> ClaimedVolAddress<T, R, W> {
        unsafe {
            self.find_available_memory_slot()
                .into_claimed()
                .into_claimed_vol_address(self.series)
        }
    }
}
