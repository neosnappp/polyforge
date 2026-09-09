pub mod bmp;
pub mod gif;
pub mod jpeg;
pub mod png;
pub mod wav;
pub mod webp;
pub mod zip;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageKind {
    Png,
    Jpeg,
    Gif,
    Webp,
    Bmp,
    Wav,
}

pub fn detect_image_kind(data: &[u8]) -> Option<ImageKind> {
    if png::is_png(data) {
        Some(ImageKind::Png)
    } else if jpeg::is_jpeg(data) {
        Some(ImageKind::Jpeg)
    } else if gif::is_gif(data) {
        Some(ImageKind::Gif)
    } else if webp::is_webp(data) {
        Some(ImageKind::Webp)
    } else if bmp::is_bmp(data) {
        Some(ImageKind::Bmp)
    } else if wav::is_wav(data) {
        Some(ImageKind::Wav)
    } else {
        None
    }
}
