use anyhow::{bail, Context, Result};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use ::zip::ZipArchive;

use crate::formats::{bmp, detect_image_kind, gif, jpeg, png, wav, webp, ImageKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionTarget {
    All,
    Image,
    Payload,
}

pub fn extract(
    source: &Path,
    output_dir: &Path,
    target: ExtractionTarget,
) -> Result<Vec<String>> {
    let data = fs::read(source)
        .with_context(|| format!("failed to read file '{}'", source.display()))?;

    fs::create_dir_all(output_dir)?;
    let mut extracted_paths = Vec::new();

    let kind = detect_image_kind(&data);
    let boundary = match kind {
        Some(ImageKind::Png) => png::locate_png_end(&data).ok(),
        Some(ImageKind::Jpeg) => jpeg::locate_jpeg_end(&data).ok(),
        Some(ImageKind::Gif) => gif::locate_gif_end(&data).ok(),
        Some(ImageKind::Webp) => webp::locate_webp_end(&data).ok(),
        Some(ImageKind::Bmp) => bmp::locate_bmp_end(&data).ok(),
        Some(ImageKind::Wav) => wav::locate_wav_end(&data).ok(),
        None => None,
    };

    if target == ExtractionTarget::All || target == ExtractionTarget::Image {
        if let (Some(k), Some(bound)) = (kind, boundary) {
            let ext = match k {
                ImageKind::Png => "png",
                ImageKind::Jpeg => "jpg",
                ImageKind::Gif => "gif",
                ImageKind::Webp => "webp",
                ImageKind::Bmp => "bmp",
                ImageKind::Wav => "wav",
            };

            let img_path = output_dir.join(format!("extracted_cover.{}", ext));
            fs::write(&img_path, &data[..bound])?;
            extracted_paths.push(img_path.display().to_string());
        } else if target == ExtractionTarget::Image {
            bail!("no valid image header detected in source file");
        }
    }

    if target == ExtractionTarget::All || target == ExtractionTarget::Payload {
        let cursor = Cursor::new(&data);
        match ZipArchive::new(cursor) {
            Ok(mut archive) => {
                let payload_dir = output_dir.join("payload");
                fs::create_dir_all(&payload_dir)?;

                for i in 0..archive.len() {
                    let mut file = archive.by_index(i)?;
                    let out_path = payload_dir.join(file.mangled_name());

                    if file.is_dir() {
                        fs::create_dir_all(&out_path)?;
                    } else {
                        if let Some(parent) = out_path.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        let mut outfile = fs::File::create(&out_path)?;
                        std::io::copy(&mut file, &mut outfile)?;
                        extracted_paths.push(out_path.display().to_string());
                    }
                }
            }
            Err(e) => {
                if target == ExtractionTarget::Payload {
                    bail!("failed to parse embedded ZIP payload: {}", e);
                }
            }
        }
    }

    Ok(extracted_paths)
}
