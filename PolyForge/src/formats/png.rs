use anyhow::{bail, Result};

pub const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

pub fn is_png(data: &[u8]) -> bool {
    data.len() >= 8 && data[..8] == PNG_SIGNATURE
}

pub fn locate_png_end(data: &[u8]) -> Result<usize> {
    if !is_png(data) {
        bail!("data does not start with a valid PNG signature");
    }

    let mut cursor = 8;
    while cursor + 8 <= data.len() {
        let length = u32::from_be_bytes(data[cursor..cursor + 4].try_into()?) as usize;
        let chunk_type = &data[cursor + 4..cursor + 8];
        let next_cursor = cursor + 8 + length + 4;

        if next_cursor > data.len() {
            bail!("corrupted PNG chunk extends beyond buffer boundary");
        }

        if chunk_type == b"IEND" {
            return Ok(next_cursor);
        }

        cursor = next_cursor;
    }

    bail!("PNG IEND chunk not found");
}
