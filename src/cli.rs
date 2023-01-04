use crate::variants::Variant;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Decode(Decode),
    Encode(Encode),
    Fractalize(Fractalize),
}

#[derive(Args)]
/// encodes bitmap file to frave format
pub struct Encode {
    pub bmp_path: std::path::PathBuf,

    #[arg(short, long, value_enum, default_value_t = Variant::TameTwindragon)]
    pub variant: Variant,
}

#[derive(Args)]
/// decodes frave file to bitmap format
pub struct Decode {
    pub fr_path: std::path::PathBuf,

    #[arg(short, long, value_enum, default_value_t = Variant::TameTwindragon)]
    pub variant: Variant,
}

#[derive(Args)]
/// transforms bitmap file applying frave without quantization
pub struct Fractalize {
    pub bmp_path: std::path::PathBuf,

    #[arg(short, long, value_enum, default_value_t = Variant::TameTwindragon)]
    pub variant: Variant,

    #[arg(short, long)]
    pub amount: u8,
}
