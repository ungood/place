use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod commands;
mod config;
mod error;
mod git;
mod output;
mod state;
mod url;
mod workplace;

/// Manage workplaces: directories of independent git repositories.
#[derive(Debug, Parser)]
#[command(name = "place", version)]
struct Cli {
    /// Run as if place was started in <dir>.
    #[arg(short = 'C', global = true, value_name = "dir")]
    directory: Option<PathBuf>,

    // Not global, because `place add` has its own -e/--env.
    /// Load the place.<env>.toml files for these environments.
    #[arg(
        short = 'E',
        long = "env",
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

/// What every command needs: where to run and how.
#[derive(Debug)]
pub struct Context {
    pub dir: PathBuf,
    pub envs: Vec<String>,
    pub jobs: usize,
    pub quiet: bool,
    pub verbose: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let wrap = std::io::stderr().is_terminal();
    let _ = miette::set_hook(Box::new(move |_| {
        Box::new(miette::MietteHandlerOpts::new().wrap_lines(wrap).build())
    }));
    match run(cli) {
        Ok(code) => code,
        Err(err) => {
            output::error(err);
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, error::Error> {
    let cwd = std::env::current_dir().map_err(|source| error::Error::Read {
        file: "the current directory".into(),
        source,
    })?;
    let dir = match &cli.directory {
        None => cwd,
        Some(dir) => {
            let joined = cwd.join(dir);
            if !joined.is_dir() {
                return Err(error::Error::NoSuchDirectory {
                    dir: dir.display().to_string(),
                });
            }
            joined.canonicalize().unwrap_or(joined)
        }
    };
    let jobs = cli
        .jobs
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(4, |n| n.get()))
        .max(1);
    let ctx = Context {
        dir,
        envs: cli.envs.into_iter().filter(|e| !e.is_empty()).collect(),
        jobs,
        quiet: cli.quiet,
        verbose: cli.verbose,
    };
    match cli.command {
        Command::Init => commands::init::run(&ctx),
        Command::Add {
            targets,
            branch,
            env,
        } => commands::add::run(&ctx, &targets, branch.as_deref(), env.as_deref()),
        Command::Remove { paths, keep, force } => commands::remove::run(&ctx, &paths, keep, force),
        Command::Status {
            paths,
            fetch,
            json,
            exit_code,
        } => commands::status::run(&ctx, &paths, fetch, json, exit_code),
        Command::Sync { paths } => commands::sync::run(&ctx, &paths),
        Command::Doctor => commands::doctor::run(&ctx),
    }
}
