use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pm")]
#[command(about = "Project Manager CLI", long_about = None)]
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
    AddSource,
    List {
        #[arg(short, long)]
        verbose: bool,
    },
    Edit,
}