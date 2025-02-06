pub mod commands;

use clap::Parser;
use commands::{bench, decode, encode};

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// Image compression program based on complex based numeral systems
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
    #[command(flatten)]
    pub global_options: GlobalOptions,
}


#[derive(clap::Args)]
#[non_exhaustive]
pub struct GlobalOptions {
    /// Print debug information; can be repeated.
    #[arg(short, long, global = true, default_value_t = false)]
    pub verbose: bool,
    /// Do not print logs to console.
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,  
}

#[derive(clap::Subcommand)]
pub enum Commands {
    Decode(decode::Decode),
    Encode(encode::Encode),
    Bench(bench::Bench),
}


fn main() {
    let cli = Args::parse();
    match cli.command {
        Commands::Encode(cmd) => encode::encode_image(cmd, cli.global_options.verbose),
        Commands::Decode(cmd) => decode::decode_image(cmd),
        Commands::Bench(cmd) => bench::benchmark(cmd),
    }
}
