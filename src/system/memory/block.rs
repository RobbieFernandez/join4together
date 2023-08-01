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
    // Claim this memory range, preventing the memory manager from
    // allocating it again until the returned ClaimedMemoryRange
    // object is dropped.
    // This is marked as unsafe because it assumes that no other
    // overlapping FreeMemoryRange exists. If this assumption is false
    // then this will lead to undefined behaviour.
    unsafe fn into_claimed(self) -> ClaimedMemoryRange<'a> {
        ClaimedMemoryRange::new(self.chunk, self.start)
    }
}

impl<'a> ClaimedMemoryRange<'a> {
    // Claim this memory range, preventing the memory manager from
    // allocating it again until the object is dropped.
    // This is marked as unsafe because it assumes that no other
    // overlapping FreeMemoryRange exists. If this assumption is false
    // then this will lead to undefined behaviour.
    unsafe fn new(chunk: &'a [bool], start: usize) -> ClaimedMemoryRange<'a> {
        for i in 0..chunk.len() {
            let e = chunk.get(i).unwrap();
            let e_ptr = e as *const bool;
            let e_ptr = e_ptr.cast_mut();

            assert!(*e_ptr == false);
            *e_ptr = true;
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

impl<'a, T, R, W, const C: usize> MemoryBlockManager<T, R, W, C> {
    pub fn new(block: VolBlock<T, R, W, C>) -> Self {
        Self {
            block,
            allocation_arr: [false; C],
        }
    }

    fn find_available_memory_range(
        &self,
        alignment: usize,
        requested_aligned_chunks: usize,
    ) -> FreeMemoryRange {
        let mut pos = 0; // The index of the last seen aligned chunk
        let num_chunks = C / alignment;

        while pos < self.allocation_arr.len() {
            let chunk_pos = pos * alignment;
            let mut chunk_iter = (&self.allocation_arr[chunk_pos..]).chunks_exact(alignment);

            // Find first chunk that contains unclaimed memory.
            let starting_chunk_index = chunk_iter
                .position(|chunk| chunk.iter().all(|e| !e))
                .expect("Out of memory!");

            // Find the next chunk that contains at least 1 claimed memory address
            let next_claimed_chunk_index = chunk_iter.position(|chunk| chunk.iter().any(|e| *e));

            // If there was no next claimed chunk found, then the free chunk lasts until
            // the end of the underlying VolBlock.
            let num_free_chunks = match next_claimed_chunk_index {
                Some(n) => n - starting_chunk_index,
                None => num_chunks - starting_chunk_index,
            };

            if num_free_chunks >= requested_aligned_chunks {
                let start_of_range = starting_chunk_index * alignment;
                let end_of_range = start_of_range + requested_aligned_chunks * alignment;

                return FreeMemoryRange {
                    chunk: &self.allocation_arr[start_of_range..end_of_range],
                    start: start_of_range,
                };
            }

            pos = match next_claimed_chunk_index {
                Some(n) => n,
                None => panic!("Out of memory!"),
            }
        }

        panic!("Out of memory!");
    }

    pub fn request_memory(&self, size: usize) -> ClaimedVolRegion<T, R, W> {
        self.request_aligned_memory(1, size)
    }

    pub fn request_aligned_memory(
        &'a self,
        alignment: usize,
        aligned_chunks: usize,
    ) -> ClaimedVolRegion<'a, T, R, W> {
        let block = self.block;
        // This is safe so long as this is the only method that ever constructs
        // FreeMemoryRanges. The struct itself is private so this assumption holds true.
        unsafe {
            self.find_available_memory_range(alignment, aligned_chunks)
                .into_claimed()
                .into_claimed_vol_region(block)
        }
    }
}
