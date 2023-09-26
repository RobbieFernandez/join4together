use crate::graphics::background::LoadedBackground;

// The scroll registers are 9 bit, so can only count up to 511.
const MAX_SCROLL: u16 = 512;

pub struct BackgroundScroller {
    x_speed: u16,
    y_speed: u16,
    x_offset: u16,
    y_offset: u16,
    divisor: u16, // Slow down the scroller by not updating every single tick.
    counter: u16,
}

impl BackgroundScroller {
    pub fn new(x_speed: u16, y_speed: u16) -> Self {
        BackgroundScroller {
            x_speed,
            y_speed,
            x_offset: 0,
            y_offset: 0,
            divisor: 1,
            counter: 0,
        }
    }

    pub fn with_divisor(mut self, divisor: u16) -> Self {
        self.divisor = divisor;
        self
    }

    pub fn update(&mut self) {
        self.counter += 1;

        if self.counter % self.divisor == 0 {
            self.x_offset = (self.x_offset + self.x_speed) % MAX_SCROLL;
            self.y_offset = (self.y_offset + self.y_speed) % MAX_SCROLL;
            self.counter = 0;
        }
    }

    pub fn apply_to_background(&self, background: &LoadedBackground) {
        let layer = background.get_layer();
        let h_scroll = layer.get_horizontal_scroll_register();
        let v_scroll = layer.get_vertical_scroll_register();

        h_scroll.write(self.x_offset);
        v_scroll.write(self.y_offset);
    }
}
