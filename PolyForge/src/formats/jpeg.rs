use anyhow::{bail, Result};

pub const JPEG_SOI: [u8; 2] = [0xFF, 0xD8];
pub const JPEG_EOI: [u8; 2] = [0xFF, 0xD9];

pub fn is_jpeg(data: &[u8]) -> bool {
    data.len() >= 2 && data[..2] == JPEG_SOI
}

pub fn locate_jpeg_end(data: &[u8]) -> Result<usize> {
    if !is_jpeg(data) {
        bail!("data does not start with a valid JPEG SOI marker");
    }

    let mut cursor = 2;
    while cursor < data.len() - 1 {
        if data[cursor] == 0xFF {
            let marker = data[cursor + 1];
            if marker == 0xD9 {
                return Ok(cursor + 2);
            }
            if marker == 0xDA {
                cursor += 2;
                while cursor < data.len() - 1 {
                    if data[cursor] == 0xFF && data[cursor + 1] != 0x00 && !(0xD0..=0xD7).contains(&data[cursor + 1]) && data[cursor + 1] == 0xD9 {
                        return Ok(cursor + 2);
                    }
                    cursor += 1;
                }
                break;
            } else if marker != 0x00 && marker != 0xFF && cursor + 4 <= data.len() {
                let length = u16::from_be_bytes([data[cursor + 2], data[cursor + 3]]) as usize;
                cursor += 2 + length;
                continue;
            }
        }
        cursor += 1;
    }

    if let Some(pos) = data.windows(2).rposition(|w| w == JPEG_EOI) {
        return Ok(pos + 2);
    }

    bail!("JPEG EOI marker not found");
}
