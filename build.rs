//! Sets `PLACE_VERSION`, the version `place --version` prints.
//!
//! Cargo.toml holds a placeholder, and the git tags hold the real versions.
//! A release is built from its tag, such as `v0.2.0`, and prints `0.2.0`.
//! Any other build says how far it is from the last release, such as
//! `0.2.0-3-gabc1234`. Without git history, it prints the placeholder.
//! A `PLACE_VERSION` set in the environment wins over all of these, for
//! builds that have no git history, such as the nix flake's.

use std::path::Path;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn main() {
    println!("cargo:rerun-if-env-changed=PLACE_VERSION");
    let version = std::env::var("PLACE_VERSION")
        .ok()
        .filter(|version| !version.is_empty())
        .or_else(|| {
            git(&["describe", "--tags", "--match", "v[0-9]*"])
                .and_then(|tag| tag.strip_prefix('v').map(String::from))
        })
        .unwrap_or_else(|| std::env::var("CARGO_PKG_VERSION").expect("cargo sets it"));
    println!("cargo:rustc-env=PLACE_VERSION={version}");

    // Describe again when HEAD moves, a commit lands on its branch, or tags
    // change. A worktree keeps its HEAD apart from the shared refs.
    let watch = [
        git(&["rev-parse", "--git-path", "HEAD"]),
        git(&["rev-parse", "--symbolic-full-name", "HEAD"])
            .and_then(|branch| git(&["rev-parse", "--git-path", &branch])),
        git(&["rev-parse", "--git-path", "refs/tags"]),
        git(&["rev-parse", "--git-path", "packed-refs"]),
    ];
    for path in watch.into_iter().flatten() {
        // Cargo reruns the script every build for a path that doesn't exist.
        if Path::new(&path).exists() {
            println!("cargo:rerun-if-changed={path}");
        }
    }
}
