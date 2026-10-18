use std::fs;

use cucumber::{given, then};

use super::world::PlaceWorld;

const CHANGED: &str = "# A local change\n";

#[given(expr = "{string} is a clone of {string}")]
async fn clone(world: &mut PlaceWorld, path: String, url: String) {
    let workplace = world.workplace.clone();
    world.git(&workplace, &["clone", "-q", &url, &path]).await;
}

#[given(expr = "{string} is on a new branch {string}")]
async fn new_branch(world: &mut PlaceWorld, path: String, branch: String) {
    world
        .git(&world.path(&path), &["switch", "-q", "-c", &branch])
        .await;
}

#[given(expr = "{string} has a detached HEAD")]
async fn detached_head(world: &mut PlaceWorld, path: String) {
    world
        .git(&world.path(&path), &["switch", "-q", "--detach"])
        .await;
}

#[given(expr = "{string} has an uncommitted change")]
async fn uncommitted_change(world: &mut PlaceWorld, path: String) {
    fs::write(world.path(&path).join("README.md"), CHANGED).expect("write change");
    world.uncommitted.insert(path, CHANGED.to_string());
}

#[given(expr = "{string} has a staged change")]
async fn staged_change(world: &mut PlaceWorld, path: String) {
    let dir = world.path(&path);
    fs::write(dir.join("README.md"), CHANGED).expect("write change");
    world.git(&dir, &["add", "README.md"]).await;
}

#[given(expr = "{string} has an untracked file")]
async fn untracked_file(world: &mut PlaceWorld, path: String) {
    fs::write(world.path(&path).join("scratch.txt"), "Untracked.\n").expect("write file");
}

#[given(expr = "{string} has a stash")]
async fn stash(world: &mut PlaceWorld, path: String) {
    let dir = world.path(&path);
    fs::write(dir.join("README.md"), CHANGED).expect("write change");
    world.git(&dir, &["stash", "-q"]).await;
}

#[given(expr = "{string} has a local commit on {string}")]
async fn local_commit(world: &mut PlaceWorld, path: String, branch: String) {
    let dir = world.path(&path);
    world.git(&dir, &["switch", "-q", &branch]).await;
    let commit = world
        .commit_file(&dir, "LOCAL.md", "A local commit.\n", "Local change")
        .await;
    world.local_commits.insert(path, commit);
}

#[given(expr = "{string} has a local branch {string} with its own commit")]
async fn local_branch_with_commit(world: &mut PlaceWorld, path: String, branch: String) {
    let dir = world.path(&path);
    let current = world.git(&dir, &["symbolic-ref", "--short", "HEAD"]).await;
    world.git(&dir, &["switch", "-q", "-c", &branch]).await;
    world
        .commit_file(&dir, "SPIKE.md", "A spike.\n", "Spike")
        .await;
    world.git(&dir, &["switch", "-q", &current]).await;
}

#[given(expr = "{string} has a local branch {string} at {string}")]
async fn local_branch_at(world: &mut PlaceWorld, path: String, branch: String, at: String) {
    world
        .git(&world.path(&path), &["branch", &branch, &at])
        .await;
}

#[given(expr = "the origin of {string} is {string}")]
async fn set_origin(world: &mut PlaceWorld, path: String, url: String) {
    world
        .git(&world.path(&path), &["remote", "set-url", "origin", &url])
        .await;
}

/// The origin URL as configured. `git remote get-url` would apply the test
/// config's `insteadOf` rewriting and return the local bare repo's path.
async fn origin_url(world: &PlaceWorld, dir: &std::path::Path) -> String {
    world
        .git(dir, &["config", "--get", "remote.origin.url"])
        .await
}

#[then(expr = "{string} is a clone of {string}")]
async fn is_clone_of(world: &mut PlaceWorld, path: String, url: String) {
    let dir = world.path(&path);
    assert!(dir.join(".git").exists(), "{path} isn't a git checkout");
    let origin = origin_url(world, &dir).await;
    assert_eq!(origin, url, "{path} has the wrong origin");
}

#[then(expr = "{string} is checked out on {string}")]
async fn checked_out_on(world: &mut PlaceWorld, path: String, branch: String) {
    let dir = world.path(&path);
    assert!(dir.join(".git").exists(), "{path} isn't a git checkout");
    let head = world.git(&dir, &["symbolic-ref", "--short", "HEAD"]).await;
    assert_eq!(head, branch, "{path} is on the wrong branch");
}

#[then(expr = "{string} is detached at the tag {string}")]
async fn detached_at_tag(world: &mut PlaceWorld, path: String, tag: String) {
    let dir = world.path(&path);
    assert!(dir.join(".git").exists(), "{path} isn't a git checkout");
    assert!(
        !world.git_ok(&dir, &["symbolic-ref", "-q", "HEAD"]).await,
        "{path} is on a branch, not detached"
    );
    let head = world.git(&dir, &["rev-parse", "HEAD"]).await;
    let tagged = world
        .git(&dir, &["rev-parse", &format!("{tag}^{{commit}}")])
        .await;
    assert_eq!(head, tagged, "{path} isn't at the tag {tag}");
}

async fn remote_head(world: &PlaceWorld, path: &str, branch: &str) -> String {
    let dir = world.path(path);
    let origin = origin_url(world, &dir).await;
    let bare = world.remote_path(&origin);
    world
        .git(&bare, &["rev-parse", &format!("refs/heads/{branch}")])
        .await
}

#[then(expr = "{string} is at the remote's latest commit on {string}")]
async fn at_remote_latest(world: &mut PlaceWorld, path: String, branch: String) {
    let head = world.git(&world.path(&path), &["rev-parse", "HEAD"]).await;
    assert_eq!(
        head,
        remote_head(world, &path, &branch).await,
        "{path} isn't up to date"
    );
}

#[then(expr = "{string} is not at the remote's latest commit on {string}")]
async fn not_at_remote_latest(world: &mut PlaceWorld, path: String, branch: String) {
    let head = world.git(&world.path(&path), &["rev-parse", "HEAD"]).await;
    assert_ne!(
        head,
        remote_head(world, &path, &branch).await,
        "{path} was updated"
    );
}

#[then(expr = "the local branch {string} of {string} is at the remote's latest commit on {string}")]
async fn local_branch_at_remote_latest(
    world: &mut PlaceWorld,
    branch: String,
    path: String,
    remote_branch: String,
) {
    let local = world
        .git(
            &world.path(&path),
            &["rev-parse", &format!("refs/heads/{branch}")],
        )
        .await;
    assert_eq!(
        local,
        remote_head(world, &path, &remote_branch).await,
        "{branch} in {path} isn't up to date"
    );
}

#[then(expr = "{string} still has its uncommitted change")]
async fn still_uncommitted(world: &mut PlaceWorld, path: String) {
    let expected = world
        .uncommitted
        .get(&path)
        .unwrap_or_else(|| panic!("no uncommitted change was made in {path}"));
    let actual = fs::read_to_string(world.path(&path).join("README.md")).expect("read README");
    assert_eq!(&actual, expected, "{path} lost its uncommitted change");
}

#[then(expr = "{string} still has its local commit")]
async fn still_local_commit(world: &mut PlaceWorld, path: String) {
    let commit = world
        .local_commits
        .get(&path)
        .unwrap_or_else(|| panic!("no local commit was made in {path}"))
        .clone();
    assert!(
        world
            .git_ok(
                &world.path(&path),
                &["merge-base", "--is-ancestor", &commit, "HEAD"]
            )
            .await,
        "{path} lost its local commit"
    );
}
