use gba::mmio::BG_PALETTE;
use voladdress::VolBlock;

struct MemoryManager<T, R, W, const C: usize> {
    block: VolBlock<T, R, W, C>,
    allocation_arr: [bool; C]
}

struct FreeMemoryRange<'a> {
    chunk: &'a [bool],
}

struct ClaimedMemoryRange<'a> {
    chunk: &'a [bool],
}


impl<'a> FreeMemoryRange<'a> {
    fn into_claimed(self) -> ClaimedMemoryRange<'a> {
        unsafe { 
            ClaimedMemoryRange::new(self.chunk)
        }
    }
}

impl<'a> ClaimedMemoryRange<'a> {
    unsafe fn new(chunk: &'a [bool]) -> ClaimedMemoryRange<'a> {
        for i in 0..chunk.len() {
            let e = chunk.get(i).unwrap();
            let e_ptr = e as *const bool;
            let e_ptr = e_ptr.cast_mut();

            unsafe {
                *e_ptr = true;
            }
        }

        ClaimedMemoryRange { chunk }
    }
}

impl<'a> Drop for ClaimedMemoryRange<'a> {
    fn drop(&mut self) {
        for i in 0..self.chunk.len() {
            let e = self.chunk.get(i).unwrap();
            let e_ptr = e as *const bool;
            let e_ptr = e_ptr.cast_mut();

            unsafe {
                *e_ptr = false;
            }
        }
    }
}

impl<T, R, W, const C: usize> MemoryManager<T, R, W, C> {
    fn new(block: VolBlock<T, R, W, C>) -> Self {
        MemoryManager {
            block,
            allocation_arr: [false; C]
        }
    }

    fn find_available_memory_range(&self, requested_size: usize) -> FreeMemoryRange {
        let mut pos = 0;

        while pos < self.allocation_arr.len() {
            let mut iter = (&self.allocation_arr[pos..]).iter();

            let start_of_range = iter.position(|e| { !e }).unwrap() + pos;
            let start_of_claimed_range = iter.position(|e| { *e });

            let range_size = match start_of_claimed_range {
                Some(n) =>  n,
                None => self.allocation_arr.len() - start_of_range
            };

            let end_of_range = start_of_range + range_size;
            
            if range_size > requested_size {
                return FreeMemoryRange {
                    chunk: &self.allocation_arr[start_of_range..end_of_range]
                }
            } 

            pos = end_of_range;
        }

        panic!("Out of memory!");
    }

    fn request_memory(&self, size: usize) -> ClaimedMemoryRange {
       self.find_available_memory_range(size).into_claimed()
    }
}


fn test() {
    let mut bg_pallete_manager = MemoryManager::new(BG_PALETTE);
    let mem1 = bg_pallete_manager.request_memory(16);
    let mem2 = bg_pallete_manager.request_memory(16);
}