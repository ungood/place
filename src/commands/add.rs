use std::process::ExitCode;

use crate::error::Error;
use crate::workplace::{Workplace, clean_path};
use crate::{Context, config, git, output, url};

pub fn run(
    ctx: &Context,
    targets: &[String],
    branch: Option<&str>,
    env: Option<&str>,
) -> Result<ExitCode, Error> {
    let workplace = Workplace::open(&ctx.dir, &ctx.envs)?;
    let file = config::env_file(env);
    if url::is_url(&targets[0]) {
        clone(ctx, &workplace, &file, targets, branch)
    } else {
        adopt(ctx, &workplace, &file, targets, branch)
    }
}

fn check_unlisted(workplace: &Workplace, path: &str) -> Result<(), Error> {
    match workplace.config.repos.get(path) {
        Some(entry) => Err(Error::AlreadyListed {
            path: path.into(),
            file: entry.file().into(),
        }),
        None => Ok(()),
    }
}

fn clone(
    ctx: &Context,
    workplace: &Workplace,
    file: &str,
    targets: &[String],
    branch: Option<&str>,
) -> Result<ExitCode, Error> {
    let [repo_url, rest @ ..] = targets else {
        unreachable!("clap requires a target")
    };
    let path = match rest {
        [] => url::default_path(repo_url).ok_or_else(|| Error::NoDefaultPath {
            url: repo_url.clone(),
        })?,
        [path] => clean_path(path),
        _ => return Err(Error::AddArguments),
    };
    check_unlisted(workplace, &path)?;
    let dir = workplace.dir.join(&path);
    if dir.exists() {
        return Err(Error::DirectoryExists { path });
    }
    git::clone(repo_url, &dir, branch).map_err(|message| Error::CloneFailed {
        path: path.clone(),
        url: repo_url.clone(),
        message,
    })?;
    config::add_entry(&workplace.dir, file, &path, repo_url, branch)?;
    output::progress(ctx.quiet, &path, &format!("cloned and added to {file}"));
    Ok(ExitCode::SUCCESS)
}

fn adopt(
    ctx: &Context,
    workplace: &Workplace,
    file: &str,
    targets: &[String],
    branch: Option<&str>,
) -> Result<ExitCode, Error> {
    if branch.is_some() {
        return Err(Error::BranchOnAdopt);
    }
    let mut adopted = Vec::new();
    for target in targets {
        let path = clean_path(target);
        let dir = workplace.dir.join(&path);
        if !dir.exists() {
            return Err(Error::NoSuchPath {
                target: target.clone(),
            });
        }
        check_unlisted(workplace, &path)?;
        if !git::is_checkout(&dir) {
            return Err(Error::NothingToAdopt { path });
        }
        let remotes = git::remotes(&dir)?;
        let origin = remotes
            .iter()
            .find(|(name, _)| name == "origin")
            .or(if remotes.len() == 1 {
                remotes.first()
            } else {
                None
            });
        let repo_url = match origin {
            Some((_, u)) => u.clone(),
            None if remotes.is_empty() => return Err(Error::NoRemote { path }),
            None => return Err(Error::AmbiguousRemote { path }),
        };
        adopted.push((path, repo_url));
    }
    for (path, repo_url) in adopted {
        config::add_entry(&workplace.dir, file, &path, &repo_url, None)?;
        output::progress(ctx.quiet, &path, &format!("added to {file}"));
    }
    Ok(ExitCode::SUCCESS)
}
