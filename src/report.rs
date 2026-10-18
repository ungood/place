//! How a repo's state reads to a person. Where git has words for a state,
//! these use them: the long form follows `git status`, and the short form
//! follows `git status --short --branch`.

use console::Style;

use crate::config::Entry;
use crate::state::{Inspection, State, Tracking};

/// The color for a repo's states: red when it needs attention, green when
/// it's clean and on its ref, yellow for work in progress.
pub fn tone(states: &[State]) -> Style {
    let attention = [
        State::Missing,
        State::NotARepo,
        State::WrongRemote,
        State::Diverged,
    ];
    if states.iter().any(|s| attention.contains(s)) {
        Style::new().red()
    } else if states == [State::Unlisted] {
        Style::new().dim()
    } else if states == [State::OnRef] {
        Style::new().green()
    } else {
        Style::new().yellow()
    }
}

/// A commit hash shortened the way git shortens it.
pub fn abbrev(oid: &str) -> &str {
    &oid[..oid.len().min(7)]
}

fn commits(n: u32) -> String {
    if n == 1 {
        "1 commit".into()
    } else {
        format!("{n} commits")
    }
}

/// What the repo should have checked out: its ref's branch, or its ref.
fn wanted<'a>(inspection: &'a Inspection, entry: &'a Entry) -> Option<&'a str> {
    match &inspection.tracking {
        Some(t) => Some(t.branch.as_str()),
        None => entry.reference.as_deref(),
    }
}

fn head(inspection: &Inspection) -> &str {
    inspection
        .head
        .as_deref()
        .map_or("an unborn commit", abbrev)
}

/// The ref's branch against its remote-tracking branch, as
/// `git status --short --branch` puts it.
fn counts(t: &Tracking) -> Option<String> {
    match (t.ahead, t.behind) {
        (0, 0) => None,
        (a, 0) => Some(format!("[ahead {a}]")),
        (0, b) => Some(format!("[behind {b}]")),
        (a, b) => Some(format!("[ahead {a}, behind {b}]")),
    }
}

/// One line, such as `main [behind 2]; uncommitted changes`.
pub fn short(inspection: &Inspection, entry: &Entry) -> String {
    if inspection.has(State::Missing) {
        return "not cloned".into();
    }
    if inspection.has(State::NotARepo) {
        return "not a git repo".into();
    }
    if inspection.has(State::WrongRemote) {
        let actual = inspection.actual_url.as_deref().unwrap_or("another repo");
        return format!("origin is {actual}, not {}", entry.url);
    }
    let elsewhere = inspection.has(State::Elsewhere);
    let wanted = wanted(inspection, entry);
    let mut parts = Vec::new();
    if elsewhere {
        let at = match &inspection.checked_out {
            Some(branch) => format!("on {branch}"),
            None => format!("detached at {}", head(inspection)),
        };
        parts.push(match wanted {
            Some(wanted) => format!("{at}, not {wanted}"),
            None => at,
        });
    }
    match &inspection.tracking {
        Some(t) if t.upstream.is_none() => {
            parts.push(format!("{}, not on {}", t.branch, t.remote));
        }
        // The ref's branch was never created here, so there's nothing to compare.
        Some(t) if t.local.is_none() => {}
        Some(t) => match counts(t) {
            Some(counts) => parts.push(format!("{} {counts}", t.branch)),
            None if !elsewhere => parts.push(format!("{}, up to date", t.branch)),
            None => {}
        },
        None if !elsewhere => parts.push(format!("at {}", wanted.unwrap_or("its ref"))),
        None => {}
    }
    if inspection.has(State::Dirty) {
        parts.push("uncommitted changes".into());
    }
    if let (Some(t), Some(new_default)) = (&inspection.tracking, &inspection.new_default) {
        parts.push(format!("{}'s default is now {new_default}", t.remote));
    }
    parts.join("; ")
}

/// A few sentences, in the words `git status` uses where it has them.
pub fn long(inspection: &Inspection, entry: &Entry) -> Vec<String> {
    if inspection.has(State::Missing) {
        return vec!["Not cloned yet.".into()];
    }
    if inspection.has(State::NotARepo) {
        return vec!["Not a git repository.".into()];
    }
    if inspection.has(State::WrongRemote) {
        let actual = inspection.actual_url.as_deref().unwrap_or("another repo");
        return vec![format!(
            "origin is {actual}, but {} expects {}.",
            entry.file(),
            entry.url
        )];
    }
    let elsewhere = inspection.has(State::Elsewhere);
    let wanted = wanted(inspection, entry);
    let mut lines = Vec::new();
    match (&inspection.checked_out, elsewhere) {
        (Some(branch), false) => lines.push(format!("On branch {branch}")),
        (Some(branch), true) => lines.push(match wanted {
            Some(wanted) => format!("On branch {branch}, not {wanted}"),
            None => format!("On branch {branch}"),
        }),
        (None, false) => lines.push(format!("HEAD detached at {}", wanted.unwrap_or("its ref"))),
        (None, true) => lines.push(match wanted {
            Some(wanted) => format!("HEAD detached at {}, not {wanted}", head(inspection)),
            None => format!("HEAD detached at {}", head(inspection)),
        }),
    }
    if let Some(t) = &inspection.tracking {
        let subject = if elsewhere {
            format!("Branch '{}'", t.branch)
        } else {
            "Your branch".into()
        };
        let upstream = t.upstream_name();
        if t.upstream.is_none() {
            lines.push(format!("{} has no branch '{}'.", t.remote, t.branch));
        } else if t.local.is_some() {
            match (t.ahead, t.behind) {
                (0, 0) if !elsewhere => {
                    lines.push(format!("Your branch is up to date with '{upstream}'."));
                }
                (0, 0) => {}
                (a, 0) => lines.push(format!(
                    "{subject} is ahead of '{upstream}' by {}.",
                    commits(a)
                )),
                (0, b) => lines.push(format!(
                    "{subject} is behind '{upstream}' by {}.",
                    commits(b)
                )),
                (a, b) => lines.push(format!(
                    "{subject} and '{upstream}' have diverged, with {a} and {b} different commits each."
                )),
            }
        }
    }
    if inspection.has(State::Dirty) {
        lines.push("You have uncommitted changes.".into());
    }
    if let (Some(t), Some(new_default)) = (&inspection.tracking, &inspection.new_default) {
        lines.push(format!(
            "{}'s default branch is now {new_default}, but this repo still follows {}.",
            t.remote, t.branch
        ));
    }
    lines
}
