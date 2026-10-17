use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use cucumber::World;
use tempfile::TempDir;

/// Every commit made by the harness uses this identity and date, so commit
/// hashes are the same on every run.
const GIT_IDENTITY: [(&str, &str); 6] = [
    ("GIT_AUTHOR_NAME", "Place Test"),
    ("GIT_AUTHOR_EMAIL", "test@place.invalid"),
    ("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z"),
    ("GIT_COMMITTER_NAME", "Place Test"),
    ("GIT_COMMITTER_EMAIL", "test@place.invalid"),
    ("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z"),
];

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct PlaceWorld {
    root: TempDir,
    /// The workplace, where `I run` runs commands.
    pub workplace: PathBuf,
    /// Extra environment variables for place.
    pub env: Vec<(String, String)>,
    /// The result of the last `I run`.
    pub last: Option<Output>,
    /// File contents as written by setup steps, for `is unchanged`.
    pub written: HashMap<String, String>,
    /// Commits made by `has a local commit`, for `still has its local commit`.
    pub local_commits: HashMap<String, String>,
    /// Content written by `has an uncommitted change`.
    pub uncommitted: HashMap<String, String>,
}

impl PlaceWorld {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("create temp dir");
        let base = root.path().canonicalize().expect("canonicalize temp dir");
        let workplace = base.join("workplace");
        let remotes = base.join("remotes");
        let home = base.join("home");
        for dir in [&workplace, &remotes, &home] {
            fs::create_dir_all(dir).expect("create test dir");
        }
        let github = format!("file://{}/github.com/", remotes.display());
        fs::write(
            home.join(".gitconfig"),
            format!(
                "[init]\n\tdefaultBranch = main\n\
                 [commit]\n\tgpgsign = false\n\
                 [tag]\n\tgpgsign = false\n\
                 [protocol \"file\"]\n\tallow = always\n\
                 [url \"{github}\"]\n\
                 \tinsteadOf = https://github.com/\n\
                 \tinsteadOf = git@github.com:\n\
                 \tinsteadOf = ssh://git@github.com/\n"
            ),
        )
        .expect("write test gitconfig");
        Self {
            root,
            workplace,
            env: Vec::new(),
            last: None,
            written: HashMap::new(),
            local_commits: HashMap::new(),
            uncommitted: HashMap::new(),
        }
    }

    fn base(&self) -> PathBuf {
        self.root
            .path()
            .canonicalize()
            .expect("canonicalize temp dir")
    }

    /// The environment every git and place process runs with: an isolated
    /// home and git config, no colors, and the fixed commit identity.
    pub fn process_env(&self) -> Vec<(String, String)> {
        let home = self.base().join("home");
        let mut env = vec![
            ("PATH".into(), std::env::var("PATH").unwrap_or_default()),
            ("HOME".into(), home.display().to_string()),
            (
                "GIT_CONFIG_GLOBAL".into(),
                home.join(".gitconfig").display().to_string(),
            ),
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
            ("GIT_TERMINAL_PROMPT".into(), "0".into()),
            ("NO_COLOR".into(), "1".into()),
        ];
        env.extend(
            GIT_IDENTITY
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
        );
        env
    }

    /// Where the bare repository for a GitHub URL lives.
    pub fn remote_path(&self, url: &str) -> PathBuf {
        let rest = url
            .strip_prefix("https://github.com/")
            .unwrap_or_else(|| panic!("test remotes must be https://github.com/ URLs, got {url}"));
        self.base().join("remotes/github.com").join(rest)
    }

    /// A working clone of a remote, used to make commits and push them.
    pub fn scratch_path(&self, url: &str) -> PathBuf {
        let rest = url.strip_prefix("https://github.com/").expect("github URL");
        self.base().join("scratch").join(rest)
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.workplace.join(relative)
    }

    pub fn git(&self, dir: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env_clear()
            .envs(self.process_env())
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {} failed in {}:\n{}",
            args.join(" "),
            dir.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    pub fn git_ok(&self, dir: &Path, args: &[&str]) -> bool {
        Command::new("git")
            .args(args)
            .current_dir(dir)
            .env_clear()
            .envs(self.process_env())
            .output()
            .expect("run git")
            .status
            .success()
    }

    /// Commit a file in a directory, writing `content` to it first.
    pub fn commit_file(&self, dir: &Path, file: &str, content: &str, message: &str) -> String {
        let path = dir.join(file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(&path, content).expect("write file");
        self.git(dir, &["add", file]);
        self.git(dir, &["commit", "-q", "-m", message]);
        self.git(dir, &["rev-parse", "HEAD"])
    }

    pub fn last(&self) -> &Output {
        self.last
            .as_ref()
            .expect("a step must run a command before checking its result")
    }

    pub fn stdout(&self) -> String {
        clean(&self.last().stdout)
    }

    pub fn stderr(&self) -> String {
        clean(&self.last().stderr)
    }

    /// Run place with the given arguments in a directory.
    pub fn run_place(&self, args: &[String], dir: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_place"))
            .args(args)
            .current_dir(dir)
            .env_clear()
            .envs(self.process_env())
            .envs(self.env.iter().cloned())
            .output()
            .expect("run place")
    }
}

/// Remove colors and the decoration miette draws around messages, and
/// collapse whitespace, so steps match the words rather than the layout.
pub fn clean(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(&strip_ansi_escapes::strip(bytes)).to_string();
    let text: String = text
        .chars()
        .map(|c| {
            if "│×╭╰─┬┴├┤╮╯⚠✖‽".contains(c) {
                ' '
            } else {
                c
            }
        })
        .collect();
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalize expected text from a doc string the same way.
pub fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The doc string attached to a step, without the leading newline gherkin keeps.
pub fn docstring(step: &cucumber::gherkin::Step) -> String {
    let text = step
        .docstring
        .as_deref()
        .expect("this step needs a doc string");
    text.strip_prefix('\n').unwrap_or(text).to_string()
}
