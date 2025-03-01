use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use git2::Repository;
use prettytable::{cell, Row, Table};

use crate::parse::{Project, ProjectConfig, Source};
use crate::Result;

const DEFAULT_ROOT_DIR: &str = "project-manager/projects.json";

pub mod fetchers {
    use super::*;

    /// A trait to fetch a project source (e.g. cloning from a repository)
    pub trait SourceFetcher {
        fn fetch_source(&self, source: &Source, project_dir: &Path) -> Result<PathBuf>;
    }

    pub struct GitFetcher;

    impl GitFetcher {
        fn clone_repository(url: &str, project_dir: &Path) -> Result<Repository> {
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
    }

    impl SourceFetcher for GitFetcher {
        fn fetch_source(&self, source: &Source, project_dir: &Path) -> Result<PathBuf> {
            let repo = Self::clone_repository(&source.url, project_dir)?;
            Ok(repo.path().to_path_buf())
        }
    }

    pub fn get_fetcher(source: &Source) -> Option<Box<dyn SourceFetcher>> {
        match source.source_type {
            crate::parse::SourceType::Git => Some(Box::new(GitFetcher)),
            _ => None,
        }
    }
}

pub mod prompts {
    use super::*;

    pub trait Prompter {
        fn get_input(&self, prompt: &str) -> Result<String>;
    }

    pub struct StdPrompter;

    impl Prompter for StdPrompter {
        fn get_input(&self, prompt: &str) -> Result<String> {
            let mut input = String::new();
            print!("{}", prompt);
            io::stdout().flush()?;
            io::stdin().read_line(&mut input)?;
            Ok(input.trim().to_string())
        }
    }
}

pub struct Config {
    config: ProjectConfig,
    prompter: Box<dyn prompts::Prompter>,
}

impl Config {
    pub fn new() -> Config {
        let config = match Self::load_config() {
            Ok(cfg) => cfg,
            Err(_) => ProjectConfig::new(),
        };
        Config {
            config,
            prompter: Box::new(prompts::StdPrompter),
        }
    }

    /// Allow replacing the prompter (e.g. with one that supports translation)
    #[allow(dead_code)]
    pub fn set_prompter(&mut self, prompter: Box<dyn prompts::Prompter>) {
        self.prompter = prompter;
    }

    #[allow(dead_code)]
    pub fn inner_mut(&mut self) -> &mut ProjectConfig {
        &mut self.config
    }

    pub fn inner(&self) -> &ProjectConfig {
        &self.config
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
        let config_content = std::fs::read_to_string(&config_file)?;
        let config: ProjectConfig = serde_json::from_str(&config_content)?;
        Ok(config)
    }

    fn get_root_directory(&self) -> Result<PathBuf> {
        if let Some(home) = dirs::home_dir() {
            Ok(home.join(&self.config.root_dir))
        } else {
            Err("Failed to get home directory".into())
        }
    }

    fn get_project_directory(&self, project_path: &Path) -> Result<PathBuf> {
        let root = self.get_root_directory()?;
        Ok(root.join(project_path))
    }

    pub fn save_config(&self) -> Result<()> {
        let config_file = get_config_file_path()?;
        let json = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(config_file, json)?;
        println!("Project configuration saved");
        Ok(())
    }

    /// Opens the project in the configured editor.
    pub fn open_project(&self, project_name: &str) -> Result<()> {
        let project = self
            .config
            .find_project(project_name)
            .ok_or("Project not found")?;
        let project_path = Path::new(&project.path);
        let project_dir = self.get_project_directory(project_path)?;

        if !project_dir.exists() {
            println!("Project is not on the filesystem");
            if let Some(source) = &project.source {
                println!("Fetching project from source...");
                self.fetch_project_source(source, &project_dir)?;
            } else {
                return Err("Project source is not available".into());
            }
        }

        Command::new(&self.config.editor)
            .arg(project_dir)
            .spawn()?
            .wait()?;
        Ok(())
    }

    /// Prints the full path of the project (fetching it if necessary)
    pub fn navigate_project(&self, project_name: &str) -> Result<()> {
        let project = self
            .config
            .find_project(project_name)
            .ok_or("Project not found")?;
        let project_path = Path::new(&project.path);
        let project_dir = self.get_project_directory(project_path)?;

        if !project_dir.exists() {
            println!("Project is not on the filesystem");
            if let Some(source) = &project.source {
                println!("Fetching project from source...");
                self.fetch_project_source(source, &project_dir)?;
            } else {
                return Err("Project source is not available".into());
            }
        }
        println!("{}", project_dir.display());
        Ok(())
    }

    /// Fetch the project from its source using the appropriate fetcher.
    fn fetch_project_source(&self, source: &Source, project_dir: &Path) -> Result<PathBuf> {
        if let Some(fetcher) = fetchers::get_fetcher(source) {
            fetcher.fetch_source(source, project_dir)
        } else {
            Err("Unsupported source type".into())
        }
    }

    /// Ask the user repeatedly for a unique project name.
    fn get_user_input_project_name(&self) -> Result<String> {
        loop {
            let project_name = self.prompter.get_input("Enter project name: ")?;
            if self.config.find_project(&project_name).is_some() {
                println!("Project name already exists");
            } else {
                return Ok(project_name);
            }
        }
    }

