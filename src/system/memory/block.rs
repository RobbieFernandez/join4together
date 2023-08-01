use core::ops::Range;
use voladdress::{VolBlock, VolRegion};

struct FreeMemoryRange<'a> {
    chunk: &'a [bool],
    start: usize,
}

struct ClaimedMemoryRange<'a> {
    chunk: &'a [bool],
    start: usize,
}

pub struct ClaimedVolRegion<'a, T, R, W> {
    memory_range: ClaimedMemoryRange<'a>,
    vol_region: VolRegion<T, R, W>,
}

pub struct MemoryBlockManager<T, R, W, const C: usize> {
    block: VolBlock<T, R, W, C>,
    allocation_arr: [bool; C],
}

impl<'a> FreeMemoryRange<'a> {
    fn into_claimed(self) -> ClaimedMemoryRange<'a> {
        unsafe { ClaimedMemoryRange::new(self.chunk, self.start) }
    }
}

impl<'a> ClaimedMemoryRange<'a> {
    unsafe fn new(chunk: &'a [bool], start: usize) -> ClaimedMemoryRange<'a> {
        for i in 0..chunk.len() {
            let e = chunk.get(i).unwrap();
            let e_ptr = e as *const bool;
            let e_ptr = e_ptr.cast_mut();

            unsafe {
                assert!(*e_ptr == false);
                *e_ptr = true;
            }
        }

        ClaimedMemoryRange { chunk, start }
    }

    fn len(&self) -> usize {
        self.chunk.len()
    }

    fn address_range(&self) -> Range<usize> {
        self.start..(self.start + self.len())
    }

    fn into_claimed_vol_region<T, R, W, const C: usize>(
        self,
        block: VolBlock<T, R, W, C>,
    ) -> ClaimedVolRegion<'a, T, R, W> {
        let addr_range = self.address_range();
        let region = block.as_region();

        ClaimedVolRegion {
            memory_range: self,
            vol_region: region.sub_slice(addr_range),
        }
    }
}

impl<'a> Drop for ClaimedMemoryRange<'a> {
    fn drop(&mut self) {
        for i in 0..self.chunk.len() {
            let e = self.chunk.get(i).unwrap();
            let e_ptr = e as *const bool;
            let e_ptr = e_ptr.cast_mut();

            unsafe {
                assert!(*e_ptr == true);
                *e_ptr = false;
            }
        }
    }
}

impl<'a, T, R, W> ClaimedVolRegion<'a, T, R, W> {
    pub fn as_vol_region(&mut self) -> &VolRegion<T, R, W> {
        &self.vol_region
    }

    pub fn get_start(&self) -> usize {
        self.memory_range.start
    }

    pub fn len(&self) -> usize {
        self.memory_range.len()
    }
}

impl<T, R, W, const C: usize> MemoryBlockManager<T, R, W, C> {
    pub fn new(block: VolBlock<T, R, W, C>) -> Self {
        Self {
            block,
            allocation_arr: [false; C],
        }
    }

    fn find_available_memory_range(&mut self, requested_size: usize) -> FreeMemoryRange {
        let mut pos = 0;

        while pos < self.allocation_arr.len() {
            let mut iter = (&self.allocation_arr[pos..]).iter();

            let start_of_range = iter.position(|e| !e).unwrap() + pos;
            let end_of_range = iter.position(|e| *e);

            let range_size = match end_of_range {
                Some(n) => n,
                None => self.allocation_arr.len() - start_of_range,
            };

            let end_of_range = start_of_range + range_size;

            pos = end_of_range;

            if range_size >= requested_size {
                return FreeMemoryRange {
                    chunk: &self.allocation_arr[start_of_range..end_of_range],
                    start: start_of_range,
                };
            }
        }

        panic!("Out of memory!");
    }

    pub fn request_memory(&mut self, size: usize) -> ClaimedVolRegion<T, R, W> {
        let block = self.block;
        self.find_available_memory_range(size)
            .into_claimed()
            .into_claimed_vol_region(block)
    }
}
