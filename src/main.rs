use clap::{command, Command, Parser, Subcommand};
use george::config::Config;

#[derive(Parser)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploys your dotfiles by creating symlinks
    Deploy { },
}

fn main() {
    let cli = Cli::parse();

    let cfg = Config::build("src -> dest");

    // match &cli.command {
    //     Commands::Deploy {  } => crate::,
    // }
}
