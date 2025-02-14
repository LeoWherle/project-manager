extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub version: String,
    /// The editor to use when opening a project (e.g. "code", "vim", "nano")
    /// Must support opening a directory the following way: `editor /path/to/directory`
    pub editor: String,
    pub root_dir: String,
    pub projects: Vec<Project>,
}

impl ProjectConfig {
    pub fn new() -> ProjectConfig {
        ProjectConfig {
            version: "1.0".to_string(),
            root_dir: "my_projects".to_string(),
            editor: "code".to_string(),
            projects: Vec::new(),
        }
    }

    pub fn add_project(&mut self, project: Project) {
        self.projects.push(project);
    }

    pub fn find_project(&self, project_name: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.name == project_name)
    }
}

/// The path to the project from ~/Documents
/// ```
/// {
///     "name" : "BSQ",
///     "path" : "B-CPE-110-TLS-1-1-BSQ",
///     "description" : "Find the biggest square in a map",
///     "languages": ["C"],
///     "build" : {
///         "type" : "make",
///         "target" : "bsq"
///     },
///     "source" : {
///         "type" : "git",
///         "url" : "https://github.com/Example/BSQ.git"
///     }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub languages: Vec<String>,
    pub source: Option<Source>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Source {
    #[serde(rename = "type")]
    pub source_type: SourceType,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SourceType {
    /// A git repository
    #[serde(rename = "git")]
    Git,
    /// A web URL
    #[serde(rename = "web")]
    Web,
}
