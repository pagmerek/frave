use crate::cli::{Cli, Commands};
use clap::Parser;
use commands::{decode, encode, fractalize};

mod cli;
mod commands;
mod coord;
mod encoder;
mod utils;
mod variants;

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Encode(cmd) => encode(cmd.bmp_path, cmd.variant),
        Commands::Decode(cmd) => decode(cmd.fr_path, cmd.variant),
        Commands::Fractalize(cmd) => fractalize(cmd.bmp_path, cmd.amount, cmd.variant),
    }
}
