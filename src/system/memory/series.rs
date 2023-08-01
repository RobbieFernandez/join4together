use core::marker::PhantomData;

// use core::ops::Range;
use voladdress::{VolAddress, VolSeries};

struct FreeMemorySlot<'a> {
    alloc_ptr: *mut bool,
    index: usize,
    _phantom: PhantomData<&'a ()>,
}

struct ClaimedMemorySlot<'a> {
    alloc_ptr: *mut bool,
    index: usize,
    _phantom: PhantomData<&'a ()>,
}

pub struct ClaimedVolAddress<'a, T, R, W> {
    memory_slot: ClaimedMemorySlot<'a>,
    vol_address: VolAddress<T, R, W>,
}

pub struct MemorySeriesManager<T, R, W, const C: usize, const S: usize> {
    series: VolSeries<T, R, W, C, S>,
    allocation_arr: [bool; C],
}

impl<'a> FreeMemorySlot<'a> {
    fn into_claimed(self) -> ClaimedMemorySlot<'a> {
        unsafe { ClaimedMemorySlot::new(self.alloc_ptr, self.index) }
    }
}

impl<'a> ClaimedMemorySlot<'a> {
    unsafe fn new(alloc_ptr: *mut bool, index: usize) -> ClaimedMemorySlot<'a> {
        unsafe {
            assert!(*alloc_ptr == false);
            *alloc_ptr = true;
        }
        ClaimedMemorySlot {
            alloc_ptr,
            index,
            _phantom: PhantomData,
        }
    }

    fn into_claimed_vol_address<T, R, W, const C: usize, const S: usize>(
        self,
        series: VolSeries<T, R, W, C, S>,
    ) -> ClaimedVolAddress<'a, T, R, W> {
        let vol_address = series.index(self.index);
        ClaimedVolAddress {
            memory_slot: self,
            vol_address,
        }
    }
}

impl<'a> Drop for ClaimedMemorySlot<'a> {
    fn drop(&mut self) {
        unsafe {
            assert!(*self.alloc_ptr == true);
            *self.alloc_ptr = false;
        }
    }
}

impl<'a, T, R, W> ClaimedVolAddress<'a, T, R, W> {
    pub fn as_vol_address(&'a mut self) -> &'a VolAddress<T, R, W> {
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

    unsafe fn find_available_memory_slot(&self) -> FreeMemorySlot {
        let iter = &mut self.allocation_arr.iter();

        let next_slot = match iter.position(|e| !e) {
            Some(n) => n,
            _ => panic!("Out of memory"),
        };

        let alloc_ptr: *const bool = &self.allocation_arr[next_slot];

        FreeMemorySlot {
            alloc_ptr: alloc_ptr.cast_mut(),
            index: next_slot,
            _phantom: PhantomData,
        }
    }

    pub fn request_slot(&self) -> ClaimedVolAddress<T, R, W> {
        let series = self.series;
        unsafe {
            self.find_available_memory_slot()
                .into_claimed()
                .into_claimed_vol_address(series)
        }
    }
}
