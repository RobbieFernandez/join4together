use core::ffi::c_void;
use gba::prelude::*;

static BACKGROUND_MUSIC: &[u8] = include_bytes!("bgm.raw");
static MUSIC_PLAYER_TAKEN: GbaCell<bool> = GbaCell::new(false);

// We feed samples to the FIFO queues during vblank.
// Vblank occurs every 280896 CPU cycles.
// So the sample rate needs to be a multiple of 280896
//  For more details: https://deku.gbadev.org/program/sound1.html
const SAMPLE_RATE: u32 = 18157;

// These values are pre-calculated based on the sample rate
//  For more details: https://deku.gbadev.org/program/sound1.html
const AUDIO_BUFFER_SIZE: usize = 304; // Note that this is not a multiple of 32, therefore the DMA writes must be done 16 bits at a time.
const AUDIO_TIMER_VALUE: u16 = 64612;

struct AudioDoubleBuffer([u8; AUDIO_BUFFER_SIZE * 2]);

impl AudioDoubleBuffer {
    fn first_buffer_mut<'a>(&'a mut self) -> &'a mut [u8] {
        self.0.split_at_mut(AUDIO_BUFFER_SIZE).0
    }

    fn second_buffer_mut<'a>(&'a mut self) -> &'a mut [u8] {
        self.0.split_at_mut(AUDIO_BUFFER_SIZE).1
    }
}

pub struct MusicPlayer {
    audio_buffers: AudioDoubleBuffer,
    playing_second_buffer: bool,
    music_source: &'static [u8],
    source_position: usize,
}

impl MusicPlayer {
    pub fn take() -> Self {
        if MUSIC_PLAYER_TAKEN.read() {
            panic!("Music player instance already exists.");
        }

        MUSIC_PLAYER_TAKEN.write(true);

        let mut player = Self {
            audio_buffers: AudioDoubleBuffer([0; AUDIO_BUFFER_SIZE * 2]),
            playing_second_buffer: false,
            music_source: BACKGROUND_MUSIC,
            source_position: 0,
        };

        player.init_dma();
        player.init_timer();

        player
    }

    fn init_dma(&mut self) {
        // Setup the DMA unit to copy samples into
        let dma_control = DmaControl::new()
            .with_src_addr_control(SrcAddrControl::Increment)
            .with_dest_addr_control(DestAddrControl::Increment)
            .with_start_time(DmaStartTime::Special)
            .with_transfer_32bit(false)
            .with_enabled(true)
            .with_repeat(true);

        unsafe {
            let music_addr = &self.audio_buffers.0 as *const [u8];
            DMA1_CONTROL.write(DmaControl::new().with_enabled(false));
            DMA1_SRC.write(music_addr as *const c_void);
            DMA1_DEST.write(FIFO_A.as_mut_ptr() as *mut c_void);
            DMA1_CONTROL.write(dma_control);
        }
    }

    fn init_timer(&mut self) {
        TIMER0_RELOAD.write(AUDIO_TIMER_VALUE);
        TIMER0_CONTROL.write(TimerControl::new().with_enabled(true));
    }

    pub fn swap_buffers(&mut self) {
        if self.playing_second_buffer {
            // Reset the DMA controller to point back at the first buffer.
            self.init_dma();
        }

        self.playing_second_buffer = !self.playing_second_buffer;
    }

    pub fn fill_next_buffer(&mut self) {
        let back_buffer = if self.playing_second_buffer {
            self.audio_buffers.first_buffer_mut()
        } else {
            self.audio_buffers.second_buffer_mut()
        };

        let next_source_position = self.source_position + AUDIO_BUFFER_SIZE;

        if next_source_position > self.music_source.len() {
            // Loop time !
            // Copy the remaining samples in the source, plus needed samples from the start to fill the buffer.
            // Then update the source position.
            let remaining_samples = self.music_source.len() - self.source_position;
            let overflowing_samples = AUDIO_BUFFER_SIZE - remaining_samples;

            back_buffer[0..remaining_samples]
                .copy_from_slice(&self.music_source[self.source_position..self.music_source.len()]);

            back_buffer[remaining_samples..AUDIO_BUFFER_SIZE]
                .copy_from_slice(&self.music_source[0..overflowing_samples]);

            self.source_position = overflowing_samples;
        } else {
            back_buffer
                .copy_from_slice(&self.music_source[self.source_position..next_source_position]);
            self.source_position = next_source_position;
        }
    }

    pub fn start(&mut self) {
        // Initialise direct sound channel A, which will be used for music playback.
        let sound_mix = SoundMix::new()
            .with_sound_a_left(true)
            .with_sound_a_right(true)
            .with_sound_a_timer(false) // Use timer 0 for the sampling rate.
            .with_sound_a_full(true)
            .with_sound_a_reset(true);

        SOUND_MIX.write(sound_mix);
    }

    pub fn stop(&mut self) {
        SOUND_MIX.write(
            SoundMix::new()
                .with_sound_a_left(false)
                .with_sound_a_right(false)
                .with_sound_a_reset(true),
        );
    }
}
