//! Every error and warning place reports. The wording is part of the spec in
//! `features/`, so keep the two in step.

use miette::{Diagnostic, NamedSource, SourceSpan};

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum Error {
    // The workplace.
    #[error("This directory isn't a workplace.")]
    #[diagnostic(code(place::not_a_workplace), help("{help}"))]
    NotAWorkplace { help: String },

    #[error("There's no directory at {dir}.")]
    #[diagnostic(
        code(place::no_such_directory),
        help("-C takes the path of a workplace directory.")
    )]
    NoSuchDirectory { dir: String },

    #[error("This directory is already a workplace: it has {file}.")]
    #[diagnostic(code(place::already_a_workplace), help("{help}"))]
    AlreadyAWorkplace { file: String, help: String },

    #[error("Couldn't read {file}.")]
    #[diagnostic(code(place::read))]
    Read {
        file: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Couldn't write {file}.")]
    #[diagnostic(code(place::write))]
    Write {
        file: String,
        #[source]
        source: std::io::Error,
    },

    // Configuration.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Syntax(Box<SyntaxError>),

    #[error("\"{path}\" in {file} has no url.")]
    #[diagnostic(
        code(place::no_url),
        help("Every repo needs a url, for example: url = \"https://github.com/ungood/place.git\"")
    )]
    NoUrl { path: String, file: String },

    #[error("\"{path}\" in {file} {problem}.")]
    #[diagnostic(
        code(place::bad_entry),
        help("Each repo is a table with a url and an optional ref, both strings.")
    )]
    BadEntry {
        path: String,
        file: String,
        problem: String,
    },

    #[error("{file} has an unknown setting \"{key}\".")]
    #[diagnostic(
        code(place::unknown_setting),
        help("Workplace files only hold [repos.<path>] tables.")
    )]
    UnknownSetting { file: String, key: String },

    #[error("{file} lists \"{path}\" as {article} {kind} repository, and place only supports git.")]
    #[diagnostic(
        code(place::not_git),
        help("Remove the entry from {file}, or manage that checkout with another tool.")
    )]
    NotGit {
        file: String,
        path: String,
        kind: String,
        article: &'static str,
    },

    // add
    #[error("\"{path}\" isn't a git checkout, so there's nothing to adopt.")]
    #[diagnostic(
        code(place::not_a_checkout),
        help(
            "To clone a repo there instead, move \"{path}\" aside and run \"place add <url> {path}\"."
        )
    )]
    NothingToAdopt { path: String },

    #[error("\"{target}\" isn't a URL, and there's no directory by that name.")]
    #[diagnostic(
        code(place::no_such_path),
        help(
            "Give a clone URL, such as https://github.com/owner/repo.git, or the path of a checkout in the workplace."
        )
    )]
    NoSuchPath { target: String },

    #[error("\"place add\" takes one URL and an optional path.")]
    #[diagnostic(
        code(place::add_arguments),
        help("To adopt several checkouts at once, give only their paths.")
    )]
    AddArguments,

    #[error("--branch only applies when cloning a URL.")]
    #[diagnostic(
        code(place::branch_on_adopt),
        help(
            "An adopted checkout tracks the remote's default branch. Set ref in its entry to track something else."
        )
    )]
    BranchOnAdopt,

    #[error("Can't tell what to call the checkout for {url}.")]
    #[diagnostic(
        code(place::no_default_path),
        help("Give a path: \"place add {url} <path>\".")
    )]
    NoDefaultPath { url: String },

    #[error("\"{path}\" has no remote, so there's no URL to record.")]
    #[diagnostic(
        code(place::no_remote),
        help(
            "Add one with \"git -C {path} remote add origin <url>\", then run \"place add {path}\" again."
        )
    )]
    NoRemote { path: String },

    #[error("\"{path}\" has several remotes and none of them is called origin.")]
    #[diagnostic(
        code(place::ambiguous_remote),
        help(
            "Rename the one place should record with \"git -C {path} remote rename <name> origin\"."
        )
    )]
    AmbiguousRemote { path: String },

    #[error("\"{path}\" is already listed in {file}.")]
    #[diagnostic(
        code(place::already_listed),
        help("To clone it again under another name, run \"place add <url> <path>\".")
    )]
    AlreadyListed { path: String, file: String },

    #[error("Can't clone into \"{path}\": that directory already exists.")]
    #[diagnostic(
        code(place::directory_exists),
        help(
            "Choose another path with \"place add <url> <path>\". If \"{path}\" is a checkout, \"place add {path}\" adopts it."
        )
    )]
    DirectoryExists { path: String },

    #[error("Couldn't clone \"{path}\" from {url}.")]
    #[diagnostic(
        code(place::clone_failed),
        help(
            "Check the URL and that you have access to it. git's own message is below.\n\n{message}"
        )
    )]
    CloneFailed {
        path: String,
        url: String,
        message: String,
    },

    // remove
    #[error("\"{path}\" isn't listed in this workplace.")]
    #[diagnostic(
        code(place::not_listed),
        help("\"place status\" shows the repos the workplace lists.")
    )]
    NotListed { path: String },

    #[error("\"{path}\" is listed in {file}, and place doesn't edit .repos files.")]
    #[diagnostic(
        code(place::listed_in_repos_file),
        help("Delete the entry from {file} yourself, then delete the checkout.")
    )]
    ListedInReposFile { path: String, file: String },

    #[error("\"{path}\" has {found}, so place didn't remove it.")]
    #[diagnostic(
        code(place::would_lose_work),
        help(
            "Push or discard that work first. \"place remove {path} --keep\" removes only the entry, and --force deletes the checkout anyway."
        )
    )]
    WouldLoseWork { path: String, found: String },

    // sync
    #[error("\"{path}\" is a checkout of {actual}, but {file} expects {expected}.")]
    #[diagnostic(
        code(place::wrong_remote),
        help(
            "Point its origin at the expected URL with \"git -C {path} remote set-url origin {expected}\", or change the entry in {file}."
        )
    )]
    WrongRemote {
        path: String,
        actual: String,
        expected: String,
        file: String,
    },

    #[error("\"{path}\" exists but isn't a git checkout, so place can't sync it.")]
    #[diagnostic(
        code(place::not_a_repo),
        help("Move \"{path}\" aside, then run \"place sync\" again to clone the repo there.")
    )]
    NotARepo { path: String },

    #[error("Couldn't fetch \"{path}\".")]
    #[diagnostic(
        code(place::fetch_failed),
        help(
            "Check the URL and that you have access to it. git's own message is below.\n\n{message}"
        )
    )]
    FetchFailed { path: String, message: String },

    #[error("\"{path}\": \"{branch}\" has diverged from \"{upstream}\", so place left it alone.")]
    #[diagnostic(
        code(place::diverged),
        help("Rebase or merge \"{branch}\" in \"{path}\", then run \"place sync\" again.")
    )]
    Diverged {
        path: String,
        branch: String,
        upstream: String,
    },

    // doctor
    #[error("The workplace repository doesn't ignore \"{path}\", so git shows it as untracked.")]
    #[diagnostic(
        code(place::not_ignored),
        help("Add \"/{path}/\" to the workplace's .gitignore.")
    )]
    NotIgnored { path: String },

    #[error("git {version} is too old. place needs git {required} or later.")]
    #[diagnostic(
        code(place::old_git),
        help("Update git, then run \"place doctor\" again.")
    )]
    OldGit { version: String, required: String },

    // git
    #[error("Couldn't run git.")]
    #[diagnostic(
        code(place::git_missing),
        help("place needs git. Install it and make sure it's on your PATH.")
    )]
    GitMissing {
        #[source]
        source: std::io::Error,
    },

    #[error("\"{command}\" failed in {dir}.")]
    #[diagnostic(code(place::git_failed), help("git said: {message}"))]
    GitFailed {
        command: String,
        dir: String,
        message: String,
    },
}

/// A file that doesn't parse. It holds the file's text for the snippet, so
/// it's boxed to keep `Error` small.
#[derive(Debug, thiserror::Error, Diagnostic)]
#[error("{file} has a syntax error on line {line}.")]
#[diagnostic(code(place::syntax))]
pub struct SyntaxError {
    pub file: String,
    pub line: usize,
    #[source_code]
    pub src: NamedSource<String>,
    #[label("{detail}")]
    pub span: SourceSpan,
    pub detail: String,
}

#[derive(Debug, thiserror::Error, Diagnostic)]
#[diagnostic(severity(Warning))]
pub enum Warning {
    #[error("\"{path}\" is listed in both {shadowed} and {winner}. The entry in {winner} is used.")]
    #[diagnostic(code(place::listed_twice))]
    ListedTwice {
        path: String,
        shadowed: String,
        winner: String,
    },

    #[error(
        "\"{path}\" is a git checkout that no workplace file lists. Run \"place add {path}\" to list it."
    )]
    #[diagnostic(code(place::unlisted))]
    Unlisted { path: String },
}
