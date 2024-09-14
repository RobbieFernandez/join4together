use voladdress::{VolGrid2d, VolGrid2dStrided};

use super::{
    contiguous::{ClaimedMemoryRange, ContiguousMemoryTracker},
    error::OutOfMemoryError,
};

pub struct ClaimedGridFrames<
    'a,
    T,
    R,
    W,
    const WIDTH: usize,
    const HEIGHT: usize,
    const FRAMES: usize,
    const BYTE_STRIDE: usize,
> {
    strided_grid: &'a VolGrid2dStrided<T, R, W, WIDTH, HEIGHT, FRAMES, BYTE_STRIDE>,
    claimed_memory_range: ClaimedMemoryRange<'a, FRAMES>,
}

pub struct MemoryStridedGridManager<
    T,
    R,
    W,
    const WIDTH: usize,
    const HEIGHT: usize,
    const FRAMES: usize,
    const BYTE_STRIDE: usize,
> {
    strided_grid: VolGrid2dStrided<T, R, W, WIDTH, HEIGHT, FRAMES, BYTE_STRIDE>,
    tracker: ContiguousMemoryTracker<FRAMES>,
}

impl<
        'a,
        T,
        R,
        W,
        const WIDTH: usize,
        const HEIGHT: usize,
        const FRAMES: usize,
        const BYTE_STRIDE: usize,
    > ClaimedGridFrames<'a, T, R, W, WIDTH, HEIGHT, FRAMES, BYTE_STRIDE>
{
    pub fn get_frame(&self, index: usize) -> VolGrid2d<T, R, W, WIDTH, HEIGHT> {
        let index = index + self.claimed_memory_range.start();
        self.strided_grid
            .get_frame(index)
            .expect("VolGrid2D frame index out of bounds")
    }

    pub fn get_start(&self) -> usize {
        self.claimed_memory_range.start()
    }
}

impl<
        'a,
        T,
        R,
        W,
        const WIDTH: usize,
        const HEIGHT: usize,
        const FRAMES: usize,
        const BYTE_STRIDE: usize,
    > MemoryStridedGridManager<T, R, W, WIDTH, HEIGHT, FRAMES, BYTE_STRIDE>
{
    pub fn new(
        strided_grid: VolGrid2dStrided<T, R, W, WIDTH, HEIGHT, FRAMES, BYTE_STRIDE>,
    ) -> Self {
        Self {
            strided_grid,
            tracker: ContiguousMemoryTracker::new(),
        }
    }

    pub fn request_memory(
        &self,
        size: usize,
    ) -> Result<ClaimedGridFrames<T, R, W, WIDTH, HEIGHT, FRAMES, BYTE_STRIDE>, OutOfMemoryError>
    {
        self.request_aligned_memory(1, size)
    }

    pub fn request_aligned_memory(
        &self,
        alignment: usize,
        aligned_chunks: usize,
    ) -> Result<ClaimedGridFrames<T, R, W, WIDTH, HEIGHT, FRAMES, BYTE_STRIDE>, OutOfMemoryError>
    {
        let claimed_memory_range = self
            .tracker
            .request_aligned_memory(alignment, aligned_chunks)?;

        let claimed = ClaimedGridFrames {
            claimed_memory_range,
            strided_grid: &self.strided_grid,
        };

        Ok(claimed)
    }
}
