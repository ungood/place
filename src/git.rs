//! Everything place asks of git. place shells out to the git CLI, so the
//! user's credential helpers, SSH agent and `insteadOf` rules apply.
//!
//! This module is the version control interface. Git is its only
//! implementation, and the rest of place doesn't run git directly.

use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};

use console::style;

use crate::error::Error;

static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Print each git command before running it, for `--verbose`.
pub fn set_verbose(verbose: bool) {
    VERBOSE.store(verbose, Ordering::Relaxed);
}

/// Run git in `dir` and return its output, whatever the exit status.
fn run(dir: &Path, args: &[&str]) -> Result<Output, Error> {
    if VERBOSE.load(Ordering::Relaxed) {
        let line = format!("$ git -C {} {}", dir.display(), args.join(" "));
        eprintln!("{}", style(line).dim());
    }
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|source| Error::GitMissing { source })
}

/// Run git in `dir` and return its stdout, failing if git fails.
fn stdout(dir: &Path, args: &[&str]) -> Result<String, Error> {
    let output = run(dir, args)?;
    if !output.status.success() {
        return Err(failed(dir, args, &output));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn failed(dir: &Path, args: &[&str], output: &Output) -> Error {
    Error::GitFailed {
        command: format!("git {}", args.join(" ")),
        dir: dir.display().to_string(),
        message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    }
}

/// The installed git's version, such as `(2, 55)`.
pub fn version() -> Result<(u32, u32), Error> {
    let text = stdout(Path::new("."), &["--version"])?;
    let number = text
        .split_whitespace()
        .find(|w| w.starts_with(|c: char| c.is_ascii_digit()))
        .unwrap_or_default();
    let mut parts = number.split('.').map(|p| p.parse().unwrap_or(0));
    Ok((parts.next().unwrap_or(0), parts.next().unwrap_or(0)))
}

/// Whether `dir` is the root of a git checkout.
pub fn is_checkout(dir: &Path) -> bool {
    dir.join(".git").exists()
}

/// What a configured ref names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Branch(String),
    Tag(String),
    Commit(String),
}

fn looks_like_commit(name: &str) -> bool {
    (7..=40).contains(&name.len()) && name.chars().all(|c| c.is_ascii_hexdigit())
}

/// Clone `url` into `dest`, then check out `reference` if one is given.
/// Every clone goes through here, so a shared repo cache can be added later.
/// Returns what git printed. On failure, the error is git's own message.
pub fn clone(url: &str, dest: &Path, reference: Option<&str>) -> Result<String, String> {
    let parent = dest.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    // Clone by name from the parent, so git says "Cloning into 'place'"
    // rather than naming an absolute path.
    let dest_arg = dest.file_name().map_or_else(
        || dest.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    );
    let mut args = vec!["clone"];
    if reference.is_some() {
        args.push("--no-checkout");
    }
    args.extend(["--", url, &dest_arg]);
    let output = run(parent, &args).map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let printed = printed(&output);
    let Some(reference) = reference else {
        return Ok(printed);
    };
    let refs = Refs::read(dest).map_err(|e| e.to_string())?;
    let checkout: Vec<String> = match refs.resolve(reference, "origin") {
        Some(Target::Branch(name)) if refs.get(&format!("refs/heads/{name}")).is_some() => {
            vec!["checkout".into(), "-q".into(), "-f".into(), name]
        }
        Some(Target::Branch(name)) => vec![
            "checkout".into(),
            "-q".into(),
            "-f".into(),
            "-b".into(),
            name.clone(),
            "--track".into(),
            format!("origin/{name}"),
        ],
        Some(Target::Tag(name)) => vec![
            "checkout".into(),
            "-q".into(),
            "-f".into(),
            "--detach".into(),
            format!("refs/tags/{name}"),
        ],
        Some(Target::Commit(hash)) => vec![
            "checkout".into(),
            "-q".into(),
            "-f".into(),
            "--detach".into(),
            hash,
        ],
        None => {
            return Err(format!(
                "The remote has no branch or tag \"{reference}\", and it isn't a commit hash."
            ));
        }
    };
    let args: Vec<&str> = checkout.iter().map(String::as_str).collect();
    let output = run(dest, &args).map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(printed)
}

/// Everything git printed, stdout then stderr, without trailing blank lines.
fn printed(output: &Output) -> String {
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    text.trim_end().to_string()
}

/// The remotes of a checkout, as `(name, url)` with URLs as configured.
pub fn remotes(dir: &Path) -> Result<Vec<(String, String)>, Error> {
    let output = run(dir, &["config", "--get-regexp", r"^remote\..*\.url$"])?;
    // git config exits 1 when nothing matches.
    if !output.status.success() && output.status.code() != Some(1) {
        return Err(failed(dir, &["config", "--get-regexp"], &output));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let (key, url) = line.split_once(' ')?;
            let name = key.strip_prefix("remote.")?.strip_suffix(".url")?;
            Some((name.to_string(), url.to_string()))
        })
        .collect())
}

