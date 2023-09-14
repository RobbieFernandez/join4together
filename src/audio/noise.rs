use gba::{
    prelude::{
        LEFT_RIGHT_VOLUME, NOISE_FREQ, NOISE_LEN_ENV, TONE1_FREQUENCY, TONE1_PATTERN, TONE1_SWEEP,
    },
    sound::{
        LeftRightVolume, NoiseFrequency, NoiseLenEnvelope, SweepControl, ToneFrequency, TonePattern,
    },
};

pub fn enable_noise() {
    let vol = LeftRightVolume::new()
        .with_left_volume(u16::MAX)
        .with_right_volume(u16::MAX)
        .with_noise_left(true)
        .with_noise_right(true)
        .with_tone1_left(true)
        .with_tone1_right(true);

    LEFT_RIGHT_VOLUME.write(vol);
}

/// Plays a short "impact" noise.
/// enable_noise must be called in order for any noise to be heard.
pub fn play_impact_noise() {
    let freq = NoiseFrequency::new()
        .with_enabled(true)
        .with_stop_when_expired(true)
        .with_r(0b011) // Clock divider
        .with_s(0b0000); // Pre-step frequency

    let env = NoiseLenEnvelope::new()
        .with_step_increasing(false)
        .with_volume(0b0100)
        .with_length(1)
        .with_step_time(0b001);

    NOISE_LEN_ENV.write(env);
    NOISE_FREQ.write(freq);
}

pub fn play_menu_move_noise() {
    let freq = ToneFrequency::new()
        .with_enabled(true)
        .with_stop_when_expired(true)
        .with_frequency(0b111111111);

    let sweep = SweepControl::new()
        .with_sweep_increasing(false)
        .with_sweep_num(0b100)
        .with_sweep_time(0b001);

    let env = TonePattern::new()
        .with_step_increasing(false)
        .with_volume(0b1000)
        .with_step_time(0b001)
        .with_length(0x0001)
        .with_duty(0b10);

    TONE1_SWEEP.write(sweep);
    TONE1_PATTERN.write(env);
    TONE1_FREQUENCY.write(freq);
}
