use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

use crate::formats::{bmp, detect_image_kind, gif, jpeg, png, wav, webp, zip, ImageKind};

#[derive(Debug)]
pub struct BuildResult {
    pub image_kind: ImageKind,
    pub image_size: usize,
    pub zip_size: usize,
    pub total_size: usize,
    pub entries_count: u16,
    pub output_path: String,
}

pub struct PolyglotBuilder<'a> {
    image_path: &'a Path,
    payload_path: &'a Path,
    output_path: &'a Path,
    adjust_offsets: bool,
}

impl<'a> PolyglotBuilder<'a> {
    pub fn new(image_path: &'a Path, payload_path: &'a Path, output_path: &'a Path) -> Self {
        Self {
            image_path,
            payload_path,
            output_path,
            adjust_offsets: true,
        }
    }

    pub fn with_offset_adjustment(mut self, adjust: bool) -> Self {
        self.adjust_offsets = adjust;
        self
    }

    pub fn build(self) -> Result<BuildResult> {
        let image_raw = fs::read(self.image_path)
            .with_context(|| format!("failed to read image file '{}'", self.image_path.display()))?;

        let kind = detect_image_kind(&image_raw)
            .context("unrecognized format (expected PNG, JPEG, GIF, WebP, BMP, or WAV)")?;

        let image_boundary = match kind {
            ImageKind::Png => png::locate_png_end(&image_raw)?,
            ImageKind::Jpeg => jpeg::locate_jpeg_end(&image_raw)?,
            ImageKind::Gif => gif::locate_gif_end(&image_raw)?,
            ImageKind::Webp => webp::locate_webp_end(&image_raw)?,
            ImageKind::Bmp => bmp::locate_bmp_end(&image_raw)?,
            ImageKind::Wav => wav::locate_wav_end(&image_raw)?,
        };

        let clean_image = &image_raw[..image_boundary];

        let payload_bytes = if self.payload_path.is_file() {
            let candidate = fs::read(self.payload_path)?;
            if zip::is_zip(&candidate) {
                candidate
            } else {
                zip::create_zip_archive(self.payload_path)?
            }
        } else if self.payload_path.is_dir() {
            zip::create_zip_archive(self.payload_path)?
        } else {
            bail!("payload path does not exist: '{}'", self.payload_path.display());
        };

        let final_zip = if self.adjust_offsets {
            zip::adjust_zip_offsets(&payload_bytes, clean_image.len() as u32)?
        } else {
            payload_bytes
        };

        let meta = zip::parse_zip_meta(&final_zip)?;

        let mut output_buffer = Vec::with_capacity(clean_image.len() + final_zip.len());
        output_buffer.extend_from_slice(clean_image);
        output_buffer.extend_from_slice(&final_zip);

        if let Some(parent) = self.output_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(self.output_path, &output_buffer)?;

        Ok(BuildResult {
            image_kind: kind,
            image_size: clean_image.len(),
            zip_size: final_zip.len(),
            total_size: output_buffer.len(),
            entries_count: meta.total_entries,
            output_path: self.output_path.display().to_string(),
        })
    }
}
