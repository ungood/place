use std::fs;

use cucumber::gherkin::Step;
use cucumber::given;

use super::world::{PlaceWorld, docstring};

#[given(expr = "a remote repository {string}")]
fn remote_repository(world: &mut PlaceWorld, url: String) {
    let bare = world.remote_path(&url);
    let scratch = world.scratch_path(&url);
    fs::create_dir_all(&bare).expect("create remote dir");
    fs::create_dir_all(&scratch).expect("create scratch dir");
    world.git(&bare, &["init", "-q", "--bare", "-b", "main"]);
    world.git(&scratch, &["init", "-q", "-b", "main"]);
    world.commit_file(
        &scratch,
        "README.md",
        &format!("# {url}\n"),
        "Initial commit",
    );
    world.git(
        &scratch,
        &["remote", "add", "origin", &bare.display().to_string()],
    );
    world.git(&scratch, &["push", "-q", "origin", "main"]);
}

#[given(expr = "the remote {string} has a branch {string}")]
fn remote_branch(world: &mut PlaceWorld, url: String, branch: String) {
    let scratch = world.scratch_path(&url);
    world.git(&scratch, &["branch", &branch, "main"]);
    world.git(&scratch, &["push", "-q", "origin", &branch]);
}

#[given(expr = "the remote {string} has a tag {string}")]
fn remote_tag(world: &mut PlaceWorld, url: String, tag: String) {
    let scratch = world.scratch_path(&url);
    world.git(&scratch, &["tag", &tag, "main"]);
    world.git(&scratch, &["push", "-q", "origin", &tag]);
}

#[given(expr = "the remote {string} has a new commit on {string}")]
fn remote_new_commit(world: &mut PlaceWorld, url: String, branch: String) {
    let scratch = world.scratch_path(&url);
    world.git(&scratch, &["switch", "-q", &branch]);
    world.commit_file(
        &scratch,
        "CHANGES.md",
        "A change on the remote.\n",
        "Remote change",
    );
    world.git(&scratch, &["push", "-q", "origin", &branch]);
}

#[given(expr = "the remote {string} has the default branch {string}")]
fn remote_default_branch(world: &mut PlaceWorld, url: String, branch: String) {
    let bare = world.remote_path(&url);
    world.git(
        &bare,
        &["symbolic-ref", "HEAD", &format!("refs/heads/{branch}")],
    );
}

#[given(expr = "the remote {string} has the file {string}:")]
fn remote_file(world: &mut PlaceWorld, url: String, file: String, step: &Step) {
    let scratch = world.scratch_path(&url);
    world.git(&scratch, &["switch", "-q", "main"]);
    world.commit_file(&scratch, &file, &docstring(step), &format!("Add {file}"));
    world.git(&scratch, &["push", "-q", "origin", "main"]);
}
