mod cli;
mod parse;

use clap::Parser;
use cli::{Cli, Commands};
use git2::Repository;
use parse::{Project, ProjectConfig, Source};
use prettytable::{cell, Row, Table};
use serde_json;
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const CONFIG_FILE: &str = "project-manager/projects.json";

fn get_config_file_path() -> Result<PathBuf> {
    if let Some(config_dir) = dirs::config_dir() {
        Ok(config_dir.join(CONFIG_FILE))
    } else {
        Err("Failed to get config directory".into())
    }
}

fn get_root_directory(config: &ProjectConfig) -> Result<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        Ok(home.join(&config.root_dir))
    } else {
        Err("Failed to get home directory".into())
    }
}

fn get_project_directory(config: &ProjectConfig, project_path: &Path) -> Result<PathBuf> {
    let root = get_root_directory(config)?;
    Ok(root.join(project_path))
}

fn load_config() -> Result<ProjectConfig> {
    let config_file = get_config_file_path()?;
    if !config_file.exists() {
        let config = ProjectConfig::new();
        let json = serde_json::to_string_pretty(&config)?;
        if let Some(parent) = config_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&config_file, json)?;
    }
    let config = std::fs::read_to_string(&config_file)?;
    let config: ProjectConfig = serde_json::from_str(&config)?;
    Ok(config)
}

fn save_config(config: &ProjectConfig) -> Result<()> {
    let config_file = get_config_file_path()?;
    let json = serde_json::to_string_pretty(config)?;
    std::fs::write(config_file, json)?;
    println!("Project configuration saved");
    Ok(())
}

fn clone_git_repository(url: &str, project_dir: &Path) -> Result<Repository> {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        if let Some(username) = username_from_url {
            git2::Cred::ssh_key_from_agent(username)
        } else {
            Err(git2::Error::from_str("git Username not provided"))
        }
    });

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);

    Ok(builder.clone(url, project_dir)?)
}

fn fetch_repository_from_source(source: &Source, project_dir: &Path) -> Result<PathBuf> {
    match &source.source_type[..] {
        "git" => Ok(clone_git_repository(&source.url, project_dir)?
            .path()
            .to_path_buf()),
        _ => Err("Unsupported source type".into()),
    }
}

fn open_project(config: &ProjectConfig, project_name: &str, editor: &str) -> Result<()> {
    let project = config
        .find_project(project_name)
        .ok_or("Project not found")?;
    let project_path = Path::new(&project.path);
    let project_dir = get_project_directory(&config, project_path)?;

    if !project_dir.exists() {
        println!("Project is not on the filesystem");
        if let Some(source) = &project.source {
            println!("Fetching project from source...");
            let repo = fetch_repository_from_source(source, &project_dir)?;
            println!("Cloned repository: {:?}", repo);
        } else {
            return Err("Project source is not available".into());
        }
    }

    Command::new(editor).arg(project_dir).spawn()?.wait()?;
    Ok(())
}

fn fetch_remote_url(project_dir: &Path) -> Option<String> {
    let repo = git2::Repository::open(project_dir).ok()?;
    let remote = repo.find_remote("origin").ok()?;
    remote.url().map(|url| url.to_string())
}

fn get_git_project_source(project_dir: &Path) -> Option<Source> {
    println!("Adding git repository information...");
    fetch_remote_url(project_dir)
        .map(|url| {
            println!("Git repository URL: {}", url);
            Source {
                source_type: "git".to_string(),
                url,
            }
        })
        .or_else(|| {
            println!("Failed to fetch remote URL");
            None
        })
}

fn get_project_source(project_dir: &PathBuf) -> Option<Source> {
    if project_dir.join(".git").exists() {
        return get_git_project_source(project_dir);
    }
    println!("Project is not a supported external source");
    None
}

