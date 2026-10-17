//! A repo's state, computed from its working copy and the last fetch.

use std::path::Path;

use serde::Serialize;

use crate::config::Entry;
use crate::error::Error;
use crate::git::{self, Refs, Target, WorkingCopy};
use crate::url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum State {
    #[serde(rename = "missing")]
    Missing,
    #[serde(rename = "not a repo")]
    NotARepo,
    #[serde(rename = "wrong remote")]
    WrongRemote,
    #[serde(rename = "on ref")]
    OnRef,
    #[serde(rename = "elsewhere")]
    Elsewhere,
    #[serde(rename = "dirty")]
    Dirty,
    #[serde(rename = "ahead")]
    Ahead,
    #[serde(rename = "behind")]
    Behind,
    #[serde(rename = "diverged")]
    Diverged,
    #[serde(rename = "default moved")]
    DefaultMoved,
    #[serde(rename = "unlisted")]
    Unlisted,
}

impl State {
    pub fn name(self) -> &'static str {
        match self {
            State::Missing => "missing",
            State::NotARepo => "not a repo",
            State::WrongRemote => "wrong remote",
            State::OnRef => "on ref",
            State::Elsewhere => "elsewhere",
            State::Dirty => "dirty",
            State::Ahead => "ahead",
            State::Behind => "behind",
            State::Diverged => "diverged",
            State::DefaultMoved => "default moved",
            State::Unlisted => "unlisted",
        }
    }
}

/// The ref's branch compared with its remote-tracking branch.
#[derive(Debug, Clone)]
pub struct Tracking {
    pub branch: String,
    pub remote: String,
    /// The local branch's commit, if the branch exists locally.
    pub local: Option<String>,
    /// The remote-tracking branch's commit, if it exists.
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

impl Tracking {
    pub fn upstream_name(&self) -> String {
        format!("{}/{}", self.remote, self.branch)
    }
}

/// Everything place knows about one listed repo.
#[derive(Debug, Clone)]
pub struct Inspection {
    pub states: Vec<State>,
    /// The URL of origin, or of the only remote, when no remote matches.
    pub actual_url: Option<String>,
    pub checked_out: Option<String>,
    pub tracking: Option<Tracking>,
}

impl Inspection {
    fn only(state: State) -> Self {
        Self {
            states: vec![state],
            actual_url: None,
            checked_out: None,
            tracking: None,
        }
    }

    pub fn has(&self, state: State) -> bool {
        self.states.contains(&state)
    }
}

/// Find the remote whose URL names the same repository as `wanted`, and
/// otherwise the URL of origin or of the only remote.
fn match_remote(dir: &Path, wanted: &str) -> Result<Result<String, Option<String>>, Error> {
    let remotes = git::remotes(dir)?;
    if let Some((name, _)) = remotes.iter().find(|(_, u)| url::same(u, wanted)) {
        return Ok(Ok(name.clone()));
    }
    let actual = remotes
        .iter()
        .find(|(name, _)| name == "origin")
        .or(if remotes.len() == 1 {
            remotes.first()
        } else {
            None
        })
        .map(|(_, u)| u.clone());
    Ok(Err(actual))
}

/// Inspect the repo for `entry` at `dir`, fetching first if `fetch` is set.
/// `nested` lists the checkouts of a nested workplace in the repo, which
/// don't make it dirty. A failed fetch is returned as git's message
/// alongside the inspection.
pub fn inspect(
    dir: &Path,
    entry: &Entry,
    nested: &[String],
    fetch: bool,
) -> Result<(Inspection, Option<String>), Error> {
    if !dir.exists() {
        return Ok((Inspection::only(State::Missing), None));
    }
    if !git::is_checkout(dir) {
        return Ok((Inspection::only(State::NotARepo), None));
    }
    let remote = match match_remote(dir, &entry.url)? {
        Ok(remote) => remote,
        Err(actual) => {
            let mut inspection = Inspection::only(State::WrongRemote);
            inspection.actual_url = actual;
            return Ok((inspection, None));
        }
    };
    let fetch_error = if fetch {
        git::fetch(dir, &remote, entry.reference.is_none()).err()
    } else {
        None
    };
    let copy = WorkingCopy::read(dir, nested)?;
    let refs = Refs::read(dir)?;
    let inspection = compute(dir, entry, remote, &copy, &refs)?;
    Ok((inspection, fetch_error))
}

fn compute(
    dir: &Path,
    entry: &Entry,
    remote: String,
    copy: &WorkingCopy,
    refs: &Refs,
) -> Result<Inspection, Error> {
    let mut states = Vec::new();
    let target = match &entry.reference {
        Some(reference) => refs
            .resolve(reference, &remote)
            .unwrap_or_else(|| Target::Branch(reference.clone())),
        None => {
            let cloned = refs.cloned_default(&remote);
            if let (Some(cloned), Some(fetched)) = (&cloned, refs.fetched_default(&remote))
                && *cloned != fetched
            {
                states.push(State::DefaultMoved);
            }
            match cloned.or(refs.fetched_default(&remote)) {
                Some(branch) => Target::Branch(branch),
                // Without a record of the remote's default branch, the
                // checked-out branch is the best guess.
                None => Target::Branch(copy.branch.clone().unwrap_or_default()),
            }
        }
    };

    let on_ref = match &target {
        Target::Branch(name) => copy.branch.as_deref() == Some(name.as_str()),
        Target::Tag(name) => {
            copy.branch.is_none()
                && refs
                    .get(&format!("refs/tags/{name}"))
                    .is_some_and(|tag| Some(tag.commit()) == copy.head.as_deref())
        }
        Target::Commit(hash) => {
            copy.branch.is_none() && copy.head.as_deref().is_some_and(|h| h.starts_with(hash))
        }
    };
    states.push(if on_ref {
        State::OnRef
    } else {
        State::Elsewhere
    });
    if copy.dirty() {
        states.push(State::Dirty);
    }

    let tracking = match &target {
        Target::Branch(name) if !name.is_empty() => {
            let local = refs
                .get(&format!("refs/heads/{name}"))
                .map(|r| r.oid.clone());
            let upstream = refs
                .get(&format!("refs/remotes/{remote}/{name}"))
                .map(|r| r.oid.clone());
            let (ahead, behind) = match (&local, &upstream) {
                (Some(l), Some(u)) if l != u => git::ahead_behind(dir, l, u)?,
                _ => (0, 0),
            };
            match (ahead, behind) {
                (0, 0) => {}
                (_, 0) => states.push(State::Ahead),
                (0, _) => states.push(State::Behind),
                _ => states.push(State::Diverged),
            }
            Some(Tracking {
                branch: name.clone(),
                remote,
                local,
                upstream,
                ahead,
                behind,
            })
        }
        _ => None,
    };

    // Keep the order of the design doc.
    states.sort_by_key(|s| *s as u8);
    Ok(Inspection {
        states,
        actual_url: None,
        checked_out: copy.branch.clone(),
        tracking,
    })
}
