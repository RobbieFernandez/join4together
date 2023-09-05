use core::{cell::RefCell, ops::Range};

use super::error::OutOfMemoryError;

struct FreeMemoryRange<'a, const C: usize> {
    allocation_arr: &'a RefCell<[bool; C]>,
    start: usize,
    length: usize,
}

pub struct ClaimedMemoryRange<'a, const C: usize> {
    allocation_arr: &'a RefCell<[bool; C]>,
    start: usize,
    length: usize,
}

pub struct ContiguousMemoryTracker<const C: usize> {
    allocation_arr: RefCell<[bool; C]>,
}

impl<'a, const C: usize> FreeMemoryRange<'a, C> {
    fn into_claimed(self) -> ClaimedMemoryRange<'a, C> {
        ClaimedMemoryRange::new(self.allocation_arr, self.start, self.length)
    }
}

impl<'a, const C: usize> ClaimedMemoryRange<'a, C> {
    // Claim this memory range, preventing the memory manager from
    // allocating it again until the object is dropped.
    fn new(
        allocation_arr_cell: &'a RefCell<[bool; C]>,
        start: usize,
        length: usize,
    ) -> ClaimedMemoryRange<'a, C> {
        let mut allocation_arr = allocation_arr_cell.borrow_mut();

        for i in start..(start + length) {
            let e = allocation_arr.get_mut(i).unwrap();
            assert!(!(*e));
            *e = true;
        }

        ClaimedMemoryRange {
            allocation_arr: allocation_arr_cell,
            start,
            length,
        }
    }

    /// Get the start and end point of this memory range.
    /// Note that this is relative to the starting point of the underlying block.
    pub fn address_range(&self) -> Range<usize> {
        self.start..(self.start + self.length)
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn size(&self) -> usize {
        self.length
    }
}

impl<'a, const C: usize> Drop for ClaimedMemoryRange<'a, C> {
    fn drop(&mut self) {
        let mut allocation_arr = self.allocation_arr.borrow_mut();

        for i in self.start..(self.start + self.length) {
            let e = allocation_arr.get_mut(i).unwrap();
            assert!(*e);
            *e = false;
        }
    }
}

impl<'a, const C: usize> ContiguousMemoryTracker<C> {
    pub fn new() -> Self {
        Self {
            allocation_arr: RefCell::new([false; C]),
        }
    }

    fn find_available_memory_range(
        &self,
        alignment: usize,
        requested_aligned_chunks: usize,
    ) -> Result<FreeMemoryRange<C>, OutOfMemoryError> {
        let mut pos = 0; // The index of the last seen aligned chunk
        let num_chunks = C / alignment;
        let allocation_arr = self.allocation_arr.borrow();

        while pos < allocation_arr.len() {
            let chunk_pos = pos * alignment;
            let mut chunk_iter = allocation_arr[chunk_pos..].chunks_exact(alignment);

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
                let length = requested_aligned_chunks * alignment;

                return Ok(FreeMemoryRange {
                    allocation_arr: &self.allocation_arr,
                    start: start_of_range,
                    length,
                });
            }

            pos = match next_claimed_chunk_index {
                Some(n) => n,
                None => return Err(OutOfMemoryError),
            }
        }

        Err(OutOfMemoryError)
    }

    pub fn request_aligned_memory(
        &'a self,
        alignment: usize,
        aligned_chunks: usize,
    ) -> Result<ClaimedMemoryRange<'a, C>, OutOfMemoryError> {
        // This is safe so long as this is the only method that ever constructs
        // FreeMemoryRanges. The struct itself is private so this assumption holds true.
        let memory_range = self.find_available_memory_range(alignment, aligned_chunks)?;

        Ok(memory_range.into_claimed())
    }
}
