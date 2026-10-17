use std::process::ExitCode;

use crate::error::{Error, Warning};
use crate::workplace::Workplace;
use crate::{Context, config, git, output};

/// The oldest git place is tested against: it needs `git switch` and
/// `status --porcelain=v2`.
const MINIMUM_GIT: (u32, u32) = (2, 25);

pub fn run(ctx: &Context) -> Result<ExitCode, Error> {
    let workplace = Workplace::open(&ctx.dir, &ctx.envs)?;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let version = git::version()?;
    if version < MINIMUM_GIT {
        errors.push(Error::OldGit {
            version: format!("{}.{}", version.0, version.1),
            required: format!("{}.{}", MINIMUM_GIT.0, MINIMUM_GIT.1),
        });
    }

    for (path, entry) in &workplace.config.repos {
        let winner = entry.file();
        if config::is_repos_file(winner) {
            continue;
        }
        if let Some(shadowed) = entry.files.iter().rfind(|f| config::is_repos_file(f)) {
            warnings.push(Warning::ListedTwice {
                path: path.clone(),
                shadowed: shadowed.clone(),
                winner: winner.into(),
            });
        }
    }

    for path in workplace.unlisted() {
        warnings.push(Warning::Unlisted { path });
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
        for path in present {
            if !ignored.contains(&path) {
                errors.push(Error::NotIgnored {
                    path: path.trim_end_matches('/').into(),
                });
            }
        }
    }

    let clean = errors.is_empty() && warnings.is_empty();
    let failed = !errors.is_empty();
    for warning in warnings {
        output::error(warning);
    }
    for error in errors {
        output::error(error);
    }
    if clean && !ctx.quiet {
        println!("No problems found.");
    }
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
