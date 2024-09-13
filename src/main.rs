mod cli;
mod parse;

use clap::Parser;
use cli::{Cli, Commands};
use parse::{Project, ProjectConfig, Source};
use serde_json;
use std::{io::Write, path::PathBuf, process::Command};

const CONFIG_FILE: &str = ".config/project-manager/projects.json";

fn get_config_file_path() -> String {
    let home = std::env::var("HOME").expect("Failed to get home directory");
    format!("{}/{}", home, CONFIG_FILE)
}

fn get_root_directory(config: &ProjectConfig) -> String {
    let home = std::env::var("HOME").expect("Failed to get home directory");
    format!("{}/{}", home, config.root_dir)
}

fn get_project_directory(config: &ProjectConfig, project_path: &str) -> String {
    let root = get_root_directory(config);
    format!("{}/{}", root, project_path)
}

fn load_config() -> ProjectConfig {
    // user home directory
    let config_file = get_config_file_path();
    let config_file_path = std::path::Path::new(&config_file);

    if !config_file_path.exists() {
        let config = ProjectConfig::new();
        let json = serde_json::to_string(&config).expect("Failed to serialize ProjectConfig");
        // Create the directories if they don't exist
        std::fs::create_dir_all(config_file_path.parent().unwrap())
            .expect("Failed to create directories");
        std::fs::write(config_file_path, json).expect("Failed to write projects.json");
    }
    let config = std::fs::read_to_string(config_file_path).expect("Failed to read projects.json");
    serde_json::from_str(&config).expect("Failed to deserialize ProjectConfig")
}

fn save_config(config: &ProjectConfig) {
    let config_file = get_config_file_path();
    let json = serde_json::to_string(config).expect("Failed to serialize ProjectConfig");
    std::fs::write(config_file, json).expect("Failed to write projects.json");
    println!("Project configuration saved");
}

fn open_project(config: &ProjectConfig, project_name: &str, editor: &str) {
    let project = config
        .find_project(project_name)
        .expect("Project not found");
    let project_path = &project.path;
    let project_dir = get_project_directory(&config, &project_path);
    let project_dir = std::path::Path::new(&project_dir);
    // check if the project directory exists
    if !std::path::Path::new(&project_dir).exists() {
        println!("Project is not on the filesystem");
        if !project.source.is_none() {
            println!("Cloning project from source...");
            let source = project.source.as_ref().unwrap();
            let url = &source.url;
            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(|_url, username_from_url, _allowed_types| {
                git2::Cred::ssh_key_from_agent(username_from_url.unwrap())
            });

            let mut fetch_options = git2::FetchOptions::new();
            fetch_options.remote_callbacks(callbacks);

            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(fetch_options);

            let repo = builder
                .clone(url, &project_dir)
                .expect("Failed to clone repository");
            println!("Cloned repository: {:?}", repo.path());
        } else {
            println!("Project source is not available");
            return;
        }
    }

    Command::new(editor)
        .arg(project_dir)
        .spawn()
        .expect("Failed to open editor");
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
                return Some(Source {
                    source_type: "git".to_string(),
                    url,
                });
            }
            None => {
                println!("Failed to fetch remote URL");
                return None;
            }
        };
    } else {
        println!("Project is not a supported external source");
        return None;
    }
}

/// project_dir is the directory to add to the projects.json
/// it must be a subdirectory of the root_dir & it is relative to CWD
fn add_project(config: &mut ProjectConfig, project_dir: &str) {
    // get the project absolute path it must start with root_dir
    let root_dir = get_root_directory(config);
    let project_dir = std::path::Path::new(project_dir)
        .canonicalize()
        .expect("Failed to get canonical path");
    let project_path = project_dir
        .strip_prefix(&root_dir)
        .expect("Invalid project directory");

    // prompt for the project name & optional description
    let mut project_name = String::new();
    print!("Enter project name: ");
    std::io::stdout().flush().expect("Failed to flush stdout");
    std::io::stdin()
        .read_line(&mut project_name)
        .expect("Failed to read project name");
    let project_name = project_name.trim();

    let mut project_description = String::new();
    print!("Enter project description: ");
    std::io::stdout().flush().expect("Failed to flush stdout");
    std::io::stdin()
        .read_line(&mut project_description)
        .expect("Failed to read project description");
    let project_description = project_description.trim().to_string().into();

    // get the git repository information
    let source = get_project_source(&project_dir);
    config.add_project(Project {
        name: project_name.to_string(),
        path: project_path.to_str().unwrap().to_string(),
        description: project_description,
        languages: Vec::new(),
        source,
    });
}

fn main() {
    let cli = Cli::parse();
    let mut config = load_config();

    match &cli.command {
        Commands::Open { project_name } => {
            let editor = "code";
            open_project(&config, project_name, editor);
        }
        Commands::Add { directory } => {
            add_project(&mut config, directory);
            save_config(&config);
        }
        Commands::Remove { directory } => {
            // Implement the logic to remove a directory
            println!("Removing directory: {}", directory);
            todo!();
            // save_config(&config);
        }
        Commands::AddSource => {
            // Implement the logic to prompt questions for the new project
            println!("Adding new source...");
            todo!();
            // save_config(&config);
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
            let config_file = get_config_file_path();
            Command::new("code")
                .arg(config_file)
                .spawn()
                .expect("Failed to open projects.json");
        }
    }
}
