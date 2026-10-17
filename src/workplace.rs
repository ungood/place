//! Finding the workplace, and the repos and unlisted checkouts in it.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{self, Config};
use crate::error::Error;
use crate::git;

/// A workplace directory and its merged configuration.
#[derive(Debug)]
pub struct Workplace {
    pub dir: PathBuf,
    /// Where this workplace sits relative to the one place was run in, such
    /// as `place/` for a nested workplace. Empty at the top.
    pub prefix: String,
    pub config: Config,
}

impl Workplace {
    /// Open the workplace in `dir`. place doesn't search parent directories,
    /// but when one of them is a workplace, the help names it.
    pub fn open(dir: &Path, envs: &[String]) -> Result<Self, Error> {
        if !config::is_workplace(dir) {
            return Err(Error::NotAWorkplace {
                help: not_a_workplace_help(dir),
            });
        }
        Self::load(dir.to_path_buf(), String::new(), envs)
    }

    fn load(dir: PathBuf, prefix: String, envs: &[String]) -> Result<Self, Error> {
        let config = config::load(&dir, envs)?;
        Ok(Self {
            dir,
            prefix,
            config,
        })
    }

    /// The nested workplace in the repo at `path`, if there is one.
    pub fn nested(&self, path: &str, envs: &[String]) -> Result<Option<Self>, Error> {
        let dir = self.dir.join(path);
        if !git::is_checkout(&dir) || !config::is_workplace(&dir) {
            return Ok(None);
        }
        Self::load(dir, self.display(path) + "/", envs).map(Some)
    }

    /// A repo's path as the user sees it, relative to the top workplace.
    pub fn display(&self, path: &str) -> String {
        format!("{}{path}", self.prefix)
    }

    /// Git checkouts directly inside the workplace that no file lists.
    pub fn unlisted(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut found: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| name != ".git")
            .filter(|name| !self.config.repos.contains_key(name))
            .filter(|name| git::is_checkout(&self.dir.join(name)))
            .collect();
        found.sort();
        found
    }
}

fn not_a_workplace_help(dir: &Path) -> String {
    let mut up = String::from("..");
    let mut ancestor = dir.parent();
    while let Some(parent) = ancestor {
        if config::is_workplace(parent) {
            return format!(
                "It's inside the workplace at \"{up}\". Run place there, or use \"place -C {up}\"."
            );
        }
        up.push_str("/..");
        ancestor = parent.parent();
    }
    "A workplace has a place.toml or a .repos file. Run \"place init\" to create one, or use -C <dir> to run place in another directory.".into()
}

/// Whether `path` is selected by the paths given on the command line. No
/// paths selects everything.
pub fn selected(filters: &[String], path: &str) -> bool {
    filters.is_empty()
        || filters.iter().any(|f| {
            let f = f.trim_end_matches('/');
            let f = f.strip_prefix("./").unwrap_or(f);
            path == f || path.starts_with(&format!("{f}/")) || f.starts_with(&format!("{path}/"))
        })
}

/// Fail if a path given on the command line selects no repo in the
/// workplace. Paths inside a listed repo may name a nested workplace's
/// repos, so they're accepted here.
pub fn check_selection(workplace: &Workplace, filters: &[String]) -> Result<(), Error> {
    let unlisted = workplace.unlisted();
    let known: Vec<&str> = workplace
        .config
        .repos
        .keys()
        .map(String::as_str)
        .chain(unlisted.iter().map(String::as_str))
        .collect();
    for filter in filters {
        if !known
            .iter()
            .any(|path| selected(std::slice::from_ref(filter), path))
        {
            return Err(Error::NotListed {
                path: clean_path(filter),
            });
        }
    }
    Ok(())
}

/// Clean up a path given on the command line: no `./` and no trailing slash.
pub fn clean_path(path: &str) -> String {
    let path = path.trim_end_matches('/');
    path.strip_prefix("./").unwrap_or(path).to_string()
}
