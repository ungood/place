//! Workplace files: `*.repos`, `place.toml`, `place.<env>.toml`,
//! `place.local.toml` and `place.<env>.local.toml`, merged per repo path.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use miette::{NamedSource, SourceSpan};
use serde::Deserialize;
use toml_edit::{DocumentMut, Item, Table, value};

use crate::error::{Error, SyntaxError};

/// One repo, as the highest-precedence file defines it.
#[derive(Debug, Clone)]
pub struct Entry {
    pub url: String,
    pub reference: Option<String>,
    /// The files that define this path, from lowest to highest precedence.
    /// The last one supplies the entry.
    pub files: Vec<String>,
}

impl Entry {
    /// The file this entry comes from.
    pub fn file(&self) -> &str {
        self.files.last().expect("an entry comes from a file")
    }
}

/// The merged configuration of one workplace directory.
#[derive(Debug, Default)]
pub struct Config {
    pub repos: BTreeMap<String, Entry>,
}

pub fn is_repos_file(name: &str) -> bool {
    name.ends_with(".repos") && name.len() > ".repos".len()
}

fn is_place_file(name: &str) -> bool {
    name.starts_with("place.") && name.ends_with(".toml")
}

/// The file name `place add -e <env>` writes to.
pub fn env_file(env: Option<&str>) -> String {
    match env {
        None => "place.toml".into(),
        Some(env) => format!("place.{env}.toml"),
    }
}

fn file_names(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    names
}

/// The workplace files in `dir`, whatever the environment: `place.toml`
/// first, then any other `place.*.toml`, then `*.repos` files.
pub fn workplace_files(dir: &Path) -> Vec<String> {
    let names = file_names(dir);
    let mut files: Vec<String> = names
        .iter()
        .filter(|n| *n == "place.toml")
        .cloned()
        .collect();
    files.extend(
        names
            .iter()
            .filter(|n| is_place_file(n) && *n != "place.toml")
            .cloned(),
    );
    files.extend(names.iter().filter(|n| is_repos_file(n)).cloned());
    files
}

/// Whether `dir` holds place configuration.
pub fn is_workplace(dir: &Path) -> bool {
    !workplace_files(dir).is_empty()
}

/// The files to read for the selected environments, from lowest to highest
/// precedence, that exist in `dir`.
fn files_to_read(dir: &Path, envs: &[String]) -> Vec<String> {
    let names = file_names(dir);
    let mut order: Vec<String> = names.iter().filter(|n| is_repos_file(n)).cloned().collect();
    order.push("place.toml".into());
    order.extend(envs.iter().map(|env| format!("place.{env}.toml")));
    order.push("place.local.toml".into());
    order.extend(envs.iter().map(|env| format!("place.{env}.local.toml")));
    let mut seen = Vec::new();
    order
        .into_iter()
        .filter(|name| names.contains(name))
        .filter(|name| {
            let new = !seen.contains(name);
            seen.push(name.clone());
            new
        })
        .collect()
}

/// Read and merge the configuration in `dir`.
pub fn load(dir: &Path, envs: &[String]) -> Result<Config, Error> {
    let mut config = Config::default();
    for file in files_to_read(dir, envs) {
        let text = fs::read_to_string(dir.join(&file)).map_err(|source| Error::Read {
            file: file.clone(),
            source,
        })?;
        let entries = if is_repos_file(&file) {
            parse_repos(&file, &text)?
        } else {
            parse_toml(&file, &text)?
        };
        for (path, url, reference) in entries {
            let mut files = config
                .repos
                .remove(&path)
                .map(|e| e.files)
                .unwrap_or_default();
            files.push(file.clone());
            config.repos.insert(
                path,
                Entry {
                    url,
                    reference,
                    files,
                },
            );
        }
    }
    Ok(config)
}

type Parsed = Vec<(String, String, Option<String>)>;

fn syntax_error(file: &str, text: &str, offset: usize, detail: String) -> Error {
    let offset = offset.min(text.len());
    let line = text[..offset].matches('\n').count() + 1;
    Error::Syntax(Box::new(SyntaxError {
        file: file.to_string(),
        line,
        src: NamedSource::new(file, text.to_string()),
        span: SourceSpan::from(offset..offset),
        detail,
    }))
}

