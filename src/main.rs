mod cli;
mod config;
mod parse;

use clap::Parser;
use cli::{Cli, Commands};
use config::{get_config_file_path, Config};
use parse::Source;
use std::process::Command;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn handle_commands(cli: &Cli) -> Result<()> {
    let mut config = Config::new();

    match &cli.command {
        Commands::Open { project_name } => {
            config.open_project(project_name)?;
        }
        Commands::Pwd { project_name } => {
            config.navigate_project(project_name)?;
        }
        Commands::Add { directory } => {
            config.add_project(directory)?;
            config.save_config()?;
        }
        Commands::Remove { directory } => {
            config.remove_project(directory)?;
            config.save_config()?;
        }
        Commands::AddSource { url } => {
            println!("Adding new source...");
            config.add_project_from_source(Source {
                source_type: parse::SourceType::Git,
                url: url.to_string(),
            })?;
            config.save_config()?;
        }
        Commands::List {
            path,
            description,
            languages,
            source,
        } => {
            config.list_projects(*path, *description, *languages, *source);
        }
        Commands::Edit => {
            let config_file = get_config_file_path()?;
            let editor = &config.inner().editor;
            Command::new(editor).arg(config_file).spawn()?.wait()?;
        }
        Commands::Inspect => {
            config.inspect();
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    handle_commands(&cli)
}