    /// Add a project from an existing directory
    pub fn add_project(&mut self, project_dir: &str) -> Result<()> {
        let root_dir = self.get_root_directory()?;
        let project_dir = Path::new(project_dir).canonicalize()?;
        let project_path = project_dir.strip_prefix(&root_dir)?;

        let project_name = self.get_user_input_project_name()?;
        let project_description = self.prompter.get_input("Enter project description: ")?;

        let source = get_project_source(&project_dir);
        self.config.add_project(Project {
            name: project_name,
            path: project_path
                .to_str()
                .ok_or("Failed to convert project path to string")?
                .to_string(),
            description: Some(project_description),
            languages: Vec::new(),
            source,
        });
        Ok(())
    }

    /// TODO: make it more generic to support other source types
    pub fn add_project_from_source(&mut self, source: Source) -> Result<()> {
        let project_name = self.get_user_input_project_name()?;

        let source_name = source
            .url
            .split('/')
            .last()
            .ok_or("Invalid URL: missing '/'")?;
        let source_name = if let Some(new_src_name) = source_name.strip_suffix(".git") {
            new_src_name
        } else {
            source_name
        };

        let project_description = self.prompter.get_input("Enter project description: ")?;

        self.config.add_project(Project {
            name: project_name,
            path: source_name.to_string(),
            description: Some(project_description),
            languages: Vec::new(),
            source: Some(source),
        });
        Ok(())
    }

    pub fn list_projects(&self, path: bool, description: bool, languages: bool, source: bool) {
        let mut table = Table::new();
        let mut headers = vec![];

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
        if !headers.is_empty() {
            headers.insert(0, cell!("Name"));
            table.add_row(Row::new(headers));
        }

        let mut sorted_projects = self.config.projects.clone();
        sorted_projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        for project in sorted_projects {
            let mut row = vec![cell!(project.name)];
            if path {
                row.push(cell!(project.path));
            }
            if description {
                row.push(cell!(project.description.clone().unwrap_or_default()));
            }
            if languages {
                row.push(cell!(project.languages.join(", ")));
            }
            if source {
                row.push(cell!(project.source.as_ref().map_or("", |s| &s.url)));
            }
            table.add_row(Row::new(row));
        }

        table.set_format(*prettytable::format::consts::FORMAT_CLEAN);
        table.printstd();
    }

    /// Removes a project both from the configuration and (optionally) its directory.
    pub fn remove_project(&mut self, project_name: &str) -> Result<()> {
        let project = self
            .config
            .find_project(project_name)
            .ok_or("Project not found")?;
        let project_path = Path::new(&project.path);
        let project_dir = self.get_project_directory(project_path)?;

        println!(
            "You are about to remove the following directory: {:?}",
            project_dir
        );
        let prompt = format!("Are you sure you want to remove {}? (y/N): ", project_name);
        match self.prompter.get_input(&prompt)?.to_lowercase().as_str() {
            "y" | "yes" => self.remove_project_with_source(project_name, &project_dir)?,
            _ => println!("Project removal aborted"),
        }
        Ok(())
    }

    fn remove_project_with_source(&mut self, project_name: &str, project_dir: &Path) -> Result<()> {
        let prompt = format!(
            "Do you want to remove {} from the project list? (y/N): ",
            project_name
        );
        match self.prompter.get_input(&prompt)?.to_lowercase().as_str() {
            "y" | "yes" => self.config.projects.retain(|p| p.name != project_name),
            _ => println!("Keeping project in the project list"),
        }
        if project_dir.exists() {
            std::fs::remove_dir_all(project_dir)?;
            println!("Project directory removed");
        }
        Ok(())
    }

    pub fn inspect(&self) {
        if let Ok(folders) = self.config.get_unregistered_folders() {
            if !folders.is_empty() {
                println!("Unregistered folders:");
                for folder in folders {
                    println!("  {}", folder);
                }
            } else {
                println!("No unregistered folders found");
            }
        } else {
            println!("Failed to inspect unregistered folders");
        }
    }
}

/// Returns the path to the configuration file.
pub fn get_config_file_path() -> Result<PathBuf> {
    if let Some(config_dir) = dirs::config_dir() {
        Ok(config_dir.join(DEFAULT_ROOT_DIR))
    } else {
        Err("Failed to get config directory".into())
    }
}

/// Determines the source of a project by checking for a .git folder.
/// TODO Add more checks when more source types are supported.
fn get_project_source(project_dir: &Path) -> Option<Source> {
    if project_dir.join(".git").exists() {
        get_git_project_source(project_dir)
    } else {
        println!("Project is not a supported external source");
        None
    }
}

/// Extracts git repository information.
fn get_git_project_source(project_dir: &Path) -> Option<Source> {
    println!("Adding git repository information...");
    fetch_remote_url(project_dir).map(|url| {
        println!("Git repository URL: {}", url);
        Source {
            source_type: crate::parse::SourceType::Git,
            url,
        }
    })
}

/// Fetches the remote URL from a Git repository.
fn fetch_remote_url(project_dir: &Path) -> Option<String> {
    let repo = Repository::open(project_dir).ok()?;
    let remote = repo.find_remote("origin").ok()?;
    remote.url().map(|url| url.to_string())
}
