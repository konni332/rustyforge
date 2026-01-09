use std::num::NonZero;

pub fn get_number_of_max_parallel_threads() -> NonZero<usize> {
    match std::thread::available_parallelism() {
        Ok(threads) => threads,
        Err(_) => NonZero::new(1usize).unwrap(),
    }
}
