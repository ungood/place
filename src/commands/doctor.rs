use std::process::ExitCode;

use crate::error::{Error, Warning};
use crate::workplace::Workplace;
use crate::{Context, config, git, output};

/// The oldest git place is tested against: it needs `git switch` and
/// `status --porcelain=v2`.
const MINIMUM_GIT: (u32, u32) = (2, 25);

#[derive(Default)]
struct Findings {
    errors: Vec<Error>,
    warnings: Vec<Warning>,
}

pub fn run(ctx: &Context) -> Result<ExitCode, Error> {
    let workplace = Workplace::open(&ctx.dir, &ctx.envs)?;
    let mut findings = Findings::default();

    let version = git::version()?;
    if version < MINIMUM_GIT {
        findings.errors.push(Error::OldGit {
            version: format!("{}.{}", version.0, version.1),
            required: format!("{}.{}", MINIMUM_GIT.0, MINIMUM_GIT.1),
        });
    }
    check(ctx, &workplace, &mut findings)?;

    let Findings { errors, warnings } = findings;
    let summary = summary(errors.len(), warnings.len());
    let failed = !errors.is_empty();
    for warning in warnings {
        output::error(warning);
    }
    for error in errors {
        output::error(error);
    }
    match summary {
        None if !ctx.quiet => println!("No problems found."),
        Some(summary) => eprintln!("{summary}"),
        None => {}
    }
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

/// Check one workplace, then the nested workplaces in its repos.
fn check(ctx: &Context, workplace: &Workplace, findings: &mut Findings) -> Result<(), Error> {
    for (path, entry) in &workplace.config.repos {
        let winner = entry.file();
        if config::is_repos_file(winner) {
            continue;
        }
        if let Some(shadowed) = entry.files.iter().rfind(|f| config::is_repos_file(f)) {
            findings.warnings.push(Warning::ListedTwice {
                path: workplace.display(path),
                shadowed: shadowed.clone(),
                winner: winner.into(),
            });
        }
    }

    for path in workplace.unlisted() {
        findings.warnings.push(Warning::Unlisted {
            path: workplace.display(&path),
        });
    }

    if git::is_checkout(&workplace.dir) {
        let present: Vec<String> = workplace
            .config
            .repos
            .keys()
            .filter(|path| workplace.dir.join(path).exists())
            .map(|path| format!("{path}/"))
            .collect();
        let ignored = git::ignored(&workplace.dir, &present)?;
        let unignored: Vec<&str> = present
            .iter()
            .filter(|path| !ignored.contains(path))
            .map(|path| path.trim_end_matches('/'))
            .collect();
        if !unignored.is_empty() {
            findings.errors.push(not_ignored(workplace, &unignored));
        }
    }

    for path in workplace.config.repos.keys() {
        if let Some(nested) = workplace.nested(path, &ctx.envs)? {
            check(ctx, &nested, findings)?;
        }
    }
    Ok(())
}

fn not_ignored(workplace: &Workplace, paths: &[&str]) -> Error {
    let quoted =
        |f: &dyn Fn(&str) -> String| output::join(&paths.iter().map(|p| f(p)).collect::<Vec<_>>());
    let (repository, gitignore) = match workplace.prefix.strip_suffix('/') {
        None => (
            "The workplace repository".to_string(),
            "the workplace's .gitignore".to_string(),
        ),
        Some(repo) => (
            format!("The repository \"{repo}\""),
            format!("the .gitignore of \"{repo}\""),
        ),
    };
    Error::NotIgnored {
        repository,
        paths: quoted(&|p| format!("\"{p}\"")),
        them: if paths.len() == 1 { "it" } else { "them" },
        lines: quoted(&|p| format!("\"/{p}/\"")),
        gitignore,
    }
}

fn count(n: usize, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

/// "Found 1 error and 2 warnings.", or `None` when there's nothing.
fn summary(errors: usize, warnings: usize) -> Option<String> {
    let parts: Vec<String> = [(errors, "error"), (warnings, "warning")]
        .into_iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, noun)| count(n, noun))
        .collect();
    (!parts.is_empty()).then(|| format!("Found {}.", output::join(&parts)))
}
