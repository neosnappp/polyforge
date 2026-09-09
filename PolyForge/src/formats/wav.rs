use anyhow::{bail, Result};

pub fn is_wav(data: &[u8]) -> bool {
    data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WAVE"
}

pub fn locate_wav_end(data: &[u8]) -> Result<usize> {
    if !is_wav(data) {
        bail!("data does not start with a valid WAV RIFF header");
    }

    let riff_payload_size = u32::from_le_bytes(data[4..8].try_into()?) as usize;
    let mut total_size = 8 + riff_payload_size;

    if !total_size.is_multiple_of(2) {
        total_size += 1;
    }

    if total_size > data.len() {
        bail!("WAV RIFF length extends beyond buffer");
    }

    Ok(total_size)
}
