use std::path::Path;
use std::process::ExitCode;

use console::style;

use crate::config::Entry;
use crate::error::Error;
use crate::report;
use crate::state::{self, State};
use crate::workplace::{Workplace, check_selection, selected};
use crate::{Context, output};
use crate::{config, git};

pub fn run(ctx: &Context, filters: &[String], short: bool) -> Result<ExitCode, Error> {
    let workplace = Workplace::open(&ctx.dir, &ctx.envs)?;
    check_selection(&workplace, filters)?;
    let mut tally = Tally {
        short,
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
    /// One line per repo, rather than git's output and a description.
    short: bool,
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
        let display = workplace.display(path);
        tally.repos += 1;
        let (synced, err) = match result {
            Ok(synced) => (synced, None),
            Err(err) => {
                let failed = style("failed, see below").red().to_string();
                let synced = Synced {
                    short: failed.clone(),
                    long: vec![failed],
                };
                (synced, Some(err))
            }
        };
        if tally.short {
            let display = format!("{display:0$}", tally.width);
            output::progress(ctx.quiet, &display, &synced.short);
        } else {
            output::block(ctx.quiet, &display, &synced.long);
        }
        tally.errors.extend(err);
        if let Some(nested) = workplace.nested(path, &ctx.envs)? {
            sync(ctx, &nested, filters, tally)?;
        }
    }
    Ok(())
}

/// What syncing one repo did, said both ways.
struct Synced {
    /// One line, such as `main 8006fac..d490938 (fast-forward)`.
    short: String,
    /// What git printed, then how the repo stands if that needs saying.
    long: Vec<String>,
}

/// Bring one repo in line with its entry, and say what happened.
fn sync_repo(workplace: &Workplace, path: &str, entry: &Entry) -> Result<Synced, Error> {
    let dir = workplace.dir.join(path);
    let display = workplace.display(path);
    let nested = workplace.nested_paths(path);
    let (mut inspection, fetched) = state::inspect(&dir, entry, &nested, true)?;

    // What git printed, and a short note of each change place made.
    let mut printed = Vec::new();
    let mut changes = Vec::new();
    let mut held_back = None;
    if inspection.has(State::Missing) {
        let output =
            git::clone(&entry.url, &dir, entry.reference.as_deref()).map_err(|message| {
                Error::CloneFailed {
                    path: display,
                    url: entry.url.clone(),
                    message,
                }
            })?;
        printed.push(output);
        changes.push("cloned".to_string());
    } else {
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
        printed.push(fetched.map_err(|message| Error::FetchFailed {
            path: display.clone(),
            message,
        })?);
        if let Some(t) = &inspection.tracking
            && let (Some(local), Some(upstream)) = (&t.local, &t.upstream)
        {
            if t.ahead > 0 && t.behind > 0 {
                return Err(Error::Diverged {
                    path: display,
                    branch: t.branch.clone(),
                    upstream: t.upstream_name(),
                });
            }
            if t.ahead == 0 && t.behind > 0 {
                let on_branch = inspection.checked_out.as_deref() == Some(t.branch.as_str());
                let output = if !on_branch {
                    Some(git::fast_forward_branch(&dir, &t.remote, &t.branch)?)
                } else if inspection.has(State::Dirty) {
                    held_back = Some(
                        "place didn't fast-forward it, to leave your changes alone.".to_string(),
                    );
                    None
                } else {
                    Some(git::fast_forward_head(&dir, upstream)?)
                };
                if let Some(output) = output {
                    printed.push(output);
                    changes.push(format!(
                        "{} {}..{} (fast-forward)",
                        t.branch,
                        report::abbrev(local),
                        report::abbrev(upstream)
                    ));
                }
            }
        }
    }
    if !changes.is_empty() {
        inspection = state::inspect(&dir, entry, &nested, false)?.0;
    }

    // After a change that left the repo clean and on its ref, git's output
    // says it all. Otherwise, say how the repo stands.
    let settled = !changes.is_empty() && inspection.states == [State::OnRef];
    let mut short = Vec::new();
    let mut long: Vec<String> = printed.into_iter().filter(|p| !p.is_empty()).collect();
    if !settled {
        short.push(report::short(&inspection, entry));
        long.extend(report::long(&inspection, entry));
        long.extend(held_back);
    }
    short.extend(changes);
    Ok(Synced {
        short: short.join("; "),
        long,
    })
}
