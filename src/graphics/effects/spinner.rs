use crate::graphics::sprite::AffineLoadedObjectEntry;
use crate::math::{cos, sin};

#[derive(Clone)]
pub struct Spinner {
    rotation: u16,
    speed: u16,
    decay: u16,
}

impl Spinner {
    pub fn new(initial_speed: u16, decay: u16) -> Self {
        Self {
            rotation: 0,
            speed: initial_speed,
            decay,
        }
    }

    pub fn set_speed(&mut self, speed: u16) {
        self.speed = speed;
    }

    pub fn update(&mut self) {
        self.speed = if self.speed < self.decay {
            0
        } else {
            self.speed - self.decay
        };

        self.rotation = self.rotation.wrapping_add(self.speed);
    }

    pub fn apply_to_object(&self, obj: &mut AffineLoadedObjectEntry<'_>) {
        // Update the affine matrix to apply the rotation.
        let mat = obj.get_affine_matrix();
        mat.param_a = cos(self.rotation);
        mat.param_b = -sin(self.rotation);
        mat.param_c = sin(self.rotation);
        mat.param_d = cos(self.rotation);
        mat.commit_to_memory();
    }

    pub fn finished(&self) -> bool {
        self.speed == 0
    }

    pub fn rotation(&self) -> u16 {
        self.rotation
    }
}
