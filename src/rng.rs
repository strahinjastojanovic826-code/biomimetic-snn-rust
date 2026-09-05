pub struct Xorshift32 {
    state: u32,
}

impl Xorshift32 {
    pub fn new(seed: u32) -> Self {
        Self { state: if seed == 0 { 42 } else { seed } }
    }

    pub fn next_f32(&mut self) -> f32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        (x as f32) / (u32::MAX as f32)
    }

    pub fn poisson_spike(&mut self, rate_hz: f32, dt_ms: f32) -> bool {
        let dt_sec = dt_ms / 1000.0;
        let prob = rate_hz * dt_sec;
        self.next_f32() < prob
    }
}