/// Where place records the remote's default branch as of the last fetch.
fn remote_head_ref(remote: &str) -> String {
    format!("refs/place/{remote}/HEAD")
}

/// Fetch `remote`. With `track_default`, also record which branch the
/// remote's HEAD points to, so status can tell when the default moved.
/// Returns what git printed, which is nothing when nothing changed. On
/// failure, the error is git's own message.
pub fn fetch(dir: &Path, remote: &str, track_default: bool) -> Result<String, String> {
    let output = run(dir, &["fetch", "--prune", remote]).map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let printed = printed(&output);
    if !track_default {
        return Ok(printed);
    }
    let output = run(dir, &["ls-remote", "--symref", remote, "HEAD"]).map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&output.stdout);
    let branch = text.lines().find_map(|line| {
        line.strip_prefix("ref: refs/heads/")?
            .strip_suffix("\tHEAD")
            .map(String::from)
    });
    if let Some(branch) = branch {
        let target = format!("refs/remotes/{remote}/{branch}");
        stdout(dir, &["symbolic-ref", &remote_head_ref(remote), &target])
            .map_err(|e| e.to_string())?;
    }
    Ok(printed)
}

/// A ref and what it points to.
#[derive(Debug, Clone, Default)]
pub struct RefInfo {
    pub oid: String,
    /// For an annotated tag, the commit it points to.
    pub peeled: Option<String>,
    /// For a symbolic ref, the ref it points to.
    pub symref: Option<String>,
}

impl RefInfo {
    /// The commit this ref names.
    pub fn commit(&self) -> &str {
        self.peeled.as_deref().unwrap_or(&self.oid)
    }
}

/// Every ref in a checkout.
#[derive(Debug, Default)]
pub struct Refs(HashMap<String, RefInfo>);

impl Refs {
    pub fn read(dir: &Path) -> Result<Self, Error> {
        let text = stdout(
            dir,
            &[
                "for-each-ref",
                "--format=%(refname)%00%(objectname)%00%(*objectname)%00%(symref)",
            ],
        )?;
        let refs = text
            .lines()
            .filter_map(|line| {
                let mut fields = line.split('\0');
                let name = fields.next()?.to_string();
                let oid = fields.next()?.to_string();
                let nonempty = |s: Option<&str>| s.filter(|s| !s.is_empty()).map(String::from);
                let peeled = nonempty(fields.next());
                let symref = nonempty(fields.next());
                Some((
                    name,
                    RefInfo {
                        oid,
                        peeled,
                        symref,
                    },
                ))
            })
            .collect();
        Ok(Self(refs))
    }

    pub fn get(&self, name: &str) -> Option<&RefInfo> {
        self.0.get(name)
    }

    /// Resolve a configured ref against `remote`'s branches and the tags,
    /// in the order `git rev-parse` uses: a qualified name, then a tag, then
    /// a branch, then a commit hash.
    pub fn resolve(&self, reference: &str, remote: &str) -> Option<Target> {
        if let Some(name) = reference.strip_prefix("refs/heads/") {
            return Some(Target::Branch(name.to_string()));
        }
        if let Some(name) = reference.strip_prefix("refs/tags/") {
            return Some(Target::Tag(name.to_string()));
        }
        if self.get(&format!("refs/tags/{reference}")).is_some() {
            return Some(Target::Tag(reference.to_string()));
        }
        if self
            .get(&format!("refs/remotes/{remote}/{reference}"))
            .is_some()
        {
            return Some(Target::Branch(reference.to_string()));
        }
        looks_like_commit(reference).then(|| Target::Commit(reference.to_lowercase()))
    }

    /// The branch a symbolic ref such as `refs/remotes/origin/HEAD` points
    /// to, without the remote's prefix.
    pub fn symref_branch(&self, name: &str, remote: &str) -> Option<String> {
        let target = self.get(name)?.symref.as_deref()?;
        target
            .strip_prefix(&format!("refs/remotes/{remote}/"))
            .map(String::from)
    }

