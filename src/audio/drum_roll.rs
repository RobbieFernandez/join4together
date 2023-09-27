use super::noise::play_impact_noise;

#[derive(Clone)]
pub struct DrumRoll {
    delay: u32,
    counter: u32,
    next_delay: u32,
}

impl DrumRoll {
    pub fn new(delay: u32) -> Self {
        Self {
            delay,
            next_delay: delay,
            counter: 0,
        }
    }

    pub fn set_delay(&mut self, delay: u32) {
        self.next_delay = delay;
    }

    pub fn update(&mut self) {
        self.counter += 1;
        if self.counter % self.delay == 0 {
            play_impact_noise();
            self.counter = 0;
            self.delay = self.next_delay;
        }
    }
}
