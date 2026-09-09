# polyforge

```text
               _        __                    
 ___  ___  ___| |_  ___/ /__  _______ ____   
/ _ \/ _ \/ / // _ \/ _  / _ \/ __/ _ `/ -_)  
/ .__/\___/\_,_/_//_/\_,_/\___/_/  \_, /\__/   
/_/                               /___/        
```

Tired of `cat image.png secret.zip > out.png` breaking when someone runs standard Linux `unzip`?

`polyforge` is a tiny, zero-dependency Rust CLI that stitches valid images (PNG, JPEG, GIF, WebP, BMP) and audio (WAV) together with ZIP archives into clean dual-format polyglots. It patches ZIP Central Directory offsets on the fly so both strict archive managers and image viewers read the file without a single warning.

---

## Why this exists

If you just append a ZIP to an image using raw concatenation:
- Loose tools (WinRAR, 7-Zip) might open it fine because they scan backwards from the end of the file.
- Strict tools (`unzip -t` on Linux/macOS) will complain about bad offsets or garbage at the start of the file.

`polyforge` resolves the exact byte boundary where the image parser stops (e.g. `IEND` chunk in PNG, `0xFFD9` marker in JPEG, or RIFF bounds in WebP/WAV). It then recalculates every relative header offset in the ZIP's Central Directory and updates the EOCD record.

Result: One file. Opens as a normal picture/song in photo viewers. Opens as a valid archive in zip tools.

---

## Supported Formats

| Container | End-of-Stream Boundary |
|---|---|
| **PNG** | Trailing byte of `IEND` chunk |
| **JPEG** | `0xFFD9` (EOI marker) |
| **GIF** | `0x3B` (GIF trailer) |
| **WebP** | RIFF chunk header length |
| **BMP** | Header `biSize` byte offset |
| **WAV** | RIFF audio block bounds |

---

## Quickstart

### 1. Build from source

Requires a standard Rust toolchain:

```bash
cargo build --release
```

The optimized, stripped binary lands in `target/release/polyforge` (~750 KB).

---

### 2. Pack a polyglot

#### Merge an image and an existing archive:
```bash
polyforge pack -i photo.jpg -p secrets.zip -o innocent.jpg
```

#### Auto-compress a directory on the fly:
Don't have a `.zip` ready? Just pass the folder path. `polyforge` will deflate it into an archive automatically before stitching:

```bash
polyforge pack -i avatar.png -p ./my_vault/ -o avatar.png
```

---

### 3. Inspect a suspicious file

Check if an image actually hides an archive, check its byte boundary, and see what's inside without extracting:

```bash
polyforge inspect innocent.jpg
```

Output:
```text
[*] Binary Container Inspection Report
  File               : innocent.jpg
  Total Size         : 182,410 bytes
  Primary Header     : JPEG Image (boundary: 0x0..0x2B10)
  Payload Header     : Valid ZIP Archive (start: 0x2B10, entries: 2)

  Embedded Files:
    - credentials.txt                (compressed: 42 B, size: 55 B)
    - key.pem                        (compressed: 1120 B, size: 2400 B)

[+] Status: Container is a verified polyglot file.
```

---

### 4. Carve out components

Extract the payload files directly:
```bash
polyforge extract innocent.jpg -o ./dump --target payload
```

Or carve out the untouched original cover image:
```bash
polyforge extract innocent.jpg -o ./dump --target image
```

---

## How the binary layout looks

```text
+-------------------------------------------------------+
| [0x00] Image / Media Header                           |
|        Scanlines, metadata, palettes                  |
| [0x..] Format Terminator (IEND / EOI / 0x3B / RIFF)   |
+-------------------------------------------------------+ <- Boundary S
| [0xS]  ZIP Local File Header 1 (PK\x03\x04)          |
|        ...                                            |
|        ZIP Central Directory (PK\x01\x02)             |
|        * local_header_offset patched with (+S)        |
| [0x..] ZIP End of Central Directory (PK\x05\x06)      |
|        * central_dir_start_offset patched with (+S)   |
+-------------------------------------------------------+
```

---

## Project Structure

```text
src/
├── main.rs          # CLI runner and colored output
├── cli.rs           # Clap commands & flags
├── lib.rs           # Library exports
├── core/
│   ├── builder.rs   # Pipeline & offset relocation
│   ├── inspector.rs # Signature & boundary analyzer
│   └── extractor.rs # Payload carver
└── formats/
    ├── png.rs       # Chunk walker
    ├── jpeg.rs      # Marker parser
    ├── gif.rs       # Trailer locator
    ├── webp.rs      # RIFF parser
    ├── bmp.rs       # Header validator
    ├── wav.rs       # Audio bounds
    └── zip.rs       # EOCD parser & offset math
```

---

## Disclaimer

This tool is designed for educational, steganography research, and security CTF purposes.

---

## License

MIT
