use std::process::ExitCode;

use console::style;
use serde::Serialize;

use crate::config::Entry;
use crate::error::Error;
use crate::report;
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
    #[serde(skip)]
    pub entry: Option<Entry>,
    /// What `--fetch` printed.
    #[serde(skip)]
    pub fetched: String,
}

#[derive(Serialize)]
struct Json<'a> {
    repos: &'a [Row],
}

pub fn run(
    ctx: &Context,
    filters: &[String],
    fetch: bool,
    short: bool,
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
    } else if short {
        print_short(&rows);
    } else {
        print_long(&rows);
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
        state::inspect(
            &workplace.dir.join(path),
            entry,
            &workplace.nested_paths(path),
            fetch,
        )
    });
    for ((path, entry), result) in entries.into_iter().zip(results) {
        let (inspection, fetched) = result?;
        let display = workplace.display(path);
        let fetched = fetched.unwrap_or_else(|message| {
            output::error(Error::FetchFailed {
                path: display.clone(),
                message,
            });
            String::new()
        });
        rows.push(Row {
            path: display,
            url: Some(entry.url.clone()),
            reference: entry.reference.clone(),
            states: inspection.states.clone(),
            inspection: Some(inspection),
            entry: Some((*entry).clone()),
            fetched,
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
                entry: None,
                fetched: String::new(),
            });
        }
    }
    Ok(())
}

fn print_short(rows: &[Row]) {
    let width = rows.iter().map(|r| r.path.len()).max().unwrap_or(0);
    for row in rows {
        let summary = match (&row.inspection, &row.entry) {
            (Some(inspection), Some(entry)) => report::short(inspection, entry),
            _ => "not listed".into(),
        };
        println!(
            "{}  {}",
            style(format!("{:width$}", row.path)).bold(),
            report::tone(&row.states).apply_to(summary)
        );
    }
}

fn print_long(rows: &[Row]) {
    for row in rows {
        let mut lines: Vec<String> = Some(row.fetched.clone())
            .filter(|f| !f.is_empty())
            .into_iter()
            .collect();
        match (&row.inspection, &row.entry) {
            (Some(inspection), Some(entry)) => lines.extend(report::long(inspection, entry)),
            _ => lines.push("Not listed in any workplace file.".into()),
        }
        let heading = report::tone(&row.states).bold().apply_to(&row.path);
        println!("{}", output::indented(&heading.to_string(), &lines));
    }
}