fn parse_document(file: &str, text: &str) -> Result<DocumentMut, Error> {
    text.parse::<DocumentMut>().map_err(|e| {
        let offset = e.span().map_or(0, |s| s.start);
        syntax_error(file, text, offset, e.message().to_string())
    })
}

fn parse_toml(file: &str, text: &str) -> Result<Parsed, Error> {
    let doc = parse_document(file, text)?;
    let mut entries = Vec::new();
    for (key, item) in doc.iter() {
        if key != "repos" {
            return Err(Error::UnknownSetting {
                file: file.into(),
                key: key.into(),
            });
        }
        let Some(repos) = item.as_table_like() else {
            return Err(Error::UnknownSetting {
                file: file.into(),
                key: key.into(),
            });
        };
        for (path, item) in repos.iter() {
            let bad = |problem: &str| Error::BadEntry {
                path: path.into(),
                file: file.into(),
                problem: problem.into(),
            };
            let table = item.as_table_like().ok_or_else(|| bad("isn't a table"))?;
            let mut url = None;
            let mut reference = None;
            for (key, item) in table.iter() {
                let text = item
                    .as_str()
                    .ok_or_else(|| bad(&format!("has a {key} that isn't a string")))?;
                match key {
                    "url" => url = Some(text.to_string()),
                    "ref" => reference = Some(text.to_string()),
                    _ => return Err(bad(&format!("has an unknown key \"{key}\""))),
                }
            }
            let url = url
                .filter(|u| !u.trim().is_empty())
                .ok_or_else(|| Error::NoUrl {
                    path: path.into(),
                    file: file.into(),
                })?;
            entries.push((path.to_string(), url, reference));
        }
    }
    Ok(entries)
}

#[derive(Deserialize)]
struct ReposFile {
    #[serde(default)]
    repositories: Option<BTreeMap<String, ReposEntry>>,
}

#[derive(Deserialize)]
struct ReposEntry {
    #[serde(rename = "type")]
    kind: Option<String>,
    url: Option<String>,
    version: Option<String>,
}

fn article(word: &str) -> &'static str {
    // Letters whose names start with a vowel sound: "an svn", "an hg".
    match word.chars().next().map(|c| c.to_ascii_lowercase()) {
        Some('a' | 'e' | 'f' | 'h' | 'i' | 'l' | 'm' | 'n' | 'o' | 'r' | 's' | 'x') => "an",
        _ => "a",
    }
}

fn parse_repos(file: &str, text: &str) -> Result<Parsed, Error> {
    let parsed: ReposFile = serde_yaml_ng::from_str(text).map_err(|e| {
        let offset = e.location().map_or(0, |l| l.index());
        syntax_error(file, text, offset, e.to_string())
    })?;
    let mut entries = Vec::new();
    for (path, entry) in parsed.repositories.unwrap_or_default() {
        let kind = entry.kind.unwrap_or_else(|| "git".into());
        if kind != "git" {
            return Err(Error::NotGit {
                file: file.into(),
                path,
                article: article(&kind),
                kind,
            });
        }
        let url = entry
            .url
            .filter(|u| !u.trim().is_empty())
            .ok_or_else(|| Error::NoUrl {
                path: path.clone(),
                file: file.into(),
            })?;
        entries.push((path, url, entry.version));
    }
    Ok(entries)
}

/// Add an entry to a place file in `dir`, creating the file if needed and
/// keeping its comments and formatting.
pub fn add_entry(
    dir: &Path,
    file: &str,
    path: &str,
    url: &str,
    reference: Option<&str>,
) -> Result<(), Error> {
    let full = dir.join(file);
    let text = if full.exists() {
        fs::read_to_string(&full).map_err(|source| Error::Read {
            file: file.into(),
            source,
        })?
    } else {
        String::new()
    };
    let mut doc = parse_document(file, &text)?;
    let first_table = doc.iter().next().is_none();
    let repos = doc
        .entry("repos")
        .or_insert_with(|| {
            let mut table = Table::new();
            table.set_implicit(true);
            Item::Table(table)
        })
        .as_table_mut()
        .ok_or_else(|| Error::UnknownSetting {
            file: file.into(),
            key: "repos".into(),
        })?;
    let mut table = Table::new();
    table.insert("url", value(url));
    if let Some(reference) = reference {
        table.insert("ref", value(reference));
    }
    if !first_table {
        table.decor_mut().set_prefix("\n");
    }
    repos.insert(path, Item::Table(table));
    write(dir, file, &doc.to_string())
}

