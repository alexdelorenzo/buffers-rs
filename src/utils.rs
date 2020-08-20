const NO_CAPACITY: usize = 0;


pub fn sized_vec<T: Clone>(size: usize, item: T) -> Vec<T> {
    let mut vec = Vec::with_capacity(size);
    vec.resize(size, item);
    vec
}

pub fn no_capacity_vec<T>() -> Vec<T> {
    Vec::with_capacity(NO_CAPACITY)
}