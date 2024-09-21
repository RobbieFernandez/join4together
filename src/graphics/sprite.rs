use gba::video::{
    obj::{ObjAttr, ObjDisplayStyle, ObjShape},
    Tile4,
};

use voladdress::Safe;

use super::affine::AffineMatrix;
use crate::system::gba::{ClaimedVolRegion, OAMEntry, GBA};

pub struct Sprite {
    tiles: &'static [Tile4],
    palette_bank: u8,
    shape: ObjShape,
    size: u16,
    width: usize,
    height: usize,
}

pub struct LoadedSprite<'a> {
    sprite: &'a Sprite,
    memory: ClaimedVolRegion<'a, Tile4, Safe, Safe>,
}

pub struct Animation<const C: usize> {
    sprites: [&'static Sprite; C],
    /// How many screen refreshes each frame in the animation lasts for. 1 == 60fps, 2 == 30fps etc
    tick_rate: u8,
}

pub struct LoadedAnimation<'a, const C: usize> {
    animation: &'a Animation<C>,
    loaded_sprites: [LoadedSprite<'a>; C],
}

pub struct AnimationController<'a, const C: usize> {
    loaded_animation: &'a LoadedAnimation<'a, C>,
    loaded_obj_entry: LoadedObjectEntry<'a>,
    tick_counter: u8,
    frame_number: usize,
}

pub struct LoadedObjectEntry<'a> {
    oam_entry: OAMEntry<'a>,
    sprite: &'a LoadedSprite<'a>,
}

pub struct AffineLoadedObjectEntry<'a> {
    loaded_object_entry: LoadedObjectEntry<'a>,
    affine_matrix: AffineMatrix<'a>,
}

impl Sprite {
    pub fn load<'a>(&'a self, gba: &'a GBA) -> LoadedSprite<'a> {
        let mut memory = gba
            .obj_tile_memory
            .request_memory(self.tiles.len())
            .expect("Out of VRAM.");

        let mem_region = memory.as_vol_region();

        for i in 0..self.tiles.len() {
            mem_region.index(i).write(self.tiles[i]);
        }

        LoadedSprite {
            sprite: self,
            memory,
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }
}

impl<'a> LoadedSprite<'a> {
    pub fn sprite(&'a self) -> &'a Sprite {
        self.sprite
    }

    pub fn create_obj_attr_entry(&'a self, gba: &'a GBA) -> LoadedObjectEntry<'a> {
        let oam_entry = gba.shadow_oam.request_memory().expect("Out of OBJRAM");

        let mut entry = LoadedObjectEntry {
            oam_entry,
            sprite: self,
        };

        self.store_in_obj_entry(&mut entry);

        entry
    }

    pub fn store_in_obj_entry(&self, obj_entry: &mut LoadedObjectEntry) {
        let oa = obj_entry.get_obj_attr_data();

        oa.0 = oa.0.with_bpp8(false).with_shape(self.sprite.shape);

        oa.1 = oa.1.with_size(self.sprite.size);

        oa.2 =
            oa.2.with_tile_id(self.memory.get_start().try_into().unwrap())
                .with_palbank(self.sprite.palette_bank.into());
    }
}

impl<'a> LoadedObjectEntry<'a> {
    pub fn get_obj_attr_data(&mut self) -> &mut ObjAttr {
        self.oam_entry.get_obj_attr()
    }

    pub fn loaded_sprite(&self) -> &LoadedSprite {
        self.sprite
    }

