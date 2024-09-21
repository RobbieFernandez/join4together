use voladdress::{VolAddress, VolSeries};

use super::{
    error::OutOfMemoryError,
    slot::{ClaimedMemorySlot, MemorySlotTracker},
};

pub struct ClaimedVolAddress<'a, T, R, W> {
    memory_slot: ClaimedMemorySlot<'a>,
    vol_address: VolAddress<T, R, W>,
}

pub struct MemorySeriesManager<T, R, W, const C: usize, const S: usize> {
    series: VolSeries<T, R, W, C, S>,
    tracker: MemorySlotTracker<C>,
}

impl<'a, T, R, W> ClaimedVolAddress<'a, T, R, W> {
    pub fn as_vol_address(&mut self) -> &VolAddress<T, R, W> {
        &self.vol_address
    }

    pub fn get_index(&self) -> usize {
        self.memory_slot.index()
    }

    fn from_claimed_slot<const C: usize, const S: usize>(
        memory_slot: ClaimedMemorySlot<'a>,
        vol_series: VolSeries<T, R, W, C, S>,
    ) -> Self {
        let vol_address = vol_series.index(memory_slot.index());
        Self {
            vol_address,
            memory_slot,
        }
    }
}

impl<T, R, W, const C: usize, const S: usize> MemorySeriesManager<T, R, W, C, S> {
    pub fn new(series: VolSeries<T, R, W, C, S>) -> Self {
        Self {
            series,
            tracker: MemorySlotTracker::new(),
        }
    }

    pub fn request_slot(&self) -> Result<ClaimedVolAddress<T, R, W>, OutOfMemoryError> {
        let mem_slot = self.tracker.request_slot()?;
        Ok(ClaimedVolAddress::from_claimed_slot(mem_slot, self.series))
    }
}
