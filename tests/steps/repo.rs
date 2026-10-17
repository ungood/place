use std::fs;

use cucumber::{given, then};

use super::world::PlaceWorld;

const CHANGED: &str = "# A local change\n";

#[given(expr = "{string} is a clone of {string}")]
fn clone(world: &mut PlaceWorld, path: String, url: String) {
    let workplace = world.workplace.clone();
    world.git(&workplace, &["clone", "-q", &url, &path]);
}

#[given(expr = "{string} is on a new branch {string}")]
fn new_branch(world: &mut PlaceWorld, path: String, branch: String) {
    world.git(&world.path(&path), &["switch", "-q", "-c", &branch]);
}

#[given(expr = "{string} has an uncommitted change")]
fn uncommitted_change(world: &mut PlaceWorld, path: String) {
    fs::write(world.path(&path).join("README.md"), CHANGED).expect("write change");
    world.uncommitted.insert(path, CHANGED.to_string());
}

#[given(expr = "{string} has a staged change")]
fn staged_change(world: &mut PlaceWorld, path: String) {
    let dir = world.path(&path);
    fs::write(dir.join("README.md"), CHANGED).expect("write change");
    world.git(&dir, &["add", "README.md"]);
}

#[given(expr = "{string} has an untracked file")]
fn untracked_file(world: &mut PlaceWorld, path: String) {
    fs::write(world.path(&path).join("scratch.txt"), "Untracked.\n").expect("write file");
}

#[given(expr = "{string} has a stash")]
fn stash(world: &mut PlaceWorld, path: String) {
    let dir = world.path(&path);
    fs::write(dir.join("README.md"), CHANGED).expect("write change");
    world.git(&dir, &["stash", "-q"]);
}

#[given(expr = "{string} has a local commit on {string}")]
fn local_commit(world: &mut PlaceWorld, path: String, branch: String) {
    let dir = world.path(&path);
    world.git(&dir, &["switch", "-q", &branch]);
    let commit = world.commit_file(&dir, "LOCAL.md", "A local commit.\n", "Local change");
    world.local_commits.insert(path, commit);
}

#[given(expr = "{string} has a local branch {string} with its own commit")]
fn local_branch_with_commit(world: &mut PlaceWorld, path: String, branch: String) {
    let dir = world.path(&path);
    let current = world.git(&dir, &["symbolic-ref", "--short", "HEAD"]);
    world.git(&dir, &["switch", "-q", "-c", &branch]);
    world.commit_file(&dir, "SPIKE.md", "A spike.\n", "Spike");
    world.git(&dir, &["switch", "-q", &current]);
}

#[given(expr = "{string} has a local branch {string} at {string}")]
fn local_branch_at(world: &mut PlaceWorld, path: String, branch: String, at: String) {
    world.git(&world.path(&path), &["branch", &branch, &at]);
}

#[given(expr = "the origin of {string} is {string}")]
fn set_origin(world: &mut PlaceWorld, path: String, url: String) {
    world.git(&world.path(&path), &["remote", "set-url", "origin", &url]);
}

#[then(expr = "{string} is a clone of {string}")]
fn is_clone_of(world: &mut PlaceWorld, path: String, url: String) {
    let dir = world.path(&path);
    assert!(dir.join(".git").exists(), "{path} isn't a git checkout");
    let origin = world.git(&dir, &["remote", "get-url", "origin"]);
    assert_eq!(origin, url, "{path} has the wrong origin");
}

#[then(expr = "{string} is checked out on {string}")]
fn checked_out_on(world: &mut PlaceWorld, path: String, branch: String) {
    let dir = world.path(&path);
    assert!(dir.join(".git").exists(), "{path} isn't a git checkout");
    let head = world.git(&dir, &["symbolic-ref", "--short", "HEAD"]);
    assert_eq!(head, branch, "{path} is on the wrong branch");
}

#[then(expr = "{string} is detached at the tag {string}")]
fn detached_at_tag(world: &mut PlaceWorld, path: String, tag: String) {
    let dir = world.path(&path);
    assert!(dir.join(".git").exists(), "{path} isn't a git checkout");
    assert!(
        !world.git_ok(&dir, &["symbolic-ref", "-q", "HEAD"]),
        "{path} is on a branch, not detached"
    );
    let head = world.git(&dir, &["rev-parse", "HEAD"]);
    let tagged = world.git(&dir, &["rev-parse", &format!("{tag}^{{commit}}")]);
    assert_eq!(head, tagged, "{path} isn't at the tag {tag}");
}

fn remote_head(world: &PlaceWorld, path: &str, branch: &str) -> String {
    let dir = world.path(path);
    let origin = world.git(&dir, &["remote", "get-url", "origin"]);
    let bare = world.remote_path(&origin);
    world.git(&bare, &["rev-parse", &format!("refs/heads/{branch}")])
}

#[then(expr = "{string} is at the remote's latest commit on {string}")]
fn at_remote_latest(world: &mut PlaceWorld, path: String, branch: String) {
    let head = world.git(&world.path(&path), &["rev-parse", "HEAD"]);
    assert_eq!(
        head,
        remote_head(world, &path, &branch),
        "{path} isn't up to date"
    );
}

#[then(expr = "{string} is not at the remote's latest commit on {string}")]
fn not_at_remote_latest(world: &mut PlaceWorld, path: String, branch: String) {
    let head = world.git(&world.path(&path), &["rev-parse", "HEAD"]);
    assert_ne!(
        head,
        remote_head(world, &path, &branch),
        "{path} was updated"
    );
}

#[then(expr = "the local branch {string} of {string} is at the remote's latest commit on {string}")]
fn local_branch_at_remote_latest(
    world: &mut PlaceWorld,
    branch: String,
    path: String,
    remote_branch: String,
) {
    let local = world.git(
        &world.path(&path),
        &["rev-parse", &format!("refs/heads/{branch}")],
    );
    assert_eq!(
        local,
        remote_head(world, &path, &remote_branch),
        "{branch} in {path} isn't up to date"
    );
}

#[then(expr = "{string} still has its uncommitted change")]
fn still_uncommitted(world: &mut PlaceWorld, path: String) {
    let expected = world
        .uncommitted
        .get(&path)
        .unwrap_or_else(|| panic!("no uncommitted change was made in {path}"));
    let actual = fs::read_to_string(world.path(&path).join("README.md")).expect("read README");
    assert_eq!(&actual, expected, "{path} lost its uncommitted change");
}

#[then(expr = "{string} still has its local commit")]
fn still_local_commit(world: &mut PlaceWorld, path: String) {
    let commit = world
        .local_commits
        .get(&path)
        .unwrap_or_else(|| panic!("no local commit was made in {path}"))
        .clone();
    assert!(
        world.git_ok(
            &world.path(&path),
            &["merge-base", "--is-ancestor", &commit, "HEAD"]
        ),
        "{path} lost its local commit"
    );
}
