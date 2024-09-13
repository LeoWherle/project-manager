use clap::{Parser, Subcommand};
use std::process::Command;
use std::sync::LazyLock;

fn generate_version() -> String {
    // Generate a version string from the latest git commit hash
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to execute git command");
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
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
        verbose: bool,
    },
    Edit,
}