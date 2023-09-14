use std::{path::Path, fs::{read_dir, File}, io::Read};

pub mod codegen;

#[derive(Debug)]
pub struct Sound {
    filename: String,
    bytes: Vec<u8>
}

#[derive(Debug)]
pub struct SoundError;

pub fn find_music(directory: &Path) -> Result<Vec<Sound>, SoundError> {
    let entries = read_dir(directory).map_err(|_e| SoundError)?;
    let mut sounds: Vec<Sound> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|_e| SoundError)?;
        let path = entry.path();
        let file = File::open(path);

        let filename = entry
            .file_name()
            .to_ascii_uppercase()
            .into_string()
            .map_err(|_e| SoundError)?
            .replace(".RAW", "");


        let mut file = file.map_err(|_e| SoundError)?;
        let mut sound_bytes: Vec<u8> = Vec::new();

        let _ = file.read_to_end(&mut sound_bytes).map_err(|_e| SoundError);
        let sound = Sound { filename, bytes: sound_bytes  };
        sounds.push(sound);
    }

    Ok(sounds)
}