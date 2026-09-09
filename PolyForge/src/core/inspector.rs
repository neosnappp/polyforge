use anyhow::{Context, Result};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use ::zip::ZipArchive;

use crate::formats::{bmp, detect_image_kind, gif, jpeg, png, wav, webp, zip, ImageKind};

#[derive(Debug)]
pub struct ArchiveEntryInfo {
    pub name: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
}

#[derive(Debug)]
pub struct InspectionReport {
    pub file_path: String,
    pub total_size: usize,
    pub detected_image: Option<ImageKind>,
    pub image_boundary: Option<usize>,
    pub zip_found: bool,
    pub zip_start_offset: Option<usize>,
    pub zip_entries_count: usize,
    pub entries: Vec<ArchiveEntryInfo>,
    pub slack_space_bytes: usize,
    pub is_polyglot: bool,
}

pub fn inspect_file(path: &Path) -> Result<InspectionReport> {
    let data = fs::read(path)
        .with_context(|| format!("failed to read target file '{}'", path.display()))?;

    let total_size = data.len();
    let detected_image = detect_image_kind(&data);

    let image_boundary = match detected_image {
        Some(ImageKind::Png) => png::locate_png_end(&data).ok(),
        Some(ImageKind::Jpeg) => jpeg::locate_jpeg_end(&data).ok(),
        Some(ImageKind::Gif) => gif::locate_gif_end(&data).ok(),
        Some(ImageKind::Webp) => webp::locate_webp_end(&data).ok(),
        Some(ImageKind::Bmp) => bmp::locate_bmp_end(&data).ok(),
        Some(ImageKind::Wav) => wav::locate_wav_end(&data).ok(),
        None => None,
    };

    let mut zip_found = false;
    let mut zip_start_offset = None;
    let mut zip_entries_count = 0;
    let mut entries = Vec::new();

    if let Ok(meta) = zip::parse_zip_meta(&data) {
        zip_found = true;
        zip_entries_count = meta.total_entries as usize;

        if let Some(pos) = data.windows(4).position(|w| w == zip::ZIP_LOCAL_HEADER_SIG) {
            zip_start_offset = Some(pos);
        }

        let cursor = Cursor::new(&data);
        if let Ok(mut archive) = ZipArchive::new(cursor) {
            for i in 0..archive.len() {
                if let Ok(file) = archive.by_index(i) {
                    entries.push(ArchiveEntryInfo {
                        name: file.name().to_string(),
                        compressed_size: file.compressed_size(),
                        uncompressed_size: file.size(),
                    });
                }
            }
        }
    }

    let slack_space_bytes = match (image_boundary, zip_start_offset) {
        (Some(img_end), Some(zip_start)) if zip_start > img_end => zip_start - img_end,
        _ => 0,
    };

    let is_polyglot = detected_image.is_some() && zip_found;

    Ok(InspectionReport {
        file_path: path.display().to_string(),
        total_size,
        detected_image,
        image_boundary,
        zip_found,
        zip_start_offset,
        zip_entries_count,
        entries,
        slack_space_bytes,
        is_polyglot,
    })
}
