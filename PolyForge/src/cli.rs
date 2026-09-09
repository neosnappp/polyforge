use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "polyforge",
    author = "neosnap",
    version = "0.1.0",
    about = "High-performance binary polyglot synthesizer and inspector",
    long_about = "A fast CLI tool for creating, inspecting, and extracting dual-format polyglot files (PNG/JPEG/GIF + ZIP) that parse cleanly in image viewers and archive utilities without warnings."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Pack {
        #[arg(short = 'i', long = "image", help = "Path to the cover image (PNG, JPEG, GIF)")]
        image: PathBuf,

        #[arg(short = 'p', long = "payload", help = "Path to the payload file, zip, or directory")]
        payload: PathBuf,

        #[arg(short = 'o', long = "output", help = "Output polyglot file path")]
        output: PathBuf,

        #[arg(long = "no-adjust", help = "Skip ZIP central directory offset realignment")]
        no_adjust: bool,
    },

    Inspect {
        #[arg(help = "Target file to analyze for polyglot structures")]
        file: PathBuf,
    },

    Extract {
        #[arg(help = "Polyglot file to extract components from")]
        file: PathBuf,

        #[arg(short = 'o', long = "output-dir", default_value = "./extracted", help = "Destination directory")]
        output_dir: PathBuf,

        #[arg(short = 't', long = "target", value_enum, default_value_t = ExtractTargetArg::All, help = "Components to extract")]
        target: ExtractTargetArg,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtractTargetArg {
    All,
    Image,
    Payload,
}
