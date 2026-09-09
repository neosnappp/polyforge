use anyhow::{bail, Result};

pub fn is_bmp(data: &[u8]) -> bool {
    data.len() >= 14 && &data[..2] == b"BM"
}

pub fn locate_bmp_end(data: &[u8]) -> Result<usize> {
    if !is_bmp(data) {
        bail!("data does not start with a valid BMP header");
    }

    let file_size = u32::from_le_bytes(data[2..6].try_into()?) as usize;
    if file_size < 14 || file_size > data.len() {
        bail!("corrupted or invalid BMP file size header");
    }

    Ok(file_size)
}
