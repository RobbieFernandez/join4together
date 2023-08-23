#[derive(Debug)]
pub struct Bin<T> {
    vec: Vec<T>,
    current_size: usize,
    capacity: usize,
}

#[derive(Debug)]
pub struct BinItem<T> {
    item: T,
    size: usize,
}

#[derive(Debug)]
pub enum BinPackError {
    ItemTooLarge,
}

impl<T> Bin<T> {
    fn new(capacity: usize) -> Self {
        Self {
            vec: Vec::new(),
            current_size: 0,
            capacity,
        }
    }

    fn add(&mut self, item: BinItem<T>) {
        self.vec.push(item.item);
        self.current_size += item.size;
    }

    fn item_fits(&self, item: &BinItem<T>) -> bool {
        (self.current_size + item.size) <= self.capacity
    }
}

impl<T> From<Bin<T>> for Vec<T> {
    fn from(val: Bin<T>) -> Self {
        val.vec
    }
}

impl<T> BinItem<T> {
    pub fn new(item: T, size: usize) -> Self {
        Self { item, size }
    }
}

pub fn binpack<T>(
    mut inputs: Vec<BinItem<T>>,
    bin_size: usize,
) -> Result<Vec<Bin<T>>, BinPackError> {
    let mut bins: Vec<Bin<T>> = Vec::new();
    bins.push(Bin::new(bin_size));

    // Sort inputs from largest to smallest.
    inputs.sort_by_key(|i| i.size);
    inputs.reverse();

    for item in inputs {
        // Search for first bin the item fits into.
        if item.size > bin_size {
            return Err(BinPackError::ItemTooLarge);
        }

        let target_bin = bins.iter_mut().find(|b| b.item_fits(&item));

        // If no bin was found, create a new one.
        let bin = match target_bin {
            Some(b) => b,
            None => {
                let bin: Bin<T> = Bin::new(bin_size);
                bins.push(bin);
                bins.last_mut().unwrap()
            }
        };

        bin.add(item);
    }

    Ok(bins)
}
