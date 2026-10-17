//! Test remotes are built once per run and copied into each scenario, which
//! is much faster than running git to create them every time.
//!
//! A template mirrors a scenario's layout: the bare repository lives at
//! `remotes/github.com/<owner>/<repo>` and a working clone used to push
//! commits lives at `scratch/<owner>/<repo>`. The clone's origin is a
//! relative path, so a copy works without any changes.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

use tokio::sync::OnceCell;

use super::world::{GIT_IDENTITY, run};

type Templates = Mutex<HashMap<String, Arc<OnceCell<()>>>>;

static TEMPLATES: LazyLock<Templates> = LazyLock::new(Default::default);

/// Where this run keeps its templates. Each process gets its own directory,
/// so concurrent test runs don't share one. It sits next to the scenarios'
/// temp dirs, on the same volume, so copies can be copy-on-write clones.
pub fn templates_dir() -> PathBuf {
    std::env::temp_dir().join(format!("place-cucumber-remotes-{}", std::process::id()))
}

/// The repository's path under github.com, such as `ungood/place.git`.
pub fn github_path(url: &str) -> &str {
    url.strip_prefix("https://github.com/")
        .unwrap_or_else(|| panic!("test remotes must be https://github.com/ URLs, got {url}"))
}

/// Copy the template for `url` into `base`, building the template first if
/// this is the first scenario to use it.
pub async fn install(url: &str, base: &Path) {
    let cell = TEMPLATES
        .lock()
        .expect("templates lock")
        .entry(url.to_string())
        .or_default()
        .clone();
    cell.get_or_init(|| build(url)).await;

    let rest = github_path(url);
    let templates = templates_dir();
    for sub in [
        format!("remotes/github.com/{rest}"),
        format!("scratch/{rest}"),
    ] {
        copy_dir(&templates.join(&sub), &base.join(&sub));
    }
}

async fn build(url: &str) {
    let rest = github_path(url);
    let templates = templates_dir();
    let bare = templates.join("remotes/github.com").join(rest);
    let scratch = templates.join("scratch").join(rest);
    fs::create_dir_all(&bare).expect("create template remote");
    fs::create_dir_all(&scratch).expect("create template scratch clone");

    // From scratch/<owner>/<repo>, climb back to the template root.
    let up = "../".repeat(1 + rest.split('/').count());
    let origin = format!("{up}remotes/github.com/{rest}");

    git(
        &bare,
        &["init", "-q", "--bare", "--template=", "-b", "main"],
    )
    .await;
    git(&scratch, &["init", "-q", "--template=", "-b", "main"]).await;
    fs::write(scratch.join("README.md"), format!("# {url}\n")).expect("write README");
    git(&scratch, &["add", "README.md"]).await;
    git(&scratch, &["commit", "-q", "-m", "Initial commit"]).await;
    git(&scratch, &["remote", "add", "origin", &origin]).await;
    git(&scratch, &["push", "-q", "origin", "main"]).await;
}

async fn git(dir: &Path, args: &[&str]) {
    let env = vec![
        (
            "PATH".to_string(),
            std::env::var("PATH").unwrap_or_default(),
        ),
        ("HOME".to_string(), templates_dir().display().to_string()),
        ("GIT_CONFIG_GLOBAL".to_string(), "/dev/null".to_string()),
        ("GIT_CONFIG_NOSYSTEM".to_string(), "1".to_string()),
    ]
    .into_iter()
    .chain(
        GIT_IDENTITY
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string())),
    )
    .collect();
    let output = run("git", args, dir, env).await;
    assert!(
        output.status.success(),
        "git {} failed in {}:\n{}",
        args.join(" "),
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create directory");
    for entry in fs::read_dir(from).expect("read template directory") {
        let entry = entry.expect("read template entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copy template file");
        }
    }
}

/// Delete this run's templates.
pub fn clean_up() {
    let _ = fs::remove_dir_all(templates_dir());
}
