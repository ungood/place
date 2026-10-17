use std::path::PathBuf;

use clap::{Parser, Subcommand};
use miette::{Diagnostic, Result};

/// Manage workplaces: directories of independent git repositories.
#[derive(Debug, Parser)]
#[command(name = "place", version)]
struct Cli {
    /// Run as if place was started in <dir>.
    #[arg(short = 'C', global = true, value_name = "dir")]
    directory: Option<PathBuf>,

    /// Load the place.<env>.toml files for these environments.
    #[arg(
        short = 'E',
        long = "env",
        global = true,
        env = "PLACE_ENV",
        value_delimiter = ',',
        value_name = "env"
    )]
    envs: Vec<String>,

    /// Number of repos to work on at once.
    #[arg(short = 'j', long, global = true, value_name = "n")]
    jobs: Option<usize>,

    /// Print less.
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Print more.
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Make the current directory a workplace.
    Init,

    /// List a repo in the workplace, cloning it or adopting a checkout.
    Add {
        /// A URL to clone, or paths of checkouts to adopt.
        #[arg(required = true, value_name = "url|path")]
        targets: Vec<String>,

        /// Branch, tag or commit to track.
        #[arg(short, long, value_name = "ref")]
        branch: Option<String>,

        /// Write to place.<env>.toml instead of place.toml.
        #[arg(short, long = "env", value_name = "env")]
        env: Option<String>,
    },

    /// Remove repos and delete their checkouts when nothing would be lost.
    #[command(visible_alias = "rm")]
    Remove {
        #[arg(required = true, value_name = "path")]
        paths: Vec<String>,

        /// Remove only the entry and leave the checkout on disk.
        #[arg(long)]
        keep: bool,

        /// Delete the checkout even if work would be lost.
        #[arg(long, conflicts_with = "keep")]
        force: bool,
    },

    /// Show the state of every repo.
    Status {
        #[arg(value_name = "path")]
        paths: Vec<String>,

        /// Fetch before reporting.
        #[arg(long)]
        fetch: bool,

        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,

        /// Exit with 1 when any repo isn't clean and on its ref.
        #[arg(long)]
        exit_code: bool,
    },

    /// Clone missing repos, fetch, and fast-forward clean ones.
    Sync {
        #[arg(value_name = "path")]
        paths: Vec<String>,
    },

    /// Check the workplace for problems.
    Doctor,
}

impl Command {
    fn name(&self) -> &'static str {
        match self {
            Command::Init => "init",
            Command::Add { .. } => "add",
            Command::Remove { .. } => "remove",
            Command::Status { .. } => "status",
            Command::Sync { .. } => "sync",
            Command::Doctor => "doctor",
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    Err(NotImplemented {
        command: cli.command.name(),
    }
    .into())
}

#[derive(Debug, thiserror::Error, Diagnostic)]
#[error("\"place {command}\" isn't implemented yet.")]
#[diagnostic(
    code(place::not_implemented),
    help("place is still being built. See docs/design.md for what this command will do.")
)]
struct NotImplemented {
    command: &'static str,
}
