use core::cell::RefCell;

// use core::ops::Range;
use voladdress::{VolAddress, VolSeries};

use super::error::OutOfMemoryError;

struct FreeMemorySlot<'a, const C: usize> {
    allocation_arr: &'a RefCell<[bool; C]>,
    index: usize,
}

struct ClaimedMemorySlot<'a, const C: usize> {
    allocation_arr: &'a RefCell<[bool; C]>,
    index: usize,
}

pub struct ClaimedVolAddress<'a, T, R, W, const C: usize> {
    memory_slot: ClaimedMemorySlot<'a, C>,
    vol_address: VolAddress<T, R, W>,
}

pub struct MemorySeriesManager<T, R, W, const C: usize, const S: usize> {
    series: VolSeries<T, R, W, C, S>,
    allocation_arr: RefCell<[bool; C]>,
}

impl<'a, const C: usize> FreeMemorySlot<'a, C> {
    fn into_claimed(self) -> ClaimedMemorySlot<'a, C> {
        ClaimedMemorySlot::new(self.allocation_arr, self.index)
    }
}

impl<'a, const C: usize> ClaimedMemorySlot<'a, C> {
    fn new(allocation_arr_cell: &'a RefCell<[bool; C]>, index: usize) -> ClaimedMemorySlot<'a, C> {
        let mut allocation_arr = allocation_arr_cell.borrow_mut();
        let alloc = allocation_arr.get_mut(index).unwrap();
        assert!(!*alloc);

        *alloc = true;
        ClaimedMemorySlot {
            index,
            allocation_arr: allocation_arr_cell,
        }
    }

    fn into_claimed_vol_address<T, R, W, const S: usize>(
        self,
        series: VolSeries<T, R, W, C, S>,
    ) -> ClaimedVolAddress<'a, T, R, W, C> {
        let vol_address = series.index(self.index);
        ClaimedVolAddress {
            memory_slot: self,
            vol_address,
        }
    }
}

impl<'a, const C: usize> Drop for ClaimedMemorySlot<'a, C> {
    fn drop(&mut self) {
        let mut allocation_arr = self.allocation_arr.borrow_mut();
        let alloc = allocation_arr.get_mut(self.index).unwrap();
        assert!(*alloc);
        *alloc = false;
    }
}

impl<'a, T, R, W, const C: usize> ClaimedVolAddress<'a, T, R, W, C> {
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
            allocation_arr: RefCell::new([false; C]),
        }
    }

    fn find_available_memory_slot(&self) -> Result<FreeMemorySlot<C>, OutOfMemoryError> {
        let next_slot = {
            let allocation_arr = self.allocation_arr.borrow();
            let mut iter = allocation_arr.iter();

            iter.position(|e| !e)
        };

        match next_slot {
            Some(n) => Ok(FreeMemorySlot {
                allocation_arr: &self.allocation_arr,
                index: n,
            }),
            None => Err(OutOfMemoryError),
        }
    }

    pub fn request_slot(&self) -> Result<ClaimedVolAddress<T, R, W, C>, OutOfMemoryError> {
        let series = self.series;
        let mem = self.find_available_memory_slot()?;
        Ok(mem.into_claimed().into_claimed_vol_address(series))
    }
}