    /// The remote's default branch when the checkout was cloned.
    pub fn cloned_default(&self, remote: &str) -> Option<String> {
        self.symref_branch(&format!("refs/remotes/{remote}/HEAD"), remote)
    }

    /// The remote's default branch as of the last fetch by place.
    pub fn fetched_default(&self, remote: &str) -> Option<String> {
        self.symref_branch(&remote_head_ref(remote), remote)
    }

    pub fn has_stash(&self) -> bool {
        self.get("refs/stash").is_some()
    }
}

/// The working copy: HEAD and changes.
#[derive(Debug, Default)]
pub struct WorkingCopy {
    /// The checked-out branch, or `None` when HEAD is detached.
    pub branch: Option<String>,
    pub head: Option<String>,
    /// Staged or unstaged changes to tracked files.
    pub changes: bool,
    pub untracked: bool,
}

impl WorkingCopy {
    /// Read the working copy at `dir`. Changes under `skip`, such as the
    /// checkouts of a nested workplace, don't count.
    pub fn read(dir: &Path, skip: &[String]) -> Result<Self, Error> {
        let excludes: Vec<String> = skip
            .iter()
            .map(|path| format!(":(top,exclude,literal){path}"))
            .collect();
        let mut args = vec!["status", "--porcelain=v2", "--branch"];
        if !excludes.is_empty() {
            args.extend(["--", "."]);
            args.extend(excludes.iter().map(String::as_str));
        }
        let text = stdout(dir, &args)?;
        let mut copy = WorkingCopy::default();
        for line in text.lines() {
            if let Some(head) = line.strip_prefix("# branch.head ") {
                copy.branch = (head != "(detached)").then(|| head.to_string());
            } else if let Some(oid) = line.strip_prefix("# branch.oid ") {
                copy.head = (oid != "(initial)").then(|| oid.to_string());
            } else if line.starts_with("? ") {
                copy.untracked = true;
            } else if !line.starts_with('#') && !line.starts_with("! ") {
                copy.changes = true;
            }
        }
        Ok(copy)
    }

    pub fn dirty(&self) -> bool {
        self.changes || self.untracked
    }
}

/// How many commits `local` has that `remote` doesn't, and the reverse.
pub fn ahead_behind(dir: &Path, local: &str, remote: &str) -> Result<(u32, u32), Error> {
    let range = format!("{local}...{remote}");
    let text = stdout(dir, &["rev-list", "--left-right", "--count", &range])?;
    let mut counts = text.split_whitespace().map(|n| n.parse().unwrap_or(0));
    Ok((counts.next().unwrap_or(0), counts.next().unwrap_or(0)))
}

/// Fast-forward the checked-out branch to `to`, and return what git printed.
pub fn fast_forward_head(dir: &Path, to: &str) -> Result<String, Error> {
    let args = ["merge", "--ff-only", to];
    let output = run(dir, &args)?;
    if !output.status.success() {
        return Err(failed(dir, &args, &output));
    }
    Ok(printed(&output))
}

/// Fast-forward `branch`, which isn't checked out, to its remote-tracking
/// branch, and return what git printed. Fetching from the repo itself moves
/// the branch without touching the working tree, and git refuses anything
/// that isn't a fast-forward or a branch checked out in any worktree.
pub fn fast_forward_branch(dir: &Path, remote: &str, branch: &str) -> Result<String, Error> {
    let refspec = format!("refs/remotes/{remote}/{branch}:refs/heads/{branch}");
    let args = ["fetch", ".", &refspec];
    let output = run(dir, &args)?;
    if !output.status.success() {
        return Err(failed(dir, &args, &output));
    }
    Ok(printed(&output))
}

/// Local branches with commits that no remote-tracking branch contains.
pub fn unpushed_branches(dir: &Path) -> Result<Vec<String>, Error> {
    let text = stdout(
        dir,
        &[
            "log",
            "--branches",
            "--not",
            "--remotes",
            "--source",
            "--format=%S",
        ],
    )?;
    let mut branches: Vec<String> = text.lines().map(String::from).collect();
    branches.sort();
    branches.dedup();
    Ok(branches)
}

/// Which of `paths` (relative to `dir`) the repository at `dir` ignores.
pub fn ignored(dir: &Path, paths: &[String]) -> Result<Vec<String>, Error> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let mut args = vec!["check-ignore", "--"];
    args.extend(paths.iter().map(String::as_str));
    let output = run(dir, &args)?;
    // check-ignore exits 1 when nothing is ignored.
    if !output.status.success() && output.status.code() != Some(1) {
        return Err(failed(dir, &["check-ignore"], &output));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(String::from)
        .collect())
}
