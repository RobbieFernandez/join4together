use gba::{
    prelude::{
        LEFT_RIGHT_VOLUME, NOISE_FREQ, NOISE_LEN_ENV,
    },
    sound::{
        LeftRightVolume, NoiseFrequency, NoiseLenEnvelope, 
    },
};

pub fn enable_noise() {
    let vol = LeftRightVolume::new()
        .with_left_volume(u16::MAX)
        .with_right_volume(u16::MAX)
        .with_noise_left(true)
        .with_noise_right(true);

    LEFT_RIGHT_VOLUME.write(vol);
}

/// Plays a short "impact" noise.
/// enable_noise must be called in order for any noise to be heard.
pub fn play_impact_noise() {
    let freq = NoiseFrequency::new()
        .with_enabled(true)
        .with_stop_when_expired(false)
        .with_r(0b011)
        .with_s(0b0000);

    let env = NoiseLenEnvelope::new()
        .with_step_increasing(false)
        .with_volume(0b0011)
        .with_step_time(0b001);

    NOISE_LEN_ENV.write(env);
    NOISE_FREQ.write(freq);
}
