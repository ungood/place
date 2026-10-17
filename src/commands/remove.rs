use std::path::Path;
use std::process::ExitCode;

use crate::config::{self, Entry};
use crate::error::Error;
use crate::git::{self, WorkingCopy};
use crate::workplace::{Workplace, clean_path};
use crate::{Context, output};

pub fn run(ctx: &Context, paths: &[String], keep: bool, force: bool) -> Result<ExitCode, Error> {
    let workplace = Workplace::open(&ctx.dir, &ctx.envs)?;

    // Check everything first, so a failure removes nothing.
    let mut removals: Vec<(String, &Entry)> = Vec::new();
    for path in paths {
        let path = clean_path(path);
        let entry = workplace
            .config
            .repos
            .get(&path)
            .ok_or_else(|| Error::NotListed { path: path.clone() })?;
        if let Some(file) = entry.files.iter().find(|f| config::is_repos_file(f)) {
            return Err(Error::ListedInReposFile {
                path,
                file: file.clone(),
            });
        }
        if !keep && !force {
            let found = lost_work(&workplace.dir.join(&path), &ctx.envs)?;
            if !found.is_empty() {
                return Err(Error::WouldLoseWork {
                    path,
                    found: join(&found),
                });
            }
        }
        removals.push((path, entry));
    }

    for (path, entry) in removals {
        for file in &entry.files {
            config::remove_entry(&workplace.dir, file, &path)?;
        }
        let dir = workplace.dir.join(&path);
        if !keep && dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|source| Error::Write {
                file: path.clone(),
                source,
            })?;
            output::progress(ctx.quiet, &path, "removed, and its checkout deleted");
        } else {
            output::progress(ctx.quiet, &path, "removed");
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// What deleting the checkout at `dir` would lose, including in the repos
/// of a nested workplace inside it.
fn lost_work(dir: &Path, envs: &[String]) -> Result<Vec<String>, Error> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    if !git::is_checkout(dir) {
        return Ok(vec!["files that aren't in a git checkout".into()]);
    }
    let mut found = Vec::new();
    let copy = WorkingCopy::read(dir)?;
    if copy.changes {
        found.push("uncommitted changes".into());
    }
    if copy.untracked {
        found.push("untracked files".into());
    }
    let branches = git::unpushed_branches(dir)?;
    if !branches.is_empty() {
        let names: Vec<String> = branches.iter().map(|b| format!("\"{b}\"")).collect();
        found.push(format!(
            "commits on {} that aren't on any remote",
            join(&names)
        ));
    }
    if git::Refs::read(dir)?.has_stash() {
        found.push("stashed changes".into());
    }
    if config::is_workplace(dir) {
        let nested = config::load(dir, envs)?;
        for path in nested.repos.keys() {
            for work in lost_work(&dir.join(path), envs)? {
                found.push(format!("{work} in \"{path}\""));
            }
        }
    }
    Ok(found)
}

/// Join phrases as "a", "a and b", or "a, b and c".
fn join(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}
