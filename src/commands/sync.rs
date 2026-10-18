use std::path::Path;
use std::process::ExitCode;

use console::style;

use crate::config::Entry;
use crate::error::Error;
use crate::state::{self, Inspection, State, Tracking};
use crate::workplace::{Workplace, check_selection, selected};
use crate::{Context, output};
use crate::{config, git};

pub fn run(ctx: &Context, filters: &[String]) -> Result<ExitCode, Error> {
    let workplace = Workplace::open(&ctx.dir, &ctx.envs)?;
    check_selection(&workplace, filters)?;
    let mut tally = Tally {
        width: width(
            &workplace.dir,
            "",
            &workplace.config.repos.keys().cloned().collect::<Vec<_>>(),
        ),
        ..Tally::default()
    };
    sync(ctx, &workplace, filters, &mut tally)?;

    // Errors come after the progress lines, so they aren't lost among them.
    let failed = tally.errors.len();
    for err in tally.errors {
        output::error(err);
    }
    if failed > 0 {
        let repos = if tally.repos == 1 { "repo" } else { "repos" };
        eprintln!("{failed} of {} {repos} failed to sync.", tally.repos);
        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

#[derive(Default)]
struct Tally {
    repos: usize,
    errors: Vec<Error>,
    /// The width of the longest path, to line up the progress lines.
    width: usize,
}

/// The length of the longest path among `paths` in `dir` and the nested
/// workplaces already checked out there.
fn width(dir: &Path, prefix: &str, paths: &[String]) -> usize {
    paths
        .iter()
        .map(|path| {
            let nested = dir.join(path);
            let own = prefix.len() + path.len();
            if config::is_workplace(&nested) {
                let prefix = format!("{prefix}{path}/");
                own.max(width(&nested, &prefix, &config::listed_paths(&nested)))
            } else {
                own
            }
        })
        .max()
        .unwrap_or(0)
}

/// Sync every selected repo in `workplace`, then its nested workplaces.
fn sync(
    ctx: &Context,
    workplace: &Workplace,
    filters: &[String],
    tally: &mut Tally,
) -> Result<(), Error> {
    let entries: Vec<_> = workplace
        .config
        .repos
        .iter()
        .filter(|(path, _)| selected(filters, &workplace.display(path)))
        .collect();
    let results = output::parallel(&entries, ctx.jobs, |(path, entry)| {
        sync_repo(workplace, path, entry)
    });
    for ((path, _), result) in entries.into_iter().zip(results) {
        let display = format!("{:1$}", workplace.display(path), tally.width);
        tally.repos += 1;
        match result {
            Ok(message) => output::progress(ctx.quiet, &display, &message),
            Err(err) => {
                let message = style("failed, see below").red().to_string();
                output::progress(ctx.quiet, &display, &message);
                tally.errors.push(err);
            }
        }
        if let Some(nested) = workplace.nested(path, &ctx.envs)? {
            sync(ctx, &nested, filters, tally)?;
        }
    }
    Ok(())
}

/// Bring one repo in line with its entry, and say what happened.
fn sync_repo(workplace: &Workplace, path: &str, entry: &Entry) -> Result<String, Error> {
    let dir = workplace.dir.join(path);
    let display = workplace.display(path);
    let (inspection, fetch_error) =
        state::inspect(&dir, entry, &workplace.nested_paths(path), true)?;

    if inspection.has(State::Missing) {
        git::clone(&entry.url, &dir, entry.reference.as_deref()).map_err(|message| {
            Error::CloneFailed {
                path: display,
                url: entry.url.clone(),
                message,
            }
        })?;
        return Ok(match &entry.reference {
            Some(reference) => format!("wasn't here; cloned {reference} from {}", entry.url),
            None => format!("wasn't here; cloned it from {}", entry.url),
        });
    }
    if inspection.has(State::NotARepo) {
        return Err(Error::NotARepo { path: display });
    }
    if inspection.has(State::WrongRemote) {
        return Err(Error::WrongRemote {
            path: display,
            actual: inspection
                .actual_url
                .unwrap_or_else(|| "a different repository".into()),
            expected: entry.url.clone(),
            file: entry.file().into(),
        });
    }
    if let Some(message) = fetch_error {
        return Err(Error::FetchFailed {
            path: display,
            message,
        });
    }

    // Each note says what place found and what it did about it.
    let mut notes = Vec::new();
    let elsewhere = inspection.has(State::Elsewhere);
    let wanted = match &inspection.tracking {
        Some(t) => Some(t.branch.as_str()),
        None => entry.reference.as_deref(),
    };
    if elsewhere {
        notes.push(match (&inspection.checked_out, wanted) {
            (Some(branch), Some(wanted)) => {
                format!("on branch {branch}, not {wanted}; left it alone")
            }
            (Some(branch), None) => format!("on branch {branch}; left it alone"),
            (None, Some(wanted)) => format!("detached HEAD, not {wanted}; left it alone"),
            (None, None) => "detached HEAD; left it alone".into(),
        });
    }
    match &inspection.tracking {
        Some(t) => {
            if let Some(note) = sync_branch(&dir, &display, &inspection, t)? {
                notes.push(note);
            }
        }
        None if !elsewhere => {
            let reference = entry.reference.as_deref().unwrap_or("its ref");
            notes.push(format!(
                "at {reference}, as {} pins it; nothing to update",
                entry.file()
            ));
        }
        None => {}
    }
    if let (Some(t), Some(new_default)) = (&inspection.tracking, &inspection.new_default) {
        notes.push(format!(
            "{}'s default branch moved from {} to {new_default}; kept following {} (set a ref in {} to switch)",
            t.remote,
            t.branch,
            t.branch,
            entry.file()
        ));
    }
    Ok(if notes.is_empty() {
        "nothing to update".into()
    } else {
        notes.join("; ")
    })
}

/// Fast-forward the ref's branch if it's safely behind its remote, and say
/// how it compares. Returns `None` when there's nothing worth saying.
fn sync_branch(
    dir: &Path,
    display: &str,
    inspection: &Inspection,
    t: &Tracking,
) -> Result<Option<String>, Error> {
    let upstream_name = t.upstream_name();
    let (Some(local), Some(upstream)) = (&t.local, &t.upstream) else {
        return Ok(match (&t.local, &t.upstream) {
            (_, None) => Some(format!(
                "{} has no branch {}; nothing to fast-forward to",
                t.remote, t.branch
            )),
            // The branch was never checked out here, so there's nothing to update.
            _ => None,
        });
    };
    if t.ahead > 0 && t.behind > 0 {
        return Err(Error::Diverged {
            path: display.into(),
            branch: t.branch.clone(),
            upstream: upstream_name,
        });
    }
    let commits = |n: u32| {
        if n == 1 {
            "1 commit".into()
        } else {
            format!("{n} commits")
        }
    };
    let on_branch = inspection.checked_out.as_deref() == Some(t.branch.as_str());
    Ok(Some(if t.behind > 0 {
        if !on_branch {
            git::fast_forward_branch(dir, &t.branch, local, upstream)?;
        } else if inspection.has(State::Dirty) {
            return Ok(Some(format!(
                "{} is {} behind {upstream_name} but has uncommitted changes; didn't fast-forward it",
                t.branch,
                commits(t.behind)
            )));
        } else {
            git::fast_forward_head(dir, upstream)?;
        }
        format!(
            "fast-forwarded {} by {} to match {upstream_name}",
            t.branch,
            commits(t.behind)
        )
    } else if t.ahead > 0 {
        format!(
            "{} is {} ahead of {upstream_name}; left it for you to push",
            t.branch,
            commits(t.ahead)
        )
    } else if on_branch {
        format!("on {}, up to date with {upstream_name}", t.branch)
    } else {
        return Ok(None);
    }))
}
