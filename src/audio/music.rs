use core::ffi::c_void;

use gba::prelude::*;

static BACKGROUND_MUSIC: &[u8] = include_bytes!("bgm.raw");
static NUM_SAMPLES: u32 = BACKGROUND_MUSIC.len() as u32;
static TIMER1_VALUE: GbaCell<u16> = GbaCell::new(0);

const SAMPLE_RATE: u16 = 16000;
const SAMPLE_TIMER_STEPS: u32 = 2u32.pow(24) / SAMPLE_RATE as u32;
const SAMPLE_TIMER_VALUE: u16 = 0xFFFF - (SAMPLE_TIMER_STEPS as u16);

pub fn timer2_interrupt() {
    // The overflow counter is finished.
    // We can now set TIMER1 to the remainder and enable the IRQ.
    TIMER1_CONTROL.write(TimerControl::new().with_enabled(false));
    TIMER1_RELOAD.write(TIMER1_VALUE.read());

    TIMER1_CONTROL.write(
        TimerControl::new()
            .with_enabled(true)
            .with_cascade(true)
            .with_overflow_irq(true)
            .with_scale(TimerScale::_1),
    );
    TIMER2_CONTROL.write(TimerControl::new().with_enabled(false));
}

pub fn timer1_interrupt() {
    // The timer 1 interrupt fires when its time to loop the song.
    // First stop the music by disabling the timers and DMA
    // TIMER0_CONTROL.write(TimerControl::new().with_enabled(false));
    TIMER1_CONTROL.write(TimerControl::new().with_enabled(false));
    TIMER2_CONTROL.write(TimerControl::new().with_enabled(false));

    unsafe {
        DMA1_CONTROL.write(DmaControl::new().with_enabled(false));
    }

    // Now just re-initialize the music to restart it.
    start_music();
}

pub fn init_music() {
    // Initialise direct sound channel A, which will be used for music playback.
    let sound_mix = SoundMix::new()
        .with_sound_a_left(true)
        .with_sound_a_right(true)
        .with_sound_a_timer(false) // Use timer 0 for the sampling rate.
        .with_sound_a_full(false)
        .with_sound_a_reset(true);

    SOUND_MIX.write(sound_mix);

    // Enable the sound chip.
    SOUND_ENABLED.write(SoundEnable::new().with_enabled(true));

    // Set up the timer that will be used as the sample rate.
    TIMER0_RELOAD.write(SAMPLE_TIMER_VALUE);
    TIMER0_CONTROL.write(
        TimerControl::new()
            .with_enabled(true)
            .with_scale(TimerScale::_1),
    );

    start_music();
}

fn start_music() {
    // Setup timers for tracking the song progress.
    init_music_timers();
    // Setup the DMA unit to copy samples into
    let dma_control = DmaControl::new()
        .with_src_addr_control(SrcAddrControl::Increment)
        .with_dest_addr_control(DestAddrControl::Increment)
        .with_start_time(DmaStartTime::Special)
        .with_transfer_32bit(true)
        .with_enabled(true)
        .with_repeat(true);

    unsafe {
        let music_addr = BACKGROUND_MUSIC as *const [u8];
        DMA1_SRC.write(music_addr as *const c_void);
        DMA1_DEST.write(FIFO_A.as_mut_ptr() as *mut c_void);
        DMA1_CONTROL.write(dma_control);
    }
}

fn init_music_timers() {
    // Enable Timer 1 to track where we are in the song.
    // This will be used to restart the song once it's done, so that we don't play back garbage.

    // The GBA counters are 16 bit.
    // So if the song contains more than 2^16 samples, then we have to chain 2 timers together
    // to be able to tell when it finishes.
    // We will set TIMER1 to 0, and TIMER2 to 0xFFFF minus the top 16 bits of the sample length.
    // TIMER2 will be set to cascade from TIMER1, so it counts the "overflow"
    // We will store the bottom 16 bits in a global variable, and when TIMER2's overflow interrupt
    // is triggered, we can set TIMER1 to 0xFFFF minus those 16 bits. This representing the remainder
    // after the overflow counter has finished.
    // Finally when TIMER1's interrupt is called, we know the song is over and we can loop it.
    if NUM_SAMPLES > 0xFFFF {
        let sample_count_bottom_bits = NUM_SAMPLES as u16;
        let sample_count_top_bits = (NUM_SAMPLES >> 16) as u16;

        let timer_1_reload = 0xFFFF - sample_count_bottom_bits + 1;
        TIMER1_VALUE.write(timer_1_reload);

        let timer_2_reload = 0xFFFF - sample_count_top_bits + 1;

        TIMER2_RELOAD.write(timer_2_reload);
        TIMER2_CONTROL.write(
            TimerControl::new()
                .with_enabled(true)
                .with_cascade(true)
                .with_overflow_irq(true)
                .with_scale(TimerScale::_1),
        );

        TIMER1_RELOAD.write(0);
        TIMER1_CONTROL.write(
            TimerControl::new()
                .with_enabled(true)
                .with_cascade(true)
                .with_scale(TimerScale::_1),
        );
    } else {
        // If the song is short enough then we only have to use TIMER1 and don't need
        // any overflow logic.
        TIMER1_CONTROL.write(
            TimerControl::new()
                .with_enabled(true)
                .with_cascade(true)
                .with_overflow_irq(true)
                .with_scale(TimerScale::_1),
        );
        let timer_1_reload = 0xFFFF - (NUM_SAMPLES as u16);
        TIMER1_RELOAD.write(timer_1_reload);
    }
}
