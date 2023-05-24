use std::path::PathBuf;

use crate::variants::Variant;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// Image compression program based on complex based numeral systems
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Decode(Decode),
    Encode(Encode),
}

#[derive(Args)]
/// Encodes bitmap file to frave format
pub struct Encode {
    pub bmp_path: PathBuf,

    #[arg(short, long, value_enum, default_value_t = Variant::TameTwindragon)]
    pub variant: Variant,

    #[arg(short, default_value_t = String::from("a.frv"))]
    pub output: String,
}

#[derive(Args)]
/// Decodes frave file to bitmap format
pub struct Decode {
    pub fr_path: PathBuf,

    #[arg(short, default_value_t = String::from("a.bmp"))]
    pub output: String,
}
