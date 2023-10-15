use std::fs;

use clap::{command, Parser, Subcommand};
use env_logger::Builder;
use george::{
    cache::Cache,
    clean,
    config::Config,
    deploy::{deploy, DeployOptions},
};
use std::io::Write;

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploys your dotfiles by creating symlinks
    Deploy {},
    /// Removes all (cached) created symlinks
    Clean {},
    /// Does a clean and then a deploy
    Redeploy {},
}

fn main() {
    let mut builder = Builder::new();
    builder
        .filter(None, log::LevelFilter::Info)
        .format(|buf, record| {
            let args = record.args();
            writeln!(buf, "{}", args)
        })
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Deploy {} => {
            let cfg = fs::read_to_string(".george").unwrap();
            let cfg = Config::build(&cfg).unwrap();
            let cache = Cache::load().unwrap_or_default();
            let new_cache = deploy(cache, DeployOptions::new(&cfg));
            new_cache.save().expect("Failed to save cache");
        }
        Commands::Clean {} => {
            let cache = Cache::load().unwrap_or_default();
            let new_cache = clean::clean(cache);
            new_cache.save().expect("Failed to save cache");
        }
        Commands::Redeploy {} => {}
    }
}
