use std::{fs, path::{Path, PathBuf}, io, env};

use anyhow::{Context, Result, anyhow};
use clap::{command, Parser, Subcommand};
use env_logger::Builder;
use george::{
    cache::Cache,
    clean::{self, CleanOptions},
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

    /// Keep remaining empty directories
    #[arg(short, long)]
    keep_dir: bool,

    /// Full path to the config file (including name)
    #[arg(short, long)]
    config: Option<String>,
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

fn main() -> anyhow::Result<()> {
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
            let path = if let Some(path) = &cli.config {
                PathBuf::from(path)
            } else {
                find_config().context("Failed to find config")?
            };
            let cfg = Config::build(path)?;
            let cache = Cache::load().unwrap_or_default();
            let new_cache = deploy(cache, DeployOptions::new(!cli.keep_dir), cfg);
            new_cache.save().expect("Failed to save cache");
        }
        Commands::Clean {} => {
            let cache = Cache::load().unwrap_or_default();
            let new_cache = clean::clean(cache, CleanOptions::new(!cli.keep_dir));
            new_cache.save().expect("Failed to save cache");
        }
        Commands::Redeploy {} => {}
    }
    Ok(())
}

pub fn find_config() -> Option<PathBuf> {
    let cwd = env::current_dir().unwrap();
    let config = cwd.join(".george");
    if config.exists() {
        return Some(config);
    }

    for parent in cwd.ancestors() {
        let config = parent.join(".george");
        if config.exists() {
            return Some(config);
        }
    }
    None
}
