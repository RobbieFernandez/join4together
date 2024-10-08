use voladdress::{VolBlock, VolRegion};

use super::{
    contiguous::{ClaimedMemoryRange, ContiguousMemoryTracker},
    error::OutOfMemoryError,
};

pub struct ClaimedVolRegion<'a, T, R, W> {
    vol_region: VolRegion<T, R, W>,
    claimed_memory_range: ClaimedMemoryRange<'a>,
}

pub struct MemoryBlockManager<T, R, W, const C: usize> {
    block: VolBlock<T, R, W, C>,
    tracker: ContiguousMemoryTracker<C>,
}

impl<'a, T, R, W> ClaimedVolRegion<'a, T, R, W> {
    fn new<const C: usize>(
        block: &'a VolBlock<T, R, W, C>,
        claimed_memory_range: ClaimedMemoryRange<'a>,
    ) -> Self {
        let addr_range = claimed_memory_range.address_range();
        let region: VolRegion<T, R, W> = block.as_region();

        Self {
            vol_region: region.sub_slice(addr_range),
            claimed_memory_range,
        }
    }
}

impl<'a, T, R, W> ClaimedVolRegion<'a, T, R, W> {
    pub fn as_vol_region(&mut self) -> &VolRegion<T, R, W> {
        &self.vol_region
    }

    pub fn get_start(&self) -> usize {
        self.claimed_memory_range.start()
    }

    pub fn size(&self) -> usize {
        self.claimed_memory_range.size()
    }
}

impl<'a, T, R, W, const C: usize> MemoryBlockManager<T, R, W, C> {
    pub fn new(block: VolBlock<T, R, W, C>) -> Self {
        Self {
            block,
            tracker: ContiguousMemoryTracker::new(),
        }
    }

    pub fn request_memory(
        &self,
        size: usize,
    ) -> Result<ClaimedVolRegion<T, R, W>, OutOfMemoryError> {
        self.request_aligned_memory(1, size)
    }

    pub fn request_aligned_memory(
        &self,
        alignment: usize,
        aligned_chunks: usize,
    ) -> Result<ClaimedVolRegion<T, R, W>, OutOfMemoryError> {
        let memory_range = self
            .tracker
            .request_aligned_memory(alignment, aligned_chunks)?;

        Ok(self.memory_range_to_vol_region(memory_range))
    }

    fn memory_range_to_vol_region(
        &'a self,
        memory_range: ClaimedMemoryRange<'a>,
    ) -> ClaimedVolRegion<'a, T, R, W> {
        ClaimedVolRegion::new(&self.block, memory_range)
    }
}
