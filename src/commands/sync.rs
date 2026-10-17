use std::process::ExitCode;

use crate::config::Entry;
use crate::error::Error;
use crate::git;
use crate::state::{self, State};
use crate::workplace::{Workplace, check_selection, selected};
use crate::{Context, output};

pub fn run(ctx: &Context, filters: &[String]) -> Result<ExitCode, Error> {
    let workplace = Workplace::open(&ctx.dir, &ctx.envs)?;
    check_selection(&workplace, filters)?;
    let mut failed = false;
    sync(ctx, &workplace, filters, &mut failed)?;
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

/// Sync every selected repo in `workplace`, then its nested workplaces.
fn sync(
    ctx: &Context,
    workplace: &Workplace,
    filters: &[String],
    failed: &mut bool,
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
        match result {
            Ok(message) => output::progress(ctx.quiet, &display, &message),
            Err(err) => {
                *failed = true;
                output::error(err);
            }
        }
        if let Some(nested) = workplace.nested(path, &ctx.envs)? {
            sync(ctx, &nested, filters, failed)?;
        }
    }
    Ok(())
}

/// Bring one repo in line with its entry, and say what happened.
fn sync_repo(workplace: &Workplace, path: &str, entry: &Entry) -> Result<String, Error> {
    let dir = workplace.dir.join(path);
    let display = workplace.display(path);
    let (inspection, fetch_error) = state::inspect(&dir, entry, true)?;

    if inspection.has(State::Missing) {
        git::clone(&entry.url, &dir, entry.reference.as_deref()).map_err(|message| {
            Error::CloneFailed {
                path: display,
                url: entry.url.clone(),
                message,
            }
        })?;
        return Ok("cloned".into());
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

    let mut notes = Vec::new();
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
            if !on_branch {
                git::fast_forward_branch(&dir, &t.branch, local, upstream)?;
                notes.push(format!("fast-forwarded {}", t.branch));
            } else if inspection.has(State::Dirty) {
                notes.push(format!("dirty, so {} wasn't fast-forwarded", t.branch));
            } else {
                git::fast_forward_head(&dir, upstream)?;
                notes.push(format!("fast-forwarded {}", t.branch));
            }
        }
    }
    for state in &inspection.states {
        match state {
            State::Elsewhere => notes.push("elsewhere, left alone".into()),
            State::Ahead => notes.push("ahead, left alone".into()),
            State::DefaultMoved => notes.push("the remote's default branch moved".into()),
            _ => {}
        }
    }
    Ok(if notes.is_empty() {
        "up to date".into()
    } else {
        notes.join("; ")
    })
}
