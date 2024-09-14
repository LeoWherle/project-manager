use clap::{Parser, Subcommand};
use std::sync::LazyLock;

fn generate_version() -> String {
    // Generate a version string from the latest git commit hash
    let repo = git2::Repository::discover(".").expect("Failed to discover git repository");
    let head = repo.head().expect("Failed to get HEAD reference");
    let commit = head.peel_to_commit().expect("Failed to get commit object");
    let hash = commit.id().to_string();
    format!("0.1.0-{}", hash)
}

static VERSION: LazyLock<String> = LazyLock::new(|| generate_version());

#[derive(Parser)]
#[command(name = "pm")]
#[command(about = "Project Manager CLI", long_about = None)]
#[command(version = &**VERSION)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Open {
        project_name: String,
    },
    Add {
        directory: String,
    },
    Remove {
        directory: String,
    },
    AddSource {
        url: String,
    },
    List {
        #[arg(short, long)]
        path: bool,
        #[arg(short, long)]
        description: bool,
        #[arg(short, long)]
        languages: bool,
        #[arg(short, long)]
        source: bool,
    },
    Edit,
}