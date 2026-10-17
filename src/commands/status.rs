use std::process::ExitCode;

use console::{Style, style};
use serde::Serialize;

use crate::error::Error;
use crate::state::{self, Inspection, State};
use crate::workplace::{Workplace, check_selection, selected};
use crate::{Context, output};

/// One line of `place status`.
#[derive(Debug, Serialize)]
pub struct Row {
    pub path: String,
    pub url: Option<String>,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
    pub states: Vec<State>,
    #[serde(skip)]
    pub inspection: Option<Inspection>,
}

#[derive(Serialize)]
struct Json<'a> {
    repos: &'a [Row],
}

pub fn run(
    ctx: &Context,
    filters: &[String],
    fetch: bool,
    json: bool,
    exit_code: bool,
) -> Result<ExitCode, Error> {
    let workplace = Workplace::open(&ctx.dir, &ctx.envs)?;
    check_selection(&workplace, filters)?;
    let mut rows = Vec::new();
    collect(ctx, &workplace, filters, fetch, &mut rows)?;

    if json {
        let text = serde_json::to_string_pretty(&Json { repos: &rows }).expect("status is JSON");
        println!("{text}");
    } else {
        print(&rows);
    }

    let unclean = rows
        .iter()
        .any(|row| row.url.is_some() && row.states != [State::OnRef]);
    Ok(if exit_code && unclean {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

/// Inspect every selected repo in `workplace` and its nested workplaces.
fn collect(
    ctx: &Context,
    workplace: &Workplace,
    filters: &[String],
    fetch: bool,
    rows: &mut Vec<Row>,
) -> Result<(), Error> {
    let entries: Vec<_> = workplace
        .config
        .repos
        .iter()
        .filter(|(path, _)| selected(filters, &workplace.display(path)))
        .collect();
    let results = output::parallel(&entries, ctx.jobs, |(path, entry)| {
        state::inspect(&workplace.dir.join(path), entry, fetch)
    });
    for ((path, entry), result) in entries.into_iter().zip(results) {
        let (inspection, fetch_error) = result?;
        let display = workplace.display(path);
        if let Some(message) = fetch_error {
            output::error(Error::FetchFailed {
                path: display.clone(),
                message,
            });
        }
        rows.push(Row {
            path: display,
            url: Some(entry.url.clone()),
            reference: entry.reference.clone(),
            states: inspection.states.clone(),
            inspection: Some(inspection),
        });
        if let Some(nested) = workplace.nested(path, &ctx.envs)? {
            collect(ctx, &nested, filters, fetch, rows)?;
        }
    }
    for path in workplace.unlisted() {
        let display = workplace.display(&path);
        if selected(filters, &display) {
            rows.push(Row {
                path: display,
                url: None,
                reference: None,
                states: vec![State::Unlisted],
                inspection: None,
            });
        }
    }
    Ok(())
}

fn state_style(state: State) -> Style {
    match state {
        State::OnRef => Style::new().green(),
        State::Missing | State::NotARepo | State::WrongRemote | State::Diverged => {
            Style::new().red()
        }
        State::Unlisted => Style::new().dim(),
        _ => Style::new().yellow(),
    }
}

fn describe(row: &Row, state: State) -> String {
    let tracking = row.inspection.as_ref().and_then(|i| i.tracking.as_ref());
    match (state, tracking) {
        (State::Ahead, Some(t)) => format!("ahead {}", t.ahead),
        (State::Behind, Some(t)) => format!("behind {}", t.behind),
        (State::Diverged, Some(t)) => format!("diverged +{} -{}", t.ahead, t.behind),
        (State::Elsewhere, _) => match row.inspection.as_ref().and_then(|i| i.checked_out.as_ref())
        {
            Some(branch) => format!("elsewhere on {branch}"),
            None => "elsewhere, detached".into(),
        },
        _ => state.name().into(),
    }
}

fn print(rows: &[Row]) {
    let width = rows.iter().map(|r| r.path.len()).max().unwrap_or(0);
    for row in rows {
        let states: Vec<String> = row
            .states
            .iter()
            .map(|s| state_style(*s).apply_to(describe(row, *s)).to_string())
            .collect();
        println!(
            "{}  {}",
            style(format!("{:width$}", row.path)).bold(),
            states.join(", ")
        );
    }
}
