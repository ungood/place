use std::fs;

use cucumber::gherkin::Step;
use cucumber::given;

use super::world::{PlaceWorld, docstring};

#[given(expr = "a remote repository {string}")]
async fn remote_repository(world: &mut PlaceWorld, url: String) {
    let bare = world.remote_path(&url);
    let scratch = world.scratch_path(&url);
    fs::create_dir_all(&bare).expect("create remote dir");
    fs::create_dir_all(&scratch).expect("create scratch dir");
    world
        .git(&bare, &["init", "-q", "--bare", "-b", "main"])
        .await;
    world.git(&scratch, &["init", "-q", "-b", "main"]).await;
    world
        .commit_file(
            &scratch,
            "README.md",
            &format!("# {url}\n"),
            "Initial commit",
        )
        .await;
    world
        .git(
            &scratch,
            &["remote", "add", "origin", &bare.display().to_string()],
        )
        .await;
    world.git(&scratch, &["push", "-q", "origin", "main"]).await;
}

#[given(expr = "the remote {string} has a branch {string}")]
async fn remote_branch(world: &mut PlaceWorld, url: String, branch: String) {
    let scratch = world.scratch_path(&url);
    world.git(&scratch, &["branch", &branch, "main"]).await;
    world
        .git(&scratch, &["push", "-q", "origin", &branch])
        .await;
}

#[given(expr = "the remote {string} has a tag {string}")]
async fn remote_tag(world: &mut PlaceWorld, url: String, tag: String) {
    let scratch = world.scratch_path(&url);
    world.git(&scratch, &["tag", &tag, "main"]).await;
    world.git(&scratch, &["push", "-q", "origin", &tag]).await;
}

#[given(expr = "the remote {string} has a new commit on {string}")]
async fn remote_new_commit(world: &mut PlaceWorld, url: String, branch: String) {
    let scratch = world.scratch_path(&url);
    world.git(&scratch, &["switch", "-q", &branch]).await;
    world
        .commit_file(
            &scratch,
            "CHANGES.md",
            "A change on the remote.\n",
            "Remote change",
        )
        .await;
    world
        .git(&scratch, &["push", "-q", "origin", &branch])
        .await;
}

#[given(expr = "the remote {string} has the default branch {string}")]
async fn remote_default_branch(world: &mut PlaceWorld, url: String, branch: String) {
    let bare = world.remote_path(&url);
    world
        .git(
            &bare,
            &["symbolic-ref", "HEAD", &format!("refs/heads/{branch}")],
        )
        .await;
}

#[given(expr = "the remote {string} has the file {string}:")]
async fn remote_file(world: &mut PlaceWorld, url: String, file: String, step: &Step) {
    let scratch = world.scratch_path(&url);
    world.git(&scratch, &["switch", "-q", "main"]).await;
    world
        .commit_file(&scratch, &file, &docstring(step), &format!("Add {file}"))
        .await;
    world.git(&scratch, &["push", "-q", "origin", "main"]).await;
}
