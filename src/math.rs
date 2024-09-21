use gba::fixed::i16fx8;

// Include the sine lookup table that is created by the build script.
include!(concat!(env!("OUT_DIR"), "/lut_data.rs"));

/// Get sin of theta.
/// Theta is a u16 where 0xFFFF represents 2π
pub fn sin(theta: u16) -> i16fx8 {
    // Get the top 7 bits of theta, ie limit it to 0 - 511
    // which is the size of our lookup table.
    let i: usize = ((theta >> 7) & 0x1FF).into();
    SINE_LOOKUP[i]
}

/// Get cos of theta.
/// Theta is a u16 where 0xFFFF represents 2π
pub fn cos(theta: u16) -> i16fx8 {
    // Same as sin but with an offset.
    let i: usize = (((theta >> 7) + 128) & 0x1FF).into();
    SINE_LOOKUP[i]
}
