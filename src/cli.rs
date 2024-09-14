use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pm")]
#[command(about = "Project Manager CLI", long_about = None)]
#[command(version = "0.1.1")]
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