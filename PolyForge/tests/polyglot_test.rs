use std::fs;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use polyforge::core::builder::PolyglotBuilder;
use polyforge::core::extractor::{extract, ExtractionTarget};
use polyforge::core::inspector::inspect_file;
use polyforge::formats::png::is_png;
use polyforge::formats::ImageKind;

fn create_dummy_png() -> Vec<u8> {
    let mut png = Vec::new();
    png.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    let ihdr_data = [
        0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00,
    ];
    let mut ihdr_chunk = Vec::new();
    ihdr_chunk.extend_from_slice(&(ihdr_data.len() as u32).to_be_bytes());
    ihdr_chunk.extend_from_slice(b"IHDR");
    ihdr_chunk.extend_from_slice(&ihdr_data);
    let ihdr_crc = crc32fast::hash(&[b"IHDR", &ihdr_data[..]].concat());
    ihdr_chunk.extend_from_slice(&ihdr_crc.to_be_bytes());
    png.extend_from_slice(&ihdr_chunk);

    let idat_data = [0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01];
    let mut idat_chunk = Vec::new();
    idat_chunk.extend_from_slice(&(idat_data.len() as u32).to_be_bytes());
    idat_chunk.extend_from_slice(b"IDAT");
    idat_chunk.extend_from_slice(&idat_data);
    let idat_crc = crc32fast::hash(&[b"IDAT", &idat_data[..]].concat());
    idat_chunk.extend_from_slice(&idat_crc.to_be_bytes());
    png.extend_from_slice(&idat_chunk);

    let mut iend_chunk = Vec::new();
    iend_chunk.extend_from_slice(&0u32.to_be_bytes());
    iend_chunk.extend_from_slice(b"IEND");
    let iend_crc = crc32fast::hash(b"IEND");
    iend_chunk.extend_from_slice(&iend_crc.to_be_bytes());
    png.extend_from_slice(&iend_chunk);

    png
}

fn create_dummy_payload(file_name: &str, content: &[u8]) -> Vec<u8> {
    let buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buffer);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored);

    zip.start_file(file_name, options).unwrap();
    zip.write_all(content).unwrap();

    let cursor = zip.finish().unwrap();
    cursor.into_inner()
}

#[test]
fn test_png_zip_polyglot_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!("polyforge_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let img_path = temp_dir.join("test.png");
    let payload_path = temp_dir.join("secret.zip");
    let output_path = temp_dir.join("polyglot.png");
    let extract_dir = temp_dir.join("extracted");

    let raw_png = create_dummy_png();
    let raw_secret = b"classified_data_for_polyforge_test";
    let raw_zip = create_dummy_payload("secret.txt", raw_secret);

    fs::write(&img_path, &raw_png).unwrap();
    fs::write(&payload_path, &raw_zip).unwrap();

    let builder = PolyglotBuilder::new(&img_path, &payload_path, &output_path)
        .with_offset_adjustment(true);

    let res = builder.build().expect("build failed");
    assert_eq!(res.image_kind, ImageKind::Png);
    assert_eq!(res.entries_count, 1);

    let poly_bytes = fs::read(&output_path).unwrap();
    assert!(is_png(&poly_bytes));

    let cursor = Cursor::new(&poly_bytes);
    let mut archive = ZipArchive::new(cursor).expect("failed to open polyglot as zip");
    assert_eq!(archive.len(), 1);
    let mut file = archive.by_name("secret.txt").expect("file not found in polyglot zip");
    let mut read_content = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut read_content).unwrap();
    assert_eq!(read_content, raw_secret);

    let report = inspect_file(&output_path).expect("inspect failed");
    assert!(report.is_polyglot);
    assert_eq!(report.detected_image, Some(ImageKind::Png));
    assert!(report.zip_found);
    assert_eq!(report.entries.len(), 1);
    assert_eq!(report.entries[0].name, "secret.txt");

    let extracted = extract(&output_path, &extract_dir, ExtractionTarget::All).expect("extract failed");
    assert!(!extracted.is_empty());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_jpeg_zip_polyglot_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!("polyforge_jpeg_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let img_path = temp_dir.join("test.jpg");
    let payload_path = temp_dir.join("doc.txt");
    let output_path = temp_dir.join("polyglot.jpg");

    let mut dummy_jpeg = vec![0xFF, 0xD8];
    dummy_jpeg.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x10]);
    dummy_jpeg.extend_from_slice(b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0");
    dummy_jpeg.extend_from_slice(&[0xFF, 0xD9]);

    fs::write(&img_path, &dummy_jpeg).unwrap();
    fs::write(&payload_path, b"embedded plain text payload").unwrap();

    let builder = PolyglotBuilder::new(&img_path, &payload_path, &output_path)
        .with_offset_adjustment(true);

    let res = builder.build().expect("build failed");
    assert_eq!(res.image_kind, ImageKind::Jpeg);

    let report = inspect_file(&output_path).expect("inspect failed");
    assert!(report.is_polyglot);
    assert_eq!(report.detected_image, Some(ImageKind::Jpeg));
    assert!(report.zip_found);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_directory_payload_auto_zip() {
    let temp_dir = std::env::temp_dir().join(format!("polyforge_dir_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let payload_dir = temp_dir.join("vault");
    fs::create_dir_all(&payload_dir).unwrap();
    fs::write(payload_dir.join("key.pem"), b"RSA_TEST_KEY").unwrap();
    fs::write(payload_dir.join("config.json"), b"{\"active\":true}").unwrap();

    let img_path = temp_dir.join("avatar.png");
    fs::write(&img_path, create_dummy_png()).unwrap();

    let output_path = temp_dir.join("vault_avatar.png");

    let builder = PolyglotBuilder::new(&img_path, &payload_dir, &output_path);
    let res = builder.build().expect("build failed");
    assert_eq!(res.entries_count, 3);

    let report = inspect_file(&output_path).expect("inspect failed");
    assert_eq!(report.zip_entries_count, 3);

    let extract_dir = temp_dir.join("extracted");
    let extracted = extract(&output_path, &extract_dir, ExtractionTarget::Payload).expect("extract failed");
    assert_eq!(extracted.len(), 2);

    let _ = fs::remove_dir_all(&temp_dir);
}
