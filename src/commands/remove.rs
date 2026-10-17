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
            let found = lost_work(&workplace.dir.join(&path))?;
            if !found.is_empty() {
                return Err(Error::WouldLoseWork {
                    path,
                    found: output::join(&found),
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
            remove_empty_parents(&workplace.dir, &dir);
            output::progress(ctx.quiet, &path, "removed, and its checkout deleted");
        } else {
            output::progress(ctx.quiet, &path, "removed");
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// What deleting the checkout at `dir` would lose, including in the repos
/// of a nested workplace inside it, whatever their environment.
fn lost_work(dir: &Path) -> Result<Vec<String>, Error> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    if !git::is_checkout(dir) {
        return Ok(vec!["files that aren't in a git checkout".into()]);
    }
    let nested = if config::is_workplace(dir) {
        config::listed_paths(dir)
    } else {
        Vec::new()
    };
    let mut found = Vec::new();
    let copy = WorkingCopy::read(dir, &nested)?;
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
            output::join(&names)
        ));
    }
    if git::Refs::read(dir)?.has_stash() {
        found.push("stashed changes".into());
    }
    for path in &nested {
        for work in lost_work(&dir.join(path))? {
            found.push(format!("{work} in \"{path}\""));
        }
    }
    Ok(found)
}

/// Delete the directories between `dir` and the workplace that are now
/// empty, such as `vendor` after removing `vendor/place`.
fn remove_empty_parents(workplace: &Path, dir: &Path) {
    let mut parent = dir.parent();
    while let Some(current) = parent {
        // remove_dir only deletes an empty directory.
        if current == workplace || std::fs::remove_dir(current).is_err() {
            break;
        }
        parent = current.parent();
    }
}
