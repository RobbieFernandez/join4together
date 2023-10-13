use core::ffi::c_void;
use gba::prelude::*;

use crate::system::irq;

static mut MIXER: AudioMixer = AudioMixer {
    channels: [None, None],
    audio_buffers: AudioDoubleBuffer([0; AUDIO_BUFFER_SIZE * 2]),
    playing_second_buffer: false,
};

// We feed samples to the FIFO queues during vblank.
// Vblank occurs every 280896 CPU cycles.
// So the sample rate needs to be a multiple of 280896, we will use 18157 Hz.
// These values are pre-calculated based on the sample rate
//  For more details: https://deku.gbadev.org/program/sound1.html
const AUDIO_BUFFER_SIZE: usize = 304; // Note that this is not a multiple of 32, therefore the DMA writes must be done 16 bits at a time.
const AUDIO_TIMER_VALUE: u16 = 64612;

pub struct AudioVolume(u8);

pub struct AudioSource {
    samples: &'static [u8],
    position: usize,
    is_loop: bool,
    volume: AudioVolume,
}

struct AudioMixer {
    channels: [Option<AudioSource>; 2], // 2 Channels. 1 for music, 1 for sound effects
    audio_buffers: AudioDoubleBuffer,
    playing_second_buffer: bool,
}

struct AudioDoubleBuffer([u8; AUDIO_BUFFER_SIZE * 2]);

impl AudioVolume {
    pub fn new(value: u8) -> Self {
        if value > 63 {
            panic!("Volume may not exceed 63");
        }
        Self(value)
    }

    pub fn get(&self) -> u8 {
        self.0
    }
}

impl AudioSource {
    pub fn new(samples: &'static [u8], volume: AudioVolume, is_loop: bool) -> Self {
        Self {
            samples,
            is_loop,
            volume,
            position: 0,
        }
    }
}

impl AudioDoubleBuffer {
    fn first_buffer_mut<'a>(&'a mut self) -> &'a mut [u8] {
        self.0.split_at_mut(AUDIO_BUFFER_SIZE).0
    }

    fn second_buffer_mut<'a>(&'a mut self) -> &'a mut [u8] {
        self.0.split_at_mut(AUDIO_BUFFER_SIZE).1
    }
}

impl AudioMixer {
    fn set_channel_1(&mut self, source: AudioSource) {
        self.channels[0] = Some(source);
    }

    fn set_channel_2(&mut self, source: AudioSource) {
        self.channels[1] = Some(source);
    }

    fn swap_buffers(&mut self) {
        if self.playing_second_buffer {
            // Reset the DMA controller to point back at the first buffer.
            self.init_dma();
        }

        self.playing_second_buffer = !self.playing_second_buffer;
        self.get_back_buffer().fill(0);
    }

    fn get_back_buffer(&mut self) -> &mut [u8] {
        if self.playing_second_buffer {
            self.audio_buffers.first_buffer_mut()
        } else {
            self.audio_buffers.second_buffer_mut()
        }
    }

    fn fill_next_buffer(&mut self) {
        // The mixing happens in a critical section to guarantee we don't flip the buffers mid-update
        irq::critical_section(|| {
            for c in 0..=1 {
                let audio_source = &mut self.channels[c];

                // Copy the audio source into this buffer, then mix it into the back_buffer.
                let mut next_buffer = [0u8; AUDIO_BUFFER_SIZE];

                if let Some(ref mut audio_source) = audio_source {
                    let volume = audio_source.volume.get();

                    let next_source_position = audio_source.position + AUDIO_BUFFER_SIZE;

                    if next_source_position > audio_source.samples.len() {
                        // We've reached the end of the audio source.
                        // Copy the remaining samples from the source.
                        let remaining_samples = audio_source.samples.len() - audio_source.position;
                        next_buffer[0..remaining_samples].copy_from_slice(
                            &audio_source.samples
                                [audio_source.position..audio_source.samples.len()],
                        );

                        if audio_source.is_loop {
                            // If it's a looping sound, then go back to the start and copy needed samples to fill the audio buffer.
                            let overflowing_samples = AUDIO_BUFFER_SIZE - remaining_samples;
                            next_buffer[remaining_samples..AUDIO_BUFFER_SIZE]
                                .copy_from_slice(&audio_source.samples[0..overflowing_samples]);

                            audio_source.position = overflowing_samples;
                        } else {
                            // If it's not a loop then we can drop the audio source entirely.
                            self.channels[c] = None;
                        }
                    } else {
                        // We haven't yet reached the end of the source, so it's just a simple copy.
                        next_buffer.copy_from_slice(
                            &audio_source.samples[audio_source.position..next_source_position],
                        );
                        audio_source.position = next_source_position;
                    }

                    let back_buffer = self.get_back_buffer();

                    for (i, sample) in next_buffer.iter().enumerate() {
                        // The buffers are represented as u8, but the data is actually i8
                        // So reinterpret the data as i8 before these shifts.
                        let sample: i8 = i8::from_ne_bytes([*sample]);
                        let buffered: i8 = i8::from_ne_bytes([back_buffer[i]]);
                        let buffered: i16 = buffered.into();

                        // Store samples in i16 before mixing, then clip afterwards.
                        let mut sample: i16 = sample.into();
                        let volume: i16 = volume.into();
                        sample = sample * volume >> 6;

                        let mut mixed: i16 = buffered + sample;

                        // Clip to i8
                        mixed = mixed.clamp(i8::MIN.into(), i8::MAX.into());

                        // Reinterpret as u8 before writing it to the buffer.
                        let mixed: i8 = mixed.try_into().unwrap();
                        let mixed: u8 = mixed.to_ne_bytes()[0];

                        back_buffer[i] = mixed;
                    }
                }
            }
        });
    }

    fn init(&mut self) {
        self.init_dma();
        self.init_timer();
        self.init_sound_channel();
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

    fn init_sound_channel(&mut self) {
        // Initialise direct sound channel A, which will be used for music playback.
        let sound_mix = SoundMix::new()
            .with_sound_a_left(true)
            .with_sound_a_right(true)
            .with_sound_a_timer(false) // Use timer 0 for the sampling rate.
            .with_sound_a_full(true)
            .with_sound_a_reset(true);

        SOUND_MIX.write(sound_mix);
    }
}

pub fn init_mixer() {
    unsafe {
        MIXER.init();
    }
}

pub fn set_channel_1(source: AudioSource) {
    unsafe {
        MIXER.set_channel_1(source);
    }
}

pub fn set_channel_2(source: AudioSource) {
    unsafe {
        MIXER.set_channel_2(source);
    }
}

pub fn swap_buffers() {
    unsafe {
        MIXER.swap_buffers();
    }
}

pub fn fill_next_buffer() {
    unsafe {
        MIXER.fill_next_buffer();
    }
}
