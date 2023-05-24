use clap::Parser;

use frave::cli::{Cli, Commands};
use frave::commands::{decode, encode};

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Encode(cmd) => encode(cmd.bmp_path.clone(), cmd.variant, cmd.output.clone()),
        Commands::Decode(cmd) => decode(cmd.fr_path.clone(), cmd.output.clone()),
    }
}
