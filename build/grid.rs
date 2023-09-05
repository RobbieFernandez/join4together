/// Resze a 2D matrix (represented by the row-major 1D vec `grid_vec`)
/// from (`current_width`, `current_height`) to (`new_width`, `new_height`)
/// The grid will be truncated, or padded (using `T::default()`) as required
pub fn resize_grid<T>(
    grid_vec: Vec<T>,
    width: usize,
    height: usize,
    new_width: usize,
    new_height: usize,
) -> Vec<T>
where
    T: Default + Clone,
{
    if height == new_height && width == new_width {
        return grid_vec;
    }

    let mut resized: Vec<T> = Vec::new();

    // Add each row, with padding, into the aligned vec.
    for row in 0..height {
        let row_start = row * width;
        let mut row_slice = Vec::from(&grid_vec[row_start..(row_start + width)]);
        row_slice.resize(new_width, T::default());
        resized.append(&mut row_slice);
    }

    // Add any needed extra rows
    resized.resize(new_width * new_height, T::default());

    resized
}
