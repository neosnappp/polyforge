use anyhow::{bail, Result};

pub fn is_gif(data: &[u8]) -> bool {
    data.len() >= 6 && (&data[..6] == b"GIF87a" || &data[..6] == b"GIF89a")
}

pub fn locate_gif_end(data: &[u8]) -> Result<usize> {
    if !is_gif(data) {
        bail!("data does not start with a valid GIF signature");
    }

    if let Some(pos) = data.iter().rposition(|&b| b == 0x3B) {
        return Ok(pos + 1);
    }

    bail!("GIF trailer byte (0x3B) not found");
}
