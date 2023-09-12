use crate::system::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::system::gba::GBA;

use crate::graphics::sprite::{
    LoadedObjectEntry, LoadedSprite, CPU_FACE_HAPPY_SPRITE, CPU_FACE_MAD_SPRITE,
    CPU_FACE_NEUTRAL_SPRITE, CPU_FACE_SAD_SPRITE, CPU_FACE_SURPRISED_SPRITE, CPU_HEAD_SPRITE,
};

pub struct CpuSprites<'a> {
    _gba: &'a GBA,
    cpu_head_sprite: LoadedSprite<'a>,
    cpu_neutral_face_sprite: LoadedSprite<'a>,
    cpu_happy_face_sprite: LoadedSprite<'a>,
    cpu_mad_face_sprite: LoadedSprite<'a>,
    cpu_surprised_face_sprite: LoadedSprite<'a>,
    cpu_sad_face_sprite: LoadedSprite<'a>,
}

pub struct CpuFace<'a> {
    _gba: &'a GBA,
    _cpu_head_obj: LoadedObjectEntry<'a>,
    cpu_face_obj: LoadedObjectEntry<'a>,
    cpu_sprites: &'a CpuSprites<'a>,
}

#[derive(Clone, Copy)]
pub enum CpuEmotion {
    Neutral,
    Happy,
    Mad,
    Surprised,
    Sad,
}

impl<'a> CpuSprites<'a> {
    pub fn new(gba: &'a GBA) -> Self {
        let cpu_head_sprite = CPU_HEAD_SPRITE.load(gba);
        let cpu_neutral_face_sprite = CPU_FACE_NEUTRAL_SPRITE.load(gba);
        let cpu_happy_face_sprite = CPU_FACE_HAPPY_SPRITE.load(gba);
        let cpu_mad_face_sprite = CPU_FACE_MAD_SPRITE.load(gba);
        let cpu_surprised_face_sprite = CPU_FACE_SURPRISED_SPRITE.load(gba);
        let cpu_sad_face_sprite = CPU_FACE_SAD_SPRITE.load(gba);

        Self {
            cpu_head_sprite,
            cpu_neutral_face_sprite,
            cpu_happy_face_sprite,
            cpu_mad_face_sprite,
            cpu_surprised_face_sprite,
            cpu_sad_face_sprite,
            _gba: gba,
        }
    }

    fn get_head_sprite(&'a self) -> &'a LoadedSprite<'a> {
        &self.cpu_head_sprite
    }

    fn get_face_sprite(&'a self, emotion: CpuEmotion) -> &'a LoadedSprite<'a> {
        match emotion {
            CpuEmotion::Neutral => &self.cpu_neutral_face_sprite,
            CpuEmotion::Happy => &self.cpu_happy_face_sprite,
            CpuEmotion::Mad => &self.cpu_mad_face_sprite,
            CpuEmotion::Surprised => &self.cpu_surprised_face_sprite,
            CpuEmotion::Sad => &self.cpu_sad_face_sprite,
        }
    }

    pub fn height() -> usize {
        CPU_HEAD_SPRITE.height()
    }

    pub fn width() -> usize {
        CPU_HEAD_SPRITE.width()
    }
}

impl<'a> CpuFace<'a> {
    pub fn new(gba: &'a GBA, cpu_sprites: &'a CpuSprites<'a>) -> Self {
        let mut cpu_head_obj = cpu_sprites.get_head_sprite().create_obj_attr_entry(gba);

        let cpu_head_height: u16 = CpuSprites::height().try_into().unwrap();
        let cpu_head_width: u16 = CpuSprites::width().try_into().unwrap();

        let y_pos = SCREEN_HEIGHT - cpu_head_height;
        let x_pos = SCREEN_WIDTH - cpu_head_width - 5;

        let mut cpu_face_obj = cpu_sprites
            .get_face_sprite(CpuEmotion::Neutral)
            .create_obj_attr_entry(gba);

        for obj in [&mut cpu_face_obj, &mut cpu_head_obj] {
            let oa = obj.get_obj_attr_data();
            oa.0 = oa.0.with_y(y_pos);
            oa.1 = oa.1.with_x(x_pos);
            obj.commit_to_memory();
        }

        Self {
            cpu_face_obj,
            cpu_sprites,
            _cpu_head_obj: cpu_head_obj,
            _gba: gba,
        }
    }

    pub fn set_emotion(&mut self, emotion: CpuEmotion) {
        let face_sprite = self.cpu_sprites.get_face_sprite(emotion);
        face_sprite.store_in_obj_entry(&mut self.cpu_face_obj);
        self.cpu_face_obj.commit_to_memory();
    }
}
