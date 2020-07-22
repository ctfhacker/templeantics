use macroquad::*;

/// Rng seeded with rdtsc that is generated using Lehmer64
pub struct Rng {
    value: u128,
}

impl Rng {
    pub fn new() -> Rng {
        let value = (get_frame_time() * get_time() as f32 * 0xdeadbeefcafebabe as u64 as f32);
        let mut res = Rng {
            value: value as u128
        };

        // Cycle through to create some chaos
        for _ in 0..100 {
            let _ = res.next();
        }

        res
    }

    pub fn next(&mut self) -> u64 {
        self.value = self.value.wrapping_mul(0xda942042e4dd58b5);
        (self.value >> 64) as u64
    }

    /// Returns [1, 6]
    pub fn roll_d6(&mut self) -> usize {
        (self.next() % 6 + 1) as usize
    }
}