    pub fn into_affine(mut self, gba: &'a GBA) -> AffineLoadedObjectEntry<'a> {
        let affine_matrix = AffineMatrix::new(gba);

        let oa_data = self.get_obj_attr_data();
        oa_data.set_style(ObjDisplayStyle::DoubleSizeAffine);
        oa_data.1 = oa_data.1.with_affine_index(affine_matrix.index());

        AffineLoadedObjectEntry {
            loaded_object_entry: self,
            affine_matrix,
        }
    }

    pub fn with_visible(mut self) -> Self {
        self.set_visible();
        self
    }

    pub fn set_visible(&mut self) {
        let oa = self.get_obj_attr_data();
        oa.set_style(ObjDisplayStyle::Normal)
    }

    pub fn with_hidden(mut self) -> Self {
        self.set_hidden();
        self
    }

    pub fn set_hidden(&mut self) {
        let oa = self.get_obj_attr_data();
        oa.set_style(ObjDisplayStyle::NotDisplayed)
    }
}

impl<'a> AffineLoadedObjectEntry<'a> {
    pub fn into_normal_object_entry(self) -> LoadedObjectEntry<'a> {
        self.loaded_object_entry
    }

    pub fn get_affine_matrix<'b>(&'b mut self) -> &'b mut AffineMatrix<'a>
    where
        'a: 'b,
    {
        &mut self.affine_matrix
    }

    pub fn get_obj_attr_data(&mut self) -> &mut ObjAttr {
        self.loaded_object_entry.get_obj_attr_data()
    }

    pub fn loaded_sprite(&self) -> &LoadedSprite {
        self.loaded_object_entry.sprite
    }
}

impl<const C: usize> Animation<C> {
    pub fn load<'a>(&'a self, gba: &'a GBA) -> LoadedAnimation<'a, C> {
        let loaded_sprites = core::array::from_fn(|i| self.sprites[i].load(gba));
        LoadedAnimation {
            animation: self,
            loaded_sprites,
        }
    }
}

impl<'a, const C: usize> LoadedAnimation<'a, C> {
    pub fn get_sprite(&self, time: u16) -> &LoadedSprite {
        // Convert from time to the sprite index by dividing by tick rate.
        let num_frames: u16 = C.try_into().unwrap();
        let tick_rate: u16 = self.animation.tick_rate.into();
        let index: u16 = (time / tick_rate) % num_frames;

        // This will definitely fit into a usize, because the mod guarantees
        // it's <= C, which is a usize.
        let index: usize = index.into();

        self.get_frame(index)
    }

    pub fn get_frame(&self, index: usize) -> &LoadedSprite {
        &self.loaded_sprites[index]
    }

    pub fn create_controller(&'a self, gba: &'a GBA) -> AnimationController<'a, C> {
        AnimationController::new(self, gba)
    }
}

impl<'a, const C: usize> AnimationController<'a, C> {
    fn new(loaded_animation: &'a LoadedAnimation<'a, C>, gba: &'a GBA) -> Self {
        let first_frame = &loaded_animation.loaded_sprites[0];
        let mut loaded_obj_entry = first_frame.create_obj_attr_entry(gba);

        let oa = loaded_obj_entry.get_obj_attr_data();
        oa.0 = oa.0.with_style(ObjDisplayStyle::Normal);

        Self {
            loaded_animation,
            loaded_obj_entry,
            tick_counter: 0,
            frame_number: 0,
        }
    }

    pub fn get_obj_attr_entry<'b>(&'b mut self) -> &'b mut LoadedObjectEntry<'a>
    where
        'a: 'b,
    {
        let sprite = &self.loaded_animation.loaded_sprites[self.frame_number];
        sprite.store_in_obj_entry(&mut self.loaded_obj_entry);
        &mut self.loaded_obj_entry
    }

    pub fn tick(&mut self) {
        self.tick_counter += 1;

        if self.tick_counter == self.loaded_animation.animation.tick_rate {
            self.tick_counter = 0;
            self.frame_number = (self.frame_number + 1) % C;
        }

        // TODO - This doesn't read very well.
        // Update the obj attr
        _ = self.get_obj_attr_entry();
    }

    pub fn set_hidden(&mut self) {
        let obj_entry = self.get_obj_attr_entry();
        obj_entry.set_hidden();
    }

    pub fn set_visible(&mut self) {
        let obj_entry = self.get_obj_attr_entry();
        obj_entry.set_visible();
    }
}

// Insert all of the code generated by the build sript.
// This will contain static definitions for all of our aseprite files.
include!(concat!(env!("OUT_DIR"), "/sprite_data.rs"));
