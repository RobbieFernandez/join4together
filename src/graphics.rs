enum SpriteBitDepth {
    FourBPP,
    EightBPP
}

struct Sprite<const T: usize> {
    depth: SpriteBitDepth,
    tile_data: []
}