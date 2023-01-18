use crate::cli::{Cli, Commands};
use clap::Parser;
use commands::{decode, encode, fractalize};

mod cli;
mod commands;
mod coord;
mod decoder;
mod encoder;
mod frave_image;
mod utils;
mod variants;

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Encode(cmd) => encode(cmd.bmp_path.clone(), cmd.variant, cmd.output.clone()),
        Commands::Decode(cmd) => decode(cmd.fr_path.clone(), cmd.output.clone()),
        Commands::Fractalize(cmd) => fractalize(
            cmd.bmp_path.clone(),
            cmd.amount,
            cmd.variant,
            cmd.output.clone(),
        ),
    }
}
