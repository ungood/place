use std::process::ExitCode;

use crate::config;
use crate::error::Error;
use crate::{Context, output};

pub fn run(ctx: &Context) -> Result<ExitCode, Error> {
    if let Some(file) = config::workplace_files(&ctx.dir).into_iter().next() {
        return Err(if config::is_repos_file(&file) {
            Error::AlreadyAWorkplace {
                help: format!(
                    "place reads {file} as it is. \"place add\" creates place.toml when it needs to."
                ),
                file,
            }
        } else {
            Error::AlreadyAWorkplace {
                file: if file == "place.toml" {
                    "a place.toml".into()
                } else {
                    file
                },
                help: "Use \"place add\" to list more repos in it.".into(),
            }
        });
    }
    std::fs::write(ctx.dir.join("place.toml"), "").map_err(|source| Error::Write {
        file: "place.toml".into(),
        source,
    })?;
    output::progress(ctx.quiet, "place.toml", "created");
    Ok(ExitCode::SUCCESS)
}