/// Remove an entry from a place file in `dir`, keeping everything else.
pub fn remove_entry(dir: &Path, file: &str, path: &str) -> Result<(), Error> {
    let text = fs::read_to_string(dir.join(file)).map_err(|source| Error::Read {
        file: file.into(),
        source,
    })?;
    let mut doc = parse_document(file, &text)?;
    if let Some(repos) = doc.get_mut("repos").and_then(Item::as_table_like_mut) {
        repos.remove(path);
    }
    write(dir, file, &doc.to_string())
}

fn write(dir: &Path, file: &str, text: &str) -> Result<(), Error> {
    fs::write(dir.join(file), text).map_err(|source| Error::Write {
        file: file.into(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write_file(dir: &Path, name: &str, text: &str) {
        fs::write(dir.join(name), text).unwrap();
    }

    #[test]
    fn later_files_replace_whole_entries() {
        let d = dir();
        write_file(
            d.path(),
            "a.repos",
            "repositories:\n  place:\n    type: git\n    url: https://a/place\n    version: v1\n",
        );
        write_file(
            d.path(),
            "place.toml",
            "[repos.place]\nurl = \"https://b/place\"\n",
        );
        let config = load(d.path(), &[]).unwrap();
        let entry = &config.repos["place"];
        assert_eq!(entry.url, "https://b/place");
        assert_eq!(entry.reference, None);
        assert_eq!(entry.files, ["a.repos", "place.toml"]);
    }

    #[test]
    fn environments_are_read_in_order() {
        let d = dir();
        write_file(
            d.path(),
            "place.ci.toml",
            "[repos.a]\nurl = \"https://ci/a\"\n",
        );
        write_file(
            d.path(),
            "place.local.toml",
            "[repos.a]\nurl = \"https://local/a\"\n",
        );
        write_file(
            d.path(),
            "place.ci.local.toml",
            "[repos.b]\nurl = \"https://b\"\n",
        );
        let without = load(d.path(), &[]).unwrap();
        assert_eq!(without.repos["a"].url, "https://local/a");
        assert!(!without.repos.contains_key("b"));
        let with = load(d.path(), &["ci".into()]).unwrap();
        assert_eq!(with.repos["a"].files, ["place.ci.toml", "place.local.toml"]);
        assert!(with.repos.contains_key("b"));
    }

    #[test]
    fn syntax_errors_name_the_line() {
        let d = dir();
        write_file(d.path(), "place.toml", "\n[repos.place\nurl = \"x\"\n");
        let err = load(d.path(), &[]).unwrap_err();
        assert_eq!(err.to_string(), "place.toml has a syntax error on line 2.");
    }

    #[test]
    fn add_entry_appends_a_table() {
        let d = dir();
        write_file(d.path(), "place.toml", "");
        add_entry(
            d.path(),
            "place.toml",
            "vendor/place",
            "https://x",
            Some("v1"),
        )
        .unwrap();
        let text = fs::read_to_string(d.path().join("place.toml")).unwrap();
        assert_eq!(
            text,
            "[repos.\"vendor/place\"]\nurl = \"https://x\"\nref = \"v1\"\n"
        );
    }

    #[test]
    fn add_entry_keeps_comments() {
        let d = dir();
        let original = "# Top.\n\n[repos.agents]   # keep me\nurl = \"https://a\"\n";
        write_file(d.path(), "place.toml", original);
        add_entry(d.path(), "place.toml", "place", "https://p", None).unwrap();
        let text = fs::read_to_string(d.path().join("place.toml")).unwrap();
        assert_eq!(
            text,
            format!("{original}\n[repos.place]\nurl = \"https://p\"\n")
        );
    }

    #[test]
    fn remove_entry_keeps_the_rest() {
        let d = dir();
        write_file(
            d.path(),
            "place.toml",
            "# Top.\n[repos.a]\nurl = \"https://a\"\n\n[repos.b]\nurl = \"https://b\"\n",
        );
        remove_entry(d.path(), "place.toml", "b").unwrap();
        let text = fs::read_to_string(d.path().join("place.toml")).unwrap();
        assert_eq!(text, "# Top.\n[repos.a]\nurl = \"https://a\"\n");
    }
}
