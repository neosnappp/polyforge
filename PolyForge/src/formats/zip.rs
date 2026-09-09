use anyhow::{bail, Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;
use walkdir::WalkDir;
use ::zip::write::SimpleFileOptions;
use ::zip::{CompressionMethod, ZipWriter};

pub const ZIP_LOCAL_HEADER_SIG: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
pub const ZIP_CENTRAL_DIR_SIG: [u8; 4] = [0x50, 0x4B, 0x01, 0x02];
pub const ZIP_EOCD_SIG: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];

#[derive(Debug, Clone)]
pub struct ZipMeta {
    pub total_entries: u16,
    pub cd_size: u32,
    pub cd_offset: u32,
    pub eocd_offset: usize,
}

pub fn is_zip(data: &[u8]) -> bool {
    data.len() >= 4 && data[..4] == ZIP_LOCAL_HEADER_SIG
}

pub fn find_eocd(data: &[u8]) -> Result<usize> {
    if data.len() < 22 {
        bail!("buffer too small to contain a ZIP EOCD record");
    }

    let search_window = data.len().saturating_sub(65535 + 22);
    for i in (search_window..=data.len() - 22).rev() {
        if data[i..i + 4] == ZIP_EOCD_SIG {
            let comment_len = u16::from_le_bytes([data[i + 20], data[i + 21]]) as usize;
            if i + 22 + comment_len <= data.len() {
                return Ok(i);
            }
        }
    }

    bail!("ZIP End of Central Directory (EOCD) signature not found");
}

pub fn parse_zip_meta(data: &[u8]) -> Result<ZipMeta> {
    let eocd_pos = find_eocd(data)?;
    let mut cursor = Cursor::new(&data[eocd_pos + 4..]);

    let _disk_number = cursor.read_u16::<LittleEndian>()?;
    let _cd_disk = cursor.read_u16::<LittleEndian>()?;
    let _entries_this_disk = cursor.read_u16::<LittleEndian>()?;
    let total_entries = cursor.read_u16::<LittleEndian>()?;
    let cd_size = cursor.read_u32::<LittleEndian>()?;
    let cd_offset = cursor.read_u32::<LittleEndian>()?;

    Ok(ZipMeta {
        total_entries,
        cd_size,
        cd_offset,
        eocd_offset: eocd_pos,
    })
}

pub fn adjust_zip_offsets(zip_bytes: &[u8], shift: u32) -> Result<Vec<u8>> {
    let mut modified = zip_bytes.to_vec();
    let meta = parse_zip_meta(&modified)?;

    let cd_start = meta.cd_offset as usize;
    let cd_end = cd_start + (meta.cd_size as usize);

    if cd_end > modified.len() {
        bail!("central directory spans beyond ZIP buffer");
    }

    let mut cursor = cd_start;
    while cursor + 46 <= cd_end {
        if modified[cursor..cursor + 4] != ZIP_CENTRAL_DIR_SIG {
            bail!("invalid central directory header signature at offset {}", cursor);
        }

        let file_name_len = u16::from_le_bytes([modified[cursor + 28], modified[cursor + 29]]) as usize;
        let extra_len = u16::from_le_bytes([modified[cursor + 30], modified[cursor + 31]]) as usize;
        let comment_len = u16::from_le_bytes([modified[cursor + 32], modified[cursor + 33]]) as usize;

        let current_offset = u32::from_le_bytes(
            modified[cursor + 42..cursor + 46]
                .try_into()
                .unwrap(),
        );

        let new_offset = current_offset.checked_add(shift).context("offset overflow")?;
        modified[cursor + 42..cursor + 46].copy_from_slice(&new_offset.to_le_bytes());

        cursor += 46 + file_name_len + extra_len + comment_len;
    }

    let new_cd_offset = meta
        .cd_offset
        .checked_add(shift)
        .context("CD offset overflow")?;

    let eocd_cd_offset_field = meta.eocd_offset + 16;
    modified[eocd_cd_offset_field..eocd_cd_offset_field + 4]
        .copy_from_slice(&new_cd_offset.to_le_bytes());

    Ok(modified)
}

pub fn create_zip_archive(path: &Path) -> Result<Vec<u8>> {
    let buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buffer);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    if path.is_file() {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("payload.bin");

        zip.start_file(file_name, options)?;
        let mut f = File::open(path)?;
        let mut content = Vec::new();
        f.read_to_end(&mut content)?;
        zip.write_all(&content)?;
    } else if path.is_dir() {
        let prefix = path.parent().unwrap_or(path);

        for entry in WalkDir::new(path) {
            let entry = entry?;
            let entry_path = entry.path();
            let relative_path = entry_path.strip_prefix(prefix)?;
            let relative_str = relative_path.to_str().unwrap_or_default().replace('\\', "/");

            if relative_str.is_empty() {
                continue;
            }

            if entry_path.is_file() {
                zip.start_file(&relative_str, options)?;
                let mut f = File::open(entry_path)?;
                let mut content = Vec::new();
                f.read_to_end(&mut content)?;
                zip.write_all(&content)?;
            } else if entry_path.is_dir() {
                zip.add_directory(&relative_str, options)?;
            }
        }
    } else {
        bail!("payload path does not exist or is neither a file nor a directory");
    }

    let mut cursor = zip.finish()?;
    cursor.seek(SeekFrom::Start(0))?;
    Ok(cursor.into_inner())
}
