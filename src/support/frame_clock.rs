
use std::time;

#[derive(Debug)]
pub struct FrameClock {
    clock: time::Instant,
    previous: u64,
    elapsed: u64
}

impl FrameClock {
    pub fn new() -> FrameClock {
        FrameClock {
            clock: time::Instant::now(),
            previous: 0,
            elapsed: u64::max_value()
        }
    }

    pub fn get_fps(&self) -> u64 {
        if self.elapsed == 0 {
            return 0;
        }
        1_000 / self.elapsed
    }

    pub fn get_last_frame_duration(&self) -> u64 {
        self.elapsed
    }

    fn get_duration(&self) -> u64 {
        let t = self.clock.elapsed();
        (t.as_secs() * 1_000) + (t.subsec_nanos() / 1_000_000) as u64
    }

    pub fn tick(&mut self) {
        let current = self.get_duration();
        self.elapsed = current - self.previous;
        self.previous = current;
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_fps() {
        // TODO
        assert!(1==1);
    }
}
