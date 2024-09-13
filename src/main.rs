mod cli;
mod parse;

use clap::Parser;
use cli::{Cli, Commands};
use parse::{Project, ProjectConfig, Source};
use serde_json;
use std::{io::{self, Write}, path::{Path, PathBuf}, process::Command};

const CONFIG_FILE: &str = ".config/project-manager/projects.json";

fn get_config_file_path() -> Result<String, std::env::VarError> {
    let home = std::env::var("HOME")?;
    Ok(format!("{}/{}", home, CONFIG_FILE))
}

fn get_root_directory(config: &ProjectConfig) -> Result<String, std::env::VarError> {
    let home = std::env::var("HOME")?;
    Ok(format!("{}/{}", home, config.root_dir))
}

fn get_project_directory(config: &ProjectConfig, project_path: &str) -> Result<String, std::env::VarError> {
    let root = get_root_directory(config)?;
    Ok(format!("{}/{}", root, project_path))
}

fn load_config() -> Result<ProjectConfig, Box<dyn std::error::Error>> {
    let config_file = get_config_file_path()?;
    let config_file_path = Path::new(&config_file);

    if !config_file_path.exists() {
        let config = ProjectConfig::new();
        let json = serde_json::to_string_pretty(&config)?;
        std::fs::create_dir_all(config_file_path.parent().unwrap())?;
        std::fs::write(config_file_path, json)?;
    }
    let config = std::fs::read_to_string(config_file_path)?;
    let config: ProjectConfig = serde_json::from_str(&config)?;
    Ok(config)
}

fn save_config(config: &ProjectConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_file = get_config_file_path()?;
    let json = serde_json::to_string_pretty(config)?;
    std::fs::write(config_file, json)?;
    println!("Project configuration saved");
    Ok(())
}

fn open_project(config: &ProjectConfig, project_name: &str, editor: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project = config.find_project(project_name).ok_or("Project not found")?;
    let project_path = &project.path;
    let project_dir = get_project_directory(&config, &project_path)?;
    let project_dir = Path::new(&project_dir);

    if !project_dir.exists() {
        println!("Project is not on the filesystem");
        if let Some(source) = &project.source {
            println!("Cloning project from source...");
            let url = &source.url;
            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(|_url, username_from_url, _allowed_types| {
                git2::Cred::ssh_key_from_agent(username_from_url.unwrap())
            });

            let mut fetch_options = git2::FetchOptions::new();
            fetch_options.remote_callbacks(callbacks);

            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(fetch_options);

            let repo = builder.clone(url, &project_dir)?;
            println!("Cloned repository: {:?}", repo.path());
        } else {
            println!("Project source is not available");
            return Ok(());
        }
    }

    Command::new(editor)
        .arg(project_dir)
        .spawn()?
        .wait()?;
    Ok(())
}

fn fetch_remote_url(project_dir: &str) -> Option<String> {
    let repo = git2::Repository::open(project_dir).ok()?;
    let remote = repo.find_remote("origin").ok()?;
    remote.url().map(|url| url.to_string())
}

fn get_project_source(project_dir: &PathBuf) -> Option<Source> {
    if project_dir.join(".git").exists() {
        println!("Adding git repository information...");
        match fetch_remote_url(project_dir.to_str().unwrap()) {
            Some(url) => {
                println!("Git repository URL: {}", url);
                Some(Source {
                    source_type: "git".to_string(),
                    url,
                })
            }
            None => {
                println!("Failed to fetch remote URL");
                None
            }
        }
    } else {
        println!("Project is not a supported external source");
        None
    }
}

fn add_project(config: &mut ProjectConfig, project_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root_dir = get_root_directory(config)?;
    let project_dir = Path::new(project_dir).canonicalize()?;
    let project_path = project_dir.strip_prefix(&root_dir)?;

    let mut project_name = String::new();
    print!("Enter project name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut project_name)?;
    let project_name = project_name.trim();

    let mut project_description = String::new();
    print!("Enter project description: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut project_description)?;
    let project_description = project_description.trim().to_string().into();

    let source = get_project_source(&project_dir);
    config.add_project(Project {
        name: project_name.to_string(),
        path: project_path.to_str().unwrap().to_string(),
        description: project_description,
        languages: Vec::new(),
        source,
    });
    Ok(())
}

fn add_project_from_source(config: &mut ProjectConfig, source: Source) -> Result<(), Box<dyn std::error::Error>> {
    let mut project_name = String::new();
    print!("Enter project name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut project_name)?;
    let project_name = project_name.trim();

    let source_name = source.url.split('/').last().unwrap();
    let source_name = source_name.split('.').next().unwrap();

    let mut project_description = String::new();
    print!("Enter project description: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut project_description)?;
    let project_description = project_description.trim().to_string().into();

    config.add_project(Project {
        name: project_name.to_string(),
        path: source_name.to_string(),
        description: project_description,
        languages: Vec::new(),
        source: Some(source),
    });
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut config = load_config()?;

    match &cli.command {
        Commands::Open { project_name } => {
            let editor = "code";
            open_project(&config, project_name, editor)?;
        }
        Commands::Add { directory } => {
            add_project(&mut config, directory)?;
            save_config(&config)?;
        }
        Commands::Remove { directory } => {
            // Implement the logic to remove a directory
            println!("Removing directory: {}", directory);
            todo!();
            // save_config(&config)?;
        }
        Commands::AddSource { url } => {
            // Implement the logic to prompt questions for the new project
            println!("Adding new source...");
            add_project_from_source(
                &mut config,
                Source {
                    source_type: "git".to_string(),
                    url: url.to_string(),
                },
            )?;
            save_config(&config)?;
        }
        Commands::List { verbose } => {
            // Implement the logic to list projects
            if *verbose {
                println!("Listing projects in verbose mode...");
            } else {
                println!("Listing projects...");
            }
        }
        Commands::Edit => {
            let config_file = get_config_file_path()?;
            Command::new("code")
                .arg(config_file)
                .spawn()?
                .wait()?;
        }
    }

    Ok(())
}
