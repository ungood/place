//! Every error and warning place reports. The wording is part of the spec in
//! `features/`, so keep the two in step.

use miette::{Diagnostic, NamedSource, SourceSpan};

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum Error {
    // The workplace.
    #[error("This directory isn't a workplace.")]
    #[diagnostic(help("{help}"))]
    NotAWorkplace { help: String },

    #[error("There's no directory at {dir}.")]
    #[diagnostic(help("-C takes the path of a workplace directory."))]
    NoSuchDirectory { dir: String },

    #[error("This directory is already a workplace: it has {file}.")]
    #[diagnostic(help("{help}"))]
    AlreadyAWorkplace { file: String, help: String },

    #[error("Couldn't read {file}.")]
    Read {
        file: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Couldn't write {file}.")]
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
    #[diagnostic(help(
        "Every repo needs a url, for example: url = \"https://github.com/ungood/place.git\""
    ))]
    NoUrl { path: String, file: String },

    #[error("\"{path}\" in {file} {problem}.")]
    #[diagnostic(help("Each repo is a table with a url and an optional ref, both strings."))]
    BadEntry {
        path: String,
        file: String,
        problem: String,
    },

    #[error("{file} has an unknown setting \"{key}\".")]
    #[diagnostic(help("Workplace files only hold [repos.<path>] tables."))]
    UnknownSetting { file: String, key: String },

    #[error("{file} lists \"{path}\" as {article} {kind} repository, and place only supports git.")]
    #[diagnostic(help("Remove the entry from {file}, or manage that checkout with another tool."))]
    NotGit {
        file: String,
        path: String,
        kind: String,
        article: &'static str,
    },

    // add
    #[error("\"{path}\" isn't a git checkout, so there's nothing to adopt.")]
    #[diagnostic(help(
        "To clone a repo there instead, move \"{path}\" aside and run \"place add <url> {path}\"."
    ))]
    NothingToAdopt { path: String },

    #[error("\"{target}\" isn't a URL, and there's no directory by that name.")]
    #[diagnostic(help(
        "Give a clone URL, such as https://github.com/owner/repo.git, or the path of a checkout in the workplace."
    ))]
    NoSuchPath { target: String },

    #[error("\"{path}\" is outside the workplace.")]
    #[diagnostic(help(
        "place add lists checkouts inside the workplace. To clone a repo from a local path, give it as a URL, such as file:///path/to/repo.git."
    ))]
    OutsideWorkplace { path: String },

    #[error("\"place add\" takes one URL and an optional path.")]
    #[diagnostic(help("To adopt several checkouts at once, give only their paths."))]
    AddArguments,

    #[error("--branch only applies when cloning a URL.")]
    #[diagnostic(help(
        "An adopted checkout tracks the remote's default branch. Set ref in its entry to track something else."
    ))]
    BranchOnAdopt,

    #[error("Can't tell what to call the checkout for {url}.")]
    #[diagnostic(help("Give a path: \"place add {url} <path>\"."))]
    NoDefaultPath { url: String },

    #[error("\"{path}\" has no remote, so there's no URL to record.")]
    #[diagnostic(help(
        "Add one with \"git -C {path} remote add origin <url>\", then run \"place add {path}\" again."
    ))]
    NoRemote { path: String },

    #[error("\"{path}\" has several remotes and none of them is called origin.")]
    #[diagnostic(help(
        "Rename the one place should record with \"git -C {path} remote rename <name> origin\"."
    ))]
    AmbiguousRemote { path: String },

    #[error("\"{path}\" is already listed in {file}.")]
    #[diagnostic(help("To clone it again under another name, run \"place add <url> <path>\"."))]
    AlreadyListed { path: String, file: String },

    #[error("Can't clone into \"{path}\": that directory already exists.")]
    #[diagnostic(help(
        "Choose another path with \"place add <url> <path>\". If \"{path}\" is a checkout, \"place add {path}\" adopts it."
    ))]
    DirectoryExists { path: String },

    #[error("Couldn't clone \"{path}\" from {url}.")]
    #[diagnostic(help(
        "Check the URL and that you have access to it. git's own message is below.\n\n{message}"
    ))]
    CloneFailed {
        path: String,
        url: String,
        message: String,
    },

    // remove
    #[error("\"{path}\" isn't listed in this workplace.")]
    #[diagnostic(help("\"place status\" shows the repos the workplace lists."))]
    NotListed { path: String },

    #[error("\"{path}\" is listed in {file}, and place doesn't edit .repos files.")]
    #[diagnostic(help("Delete the entry from {file} yourself, then delete the checkout."))]
    ListedInReposFile { path: String, file: String },

    #[error("\"{path}\" has {found}, so place didn't remove it.")]
    #[diagnostic(help(
        "Push or discard that work first. \"place remove {path} --keep\" removes only the entry, and --force deletes the checkout anyway."
    ))]
    WouldLoseWork { path: String, found: String },

    // sync
    #[error("\"{path}\" is a checkout of {actual}, but {file} expects {expected}.")]
    #[diagnostic(help(
        "Point its origin at the expected URL with \"git -C {path} remote set-url origin {expected}\", or change the entry in {file}."
    ))]
    WrongRemote {
        path: String,
        actual: String,
        expected: String,
        file: String,
    },

    #[error("\"{path}\" exists but isn't a git checkout, so place can't sync it.")]
    #[diagnostic(help(
        "Move \"{path}\" aside, then run \"place sync\" again to clone the repo there."
    ))]
    NotARepo { path: String },

    #[error("Couldn't fetch \"{path}\".")]
    #[diagnostic(help(
        "Check the URL and that you have access to it. git's own message is below.\n\n{message}"
    ))]
    FetchFailed { path: String, message: String },

    #[error("\"{path}\": \"{branch}\" has diverged from \"{upstream}\", so place left it alone.")]
    #[diagnostic(help(
        "Rebase or merge \"{branch}\" in \"{path}\", then run \"place sync\" again."
    ))]
    Diverged {
        path: String,
        branch: String,
        upstream: String,
    },

    // doctor
    /// `repository` is "The workplace repository" or "The repository
    /// \"place\"", `paths` and `lines` are lists such as "\"a\" and \"b\"",
    /// and `gitignore` names the file to edit.
    #[error("{repository} doesn't ignore {paths}, so git shows {them} as untracked.")]
    #[diagnostic(help("Add {lines} to {gitignore}."))]
    NotIgnored {
        repository: String,
        paths: String,
        them: &'static str,
        lines: String,
        gitignore: String,
    },

    #[error("git {version} is too old. place needs git {required} or later.")]
    #[diagnostic(help("Update git, then run \"place doctor\" again."))]
    OldGit { version: String, required: String },

    // git
    #[error("Couldn't run git.")]
    #[diagnostic(help("place needs git. Install it and make sure it's on your PATH."))]
    GitMissing {
        #[source]
        source: std::io::Error,
    },

    #[error("\"{command}\" failed in {dir}.")]
    #[diagnostic(help("git said: {message}"))]
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
    ListedTwice {
        path: String,
        shadowed: String,
        winner: String,
    },

    #[error(
        "\"{path}\" is a git checkout that no workplace file lists. Run \"place add {path}\" to list it."
    )]
    Unlisted { path: String },
}
