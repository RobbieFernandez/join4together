use core::{cell::RefCell, marker::PhantomData, ops::Range};

use super::error::OutOfMemoryError;

struct FreeMemoryRange<'a> {
    start: usize,
    length: usize,
    alloc_marker_ptr: *mut bool,
    phantom: PhantomData<&'a ()>
}

pub struct ClaimedMemoryRange<'a> {
    start: usize,
    length: usize,
    alloc_marker_ptr: *mut bool,
    phantom: PhantomData<&'a bool>
}

pub struct ContiguousMemoryTracker<const C: usize> {
    allocation_arr: RefCell<[bool; C]>,
}

impl<'a> FreeMemoryRange<'a> {
    /// Claim this memory region
    /// 
    /// # Safety 
    /// You must ensure that no other FreeMemoryRange exists that overlaps this one.
    unsafe fn into_claimed(self) -> ClaimedMemoryRange<'a> {
        ClaimedMemoryRange::new(self.start, self.length, self.alloc_marker_ptr)
    }
}

impl<'a> ClaimedMemoryRange<'a> {
    /// Claim this memory range, preventing the memory manager from
    /// allocating it again until the object is dropped.
    /// 
    /// # Safety
    /// You must ensure no other ClaimedMemoryRange exists that overlap with this one.
    unsafe fn new(
        start: usize,
        length: usize,
        alloc_marker_ptr: *mut bool,
    ) -> ClaimedMemoryRange<'a> {
        for i in 0..length {
            alloc_marker_ptr.add(i).write(true)
        }

        ClaimedMemoryRange {
            alloc_marker_ptr,
            start,
            length,
            phantom: PhantomData,
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

impl<'a> Drop for ClaimedMemoryRange<'a> {
    fn drop(&mut self) {
        // The pointer is sourced from the memory tracker, and the lifetime guarantees
        // that the tracker outlives this range. So the pointers should always be valid
        unsafe {
            for i in 0..self.length {
                self.alloc_marker_ptr.add(i).write(false)
            }    
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
    ) -> Result<FreeMemoryRange, OutOfMemoryError> {
        let mut chunk_pos = 0; // The index of the last seen aligned chunk
        let num_chunks = C / alignment;
        let allocation_arr = self.allocation_arr.borrow();

        while chunk_pos < allocation_arr.len() {
            let element_pos = chunk_pos * alignment;
            let mut chunk_iter = allocation_arr[element_pos..].chunks_exact(alignment);

            // Find first chunk that contains unclaimed memory.
            let starting_chunk_index = chunk_iter
                .position(|chunk| chunk.iter().all(|e| !*e))
                .expect("Out of memory!");

            // Find the next chunk that contains at least 1 claimed memory address
            let next_claimed_chunk_index = chunk_iter.position(|chunk| chunk.iter().any(|e| *e));

            // If there was no next claimed chunk found, then the free chunk lasts until
            // the end of the underlying VolBlock.
            let num_free_chunks = match next_claimed_chunk_index {
                Some(n) => n,
                None => num_chunks - starting_chunk_index,
            };

            if num_free_chunks >= requested_aligned_chunks {
                let start_of_range = (chunk_pos + starting_chunk_index) * alignment;
                let length: usize = requested_aligned_chunks * alignment;
                
                // Drop borrow to allow re-borrowing as mutable.
                // We're about to return so it's ok.
                drop(allocation_arr);

                let mut allocation_arr = self.allocation_arr.borrow_mut();
                let allocation_arr_ptr = unsafe { allocation_arr.as_mut_ptr().add(start_of_range) };

                return Ok(FreeMemoryRange {
                    start: start_of_range,
                    length,
                    phantom: PhantomData,
                    alloc_marker_ptr: allocation_arr_ptr,
                });
            }

            chunk_pos = match next_claimed_chunk_index {
                Some(next) => chunk_pos + starting_chunk_index + next,
                None => return Err(OutOfMemoryError),
            }
        }

        Err(OutOfMemoryError)
    }

    pub fn request_aligned_memory(
        &'a self,
        alignment: usize,
        aligned_chunks: usize,
    ) -> Result<ClaimedMemoryRange<'a>, OutOfMemoryError> {
        let memory_range = self.find_available_memory_range(alignment, aligned_chunks)?;

        // This is safe so long as this is the only method that ever constructs
        // FreeMemoryRanges. The struct itself is private so this assumption holds true.
        unsafe {
            Ok(memory_range.into_claimed())
        }
    }
}
