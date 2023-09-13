use gba::prelude::ObjDisplayStyle;

use super::sprite::LoadedObjectEntry;

#[derive(Clone)]
enum BlinkState {
    On(u32),
    Off(u32),
}

#[derive(Clone)]
pub struct Blinker {
    time_on: u32,
    time_off: u32,
    state: BlinkState,
}

impl Blinker {
    pub fn new(time_on: u32, time_off: u32, initial_status: bool) -> Self {
        let state = if initial_status {
            BlinkState::On(time_on)
        } else {
            BlinkState::Off(time_off)
        };

        Self {
            time_on,
            time_off,
            state,
        }
    }

    pub fn update(&mut self) {
        self.state = match self.state {
            BlinkState::On(time) => {
                if time > 0 {
                    BlinkState::On(time - 1)
                } else {
                    BlinkState::Off(self.time_off)
                }
            }
            BlinkState::Off(time) => {
                if time > 0 {
                    BlinkState::Off(time - 1)
                } else {
                    BlinkState::On(self.time_on)
                }
            }
        }
    }

    pub fn apply_to_object(&self, obj: &mut LoadedObjectEntry<'_>) {
        let oa = obj.get_obj_attr_data();
        oa.set_style(match self.state {
            BlinkState::On(_) => ObjDisplayStyle::Normal,
            BlinkState::Off(_) => ObjDisplayStyle::NotDisplayed,
        })
    }
}
