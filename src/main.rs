use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{ArgAction, Parser, Subcommand};

mod commands;
mod config;
mod error;
mod git;
mod output;
mod state;
mod url;
mod workplace;

const ABOUT: &str = "place manages workplaces: directories of independent git repositories, \
plus the configuration they share.

You list repos in place.toml. `place add <URL>` clones a repo and lists it, \
`place sync` clones what's missing and fast-forwards what's clean, and \
`place status` shows where every repo stands. place never switches branches, \
rebases or stashes. Docs: https://github.com/ungood/place";

const EXAMPLES: &str = "Examples:
    Make the current directory a workplace
    $ place init

    Clone a repo and list it in place.toml
    $ place add https://github.com/ungood/place.git

    Track a branch, and list it only on this machine
    $ place add https://github.com/ungood/agents.git -b main -e local

    Clone missing repos and fast-forward clean ones
    $ place sync

    See which repos have work in progress
    $ place status

    Sync the workplace in another directory, with its ci environment
    $ place -C ~/src/factory -E ci sync";

/// Manage workplaces: directories of independent git repositories.
#[derive(Debug, Parser)]
#[command(
    name = "place",
    version,
    long_about = ABOUT,
    after_long_help = EXAMPLES,
    override_usage = "place [FLAGS] <COMMAND>",
    disable_help_flag = true,
    disable_version_flag = true,
    next_help_heading = "Flags"
)]
struct Cli {
    /// Run as if place were started in DIR.
    #[arg(short = 'C', long = "cd", global = true, value_name = "DIR")]
    directory: Option<PathBuf>,

    // Not global, because `place add` has its own -e/--env.
    /// Load the place.<ENV>.toml files (same as PLACE_ENV).
    ///
    /// Repeat the flag or separate names with commas to load several
    /// environments. A later one wins over an earlier one. Give it before
    /// the command, as in `place -E ci sync`.
    #[arg(
        short = 'E',
        long = "env",
        env = "PLACE_ENV",
        hide_env = true,
        value_delimiter = ',',
        value_name = "ENV"
    )]
    envs: Vec<String>,

    /// How many repos to work on at once (same as PLACE_JOBS).
    ///
    /// Defaults to the number of CPUs.
    #[arg(
        short = 'j',
        long,
        global = true,
        env = "PLACE_JOBS",
        hide_env = true,
        value_name = "JOBS"
    )]
    jobs: Option<usize>,

    /// Suppress non-error messages (same as PLACE_QUIET=1).
    #[arg(short, long, global = true, env = "PLACE_QUIET", hide_env = true)]
    quiet: bool,

    /// Show the git commands place runs.
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Print help.
    #[arg(short, long, global = true, action = ArgAction::Help)]
    help: Option<bool>,

    /// Print the version.
    #[arg(short = 'V', long, action = ArgAction::Version)]
    version: Option<bool>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Make the current directory a workplace.
    ///
    /// Creates an empty place.toml. Fails if the directory already has
    /// place configuration.
    #[command(override_usage = "place init", next_help_heading = "Flags")]
    Init,

    /// Clone a repo and list it, or list a checkout that's already here.
    ///
    /// Given a URL, add clones the repo into PATH (by default, the repo's
    /// name, as git clone does) and records the URL exactly as given. Given
    /// the paths of checkouts in the workplace, add lists them with the URL
    /// of their origin. Entries go to place.toml, which add edits in place,
    /// keeping comments and formatting.
    #[command(
        override_usage = "place add [FLAGS] <URL> [PATH]\n       place add <PATH>...",
        next_help_heading = "Flags",
        after_long_help = "Examples:
    $ place add https://github.com/ungood/place.git
    $ place add https://github.com/ros-infrastructure/vcs2l.git vendor/vcs2l -b 1.1.7
    $ place add agents tools/scripts"
    )]
    Add {
        /// A URL to clone and an optional PATH, or paths of checkouts to list.
        #[arg(required = true, value_name = "URL|PATH", help_heading = "Arguments")]
        targets: Vec<String>,

        /// Track this branch, tag or commit, and check it out.
        #[arg(short, long, value_name = "REF")]
        branch: Option<String>,

        /// Write to place.<ENV>.toml instead of place.toml.
        ///
        /// `-e local` writes place.local.toml, for repos only this machine
        /// needs.
        #[arg(short, long = "env", value_name = "ENV")]
        env: Option<String>,
    },

    /// Unlist repos, and delete their checkouts when nothing would be lost.
    ///
    /// A checkout is deleted only when its working tree is clean, every
    /// local branch is on a remote, and it has no stashes. If any check
    /// fails, nothing is removed.
    #[command(
        visible_alias = "rm",
        override_usage = "place remove [FLAGS] <PATH>...",
        next_help_heading = "Flags"
    )]
    Remove {
        /// Paths of the repos to remove.
        #[arg(required = true, value_name = "PATH", help_heading = "Arguments")]
        paths: Vec<String>,

        /// Remove only the entry and leave the checkout on disk.
        #[arg(long)]
        keep: bool,

        /// Delete the checkout even if work would be lost.
        #[arg(long, conflicts_with = "keep")]
        force: bool,
    },

    /// Show the state of every repo, including nested workplaces.
    ///
    /// status reads the working copies and the last fetch, and doesn't touch
    /// the network unless --fetch is given.
    #[command(
        override_usage = "place status [FLAGS] [PATH]...",
        next_help_heading = "Flags"
    )]
    Status {
        /// Show only these repos, and the repos of nested workplaces in them.
        #[arg(value_name = "PATH", help_heading = "Arguments")]
        paths: Vec<String>,

        /// Fetch every repo before reporting.
        #[arg(long)]
        fetch: bool,

        /// Print JSON for other tools.
        #[arg(long)]
        json: bool,

        /// Exit with 1 when any repo isn't clean and on its ref.
        #[arg(long)]
        exit_code: bool,
    },

    /// Clone missing repos, fetch, and fast-forward clean ones.
    ///
    /// sync never switches branches, and leaves work in progress alone. It
    /// fails for problems that need attention: a failed clone or fetch, a
    /// checkout of the wrong repo, or a branch that has diverged.
    #[command(
        override_usage = "place sync [FLAGS] [PATH]...",
        next_help_heading = "Flags"
    )]
    Sync {
        /// Sync only these repos, and the repos of nested workplaces in them.
        #[arg(value_name = "PATH", help_heading = "Arguments")]
        paths: Vec<String>,
    },

    /// Check the workplace's configuration and layout for problems.
    #[command(override_usage = "place doctor", next_help_heading = "Flags")]
    Doctor,
}

/// What every command needs: where to run and how.
#[derive(Debug)]
pub struct Context {
    pub dir: PathBuf,
    pub envs: Vec<String>,
    pub jobs: usize,
    pub quiet: bool,
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
    git::set_verbose(cli.verbose);
    let ctx = Context {
        dir,
        envs: cli.envs.into_iter().filter(|e| !e.is_empty()).collect(),
        jobs,
        quiet: cli.quiet,
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