fn get_user_input(prompt: &str) -> Result<String> {
    let mut input = String::new();
    print!("{}", prompt);
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

// ask user for project name until a valid name is entered
fn get_user_input_project_name(config: &ProjectConfig) -> Result<String> {
    loop {
        let project_name = get_user_input("Enter project name: ")?;
        if config.find_project(&project_name).is_some() {
            println!("Project name already exists");
        } else {
            return Ok(project_name.to_string());
        }
    }
}

fn add_project(config: &mut ProjectConfig, project_dir: &str) -> Result<()> {
    let root_dir = get_root_directory(config)?;
    let project_dir = Path::new(project_dir).canonicalize()?;
    let project_path = project_dir.strip_prefix(&root_dir)?;

    let project_name = get_user_input_project_name(config)?;
    let project_description = get_user_input("Enter project description: ")?.into();

    let source = get_project_source(&project_dir);
    config.add_project(Project {
        name: project_name.to_string(),
        path: project_path
            .to_str()
            .ok_or("Failed to convert project path to string")?
            .to_string(),
        description: project_description,
        languages: Vec::new(),
        source,
    });
    Ok(())
}

fn add_project_from_source(config: &mut ProjectConfig, source: Source) -> Result<()> {
    let project_name = get_user_input_project_name(config)?;

    let source_name = source
        .url
        .split('/')
        .last()
        .ok_or("Invalid URL: missing '/'")?;
    let source_name = if source_name.ends_with(".git") {
        &source_name[..source_name.len() - 4]
    } else {
        source_name
    };

    let project_description = get_user_input("Enter project description: ")?.into();

    config.add_project(Project {
        name: project_name.to_string(),
        path: source_name.to_string(),
        description: project_description,
        languages: Vec::new(),
        source: Some(source),
    });
    Ok(())
}

fn list_projects(
    config: &ProjectConfig,
    path: bool,
    description: bool,
    languages: bool,
    source: bool,
) {
    let mut table = Table::new();

    let mut headers = if path || description || languages || source {
        vec![cell!("Name")]
    } else {
        vec![]
    };

    if path {
        headers.push(cell!("Path"));
    }
    if description {
        headers.push(cell!("Description"));
    }
    if languages {
        headers.push(cell!("Languages"));
    }
    if source {
        headers.push(cell!("Source"));
    }
    if path || description || languages || source {
        table.add_row(Row::new(headers));
    }

    for project in &config.projects {
        let mut row = vec![cell!(project.name.to_string())];

        if path {
            row.push(cell!(project.path.to_string()));
        }
        if description {
            row.push(cell!(project.description.clone().unwrap_or_default()));
        }
        if languages {
            row.push(cell!(project.languages.join(", ")));
        }
        if source {
            row.push(cell!(project
                .source
                .as_ref()
                .map_or("".to_string(), |s| s.url.clone())));
        }

        table.add_row(Row::new(row));
    }

    table.set_format(*prettytable::format::consts::FORMAT_CLEAN);
    table.printstd();
}

// if the project config has source information, ask the user if they want to keep the project in the config
fn remove_project_with_source(
    config: &mut ProjectConfig,
    project_name: &str,
    project_dir: &Path,
) -> Result<()> {
    let prompt = format!("Do you want to remove {project_name} from the project list? (y/N): ",);
    match get_user_input(&prompt)?.to_lowercase().as_str() {
        "y" | "yes" => {
            config.projects.retain(|p| p.name != project_name);
        }
        _ => println!("Keeping project in the project list"),
    }
    if project_dir.exists() {
        std::fs::remove_dir_all(&project_dir)?;
        println!("Project directory removed");
    }
    Ok(())
}

fn remove_project(config: &mut ProjectConfig, project_name: &str) -> Result<()> {
    let project = config
        .find_project(project_name)
        .ok_or("Project not found")?;

    let project_path = Path::new(&project.path);
    let project_dir = get_project_directory(&config, project_path)?;

    println!(
        "You are about to remove the following directory: {:?}",
        project_dir
    );
    let prompt = format!("Are you sure you want to remove {}? (y/N): ", project_name);

    match get_user_input(&prompt)?.to_lowercase().as_str() {
        "y" | "yes" => {
            remove_project_with_source(config, project_name, &project_dir)?;
        }
        _ => println!("Project removal aborted"),
    }
    Ok(())
}

fn main() -> Result<()> {
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
            remove_project(&mut config, directory)?;
            save_config(&config)?;
        }
        Commands::AddSource { url } => {
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
        Commands::List {
            path,
            description,
            languages,
            source,
        } => {
            list_projects(&config, *path, *description, *languages, *source);
        }
        Commands::Edit => {
            let config_file = get_config_file_path()?;
            Command::new("code").arg(config_file).spawn()?.wait()?;
        }
    }

    Ok(())
}
