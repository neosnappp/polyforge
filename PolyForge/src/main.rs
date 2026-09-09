use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::process;

use polyforge::cli::{Cli, Commands, ExtractTargetArg};
use polyforge::core::builder::PolyglotBuilder;
use polyforge::core::extractor::{extract, ExtractionTarget};
use polyforge::core::inspector::inspect_file;
use polyforge::formats::ImageKind;

fn main() {
    let args = Cli::parse();

    if let Err(err) = run(args) {
        eprintln!("{} {}", "[-] Error:".red().bold(), err);
        process::exit(1);
    }
}

fn run(args: Cli) -> Result<()> {
    match args.command {
        Commands::Pack {
            image,
            payload,
            output,
            no_adjust,
        } => {
            println!(
                "{} Synthesizing polyglot container...",
                "[*]".cyan().bold()
            );

            let builder = PolyglotBuilder::new(&image, &payload, &output)
                .with_offset_adjustment(!no_adjust);

            let res = builder.build()?;

            let kind_label = match res.image_kind {
                ImageKind::Png => "PNG Image",
                ImageKind::Jpeg => "JPEG Image",
                ImageKind::Gif => "GIF Image",
                ImageKind::Webp => "WebP Image",
                ImageKind::Bmp => "BMP Image",
                ImageKind::Wav => "WAV Audio",
            };

            println!("{}", "[+] Polyglot build completed successfully!".green().bold());
            println!("  {:<18} : {}", "Target".dimmed(), res.output_path.bold());
            println!("  {:<18} : {}", "Cover Format".dimmed(), kind_label);
            println!("  {:<18} : {} bytes", "Cover Size".dimmed(), res.image_size);
            println!("  {:<18} : {} bytes ({} entries)", "Payload Size".dimmed(), res.zip_size, res.entries_count);
            println!("  {:<18} : {} bytes", "Total Container".dimmed(), res.total_size);
            println!("  {:<18} : {}", "Offset Realignment".dimmed(), if !no_adjust { "Enabled".green() } else { "Disabled (Raw)".yellow() });
        }

        Commands::Inspect { file } => {
            let report = inspect_file(&file)?;

            println!("{}", "[*] Binary Container Inspection Report".bold().cyan());
            println!("  {:<18} : {}", "File".dimmed(), report.file_path);
            println!("  {:<18} : {} bytes", "Total Size".dimmed(), report.total_size);

            if let Some(kind) = report.detected_image {
                let name = match kind {
                    ImageKind::Png => "PNG Image",
                    ImageKind::Jpeg => "JPEG Image",
                    ImageKind::Gif => "GIF Image",
                    ImageKind::Webp => "WebP Image",
                    ImageKind::Bmp => "BMP Image",
                    ImageKind::Wav => "WAV Audio",
                };
                let bound = report.image_boundary.unwrap_or(0);
                println!(
                    "  {:<18} : {} (boundary: 0x0..0x{:X})",
                    "Primary Header".dimmed(),
                    name.green().bold(),
                    bound
                );
            } else {
                println!("  {:<18} : {}", "Primary Header".dimmed(), "None / Unknown".yellow());
            }

            if report.zip_found {
                let start_hex = report
                    .zip_start_offset
                    .map(|o| format!("0x{:X}", o))
                    .unwrap_or_else(|| "unknown".to_string());

                println!(
                    "  {:<18} : {} (start: {}, entries: {})",
                    "Payload Header".dimmed(),
                    "Valid ZIP Archive".green().bold(),
                    start_hex,
                    report.zip_entries_count
                );

                if report.slack_space_bytes > 0 {
                    println!(
                        "  {:<18} : {} bytes",
                        "Slack Space".dimmed(),
                        report.slack_space_bytes
                    );
                }

                if !report.entries.is_empty() {
                    println!("\n  {}", "Embedded Files:".bold());
                    for entry in &report.entries {
                        println!(
                            "    - {:<30} (compressed: {} B, size: {} B)",
                            entry.name.cyan(),
                            entry.compressed_size,
                            entry.uncompressed_size
                        );
                    }
                }
            } else {
                println!("  {:<18} : {}", "Payload Header".dimmed(), "No ZIP markers found".yellow());
            }

            println!();
            if report.is_polyglot {
                println!(
                    "{} Container is a verified polyglot file.",
                    "[+] Status:".green().bold()
                );
            } else {
                println!(
                    "{} File is not a dual-format polyglot.",
                    "[*] Status:".yellow().bold()
                );
            }
        }

        Commands::Extract {
            file,
            output_dir,
            target,
        } => {
            let t = match target {
                ExtractTargetArg::All => ExtractionTarget::All,
                ExtractTargetArg::Image => ExtractionTarget::Image,
                ExtractTargetArg::Payload => ExtractionTarget::Payload,
            };

            println!("{} Extracting components...", "[*]".cyan().bold());
            let paths = extract(&file, &output_dir, t)?;

            println!("{}", "[+] Extraction finished successfully:".green().bold());
            for p in paths {
                println!("  -> {}", p);
            }
        }
    }

    Ok(())
}
