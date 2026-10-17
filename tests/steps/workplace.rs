use std::fs;

use cucumber::gherkin::Step;
use cucumber::{given, then};

use super::world::{PlaceWorld, docstring};

fn write_file(world: &mut PlaceWorld, file: &str, content: String) {
    let path = world.path(file);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(&path, &content).expect("write workplace file");
    world.written.insert(file.to_string(), content);
}

fn read_file(world: &PlaceWorld, file: &str) -> String {
    fs::read_to_string(world.path(file))
        .unwrap_or_else(|e| panic!("can't read workplace file {file}: {e}"))
}

fn repos(world: &PlaceWorld, file: &str) -> Vec<String> {
    let content = read_file(world, file);
    let table: toml::Table = content
        .parse()
        .unwrap_or_else(|e| panic!("{file} isn't valid TOML: {e}\n{content}"));
    match table.get("repos") {
        None => Vec::new(),
        Some(toml::Value::Table(repos)) => repos.keys().cloned().collect(),
        Some(other) => panic!("repos in {file} isn't a table: {other}"),
    }
}

#[given("an empty directory")]
fn empty_directory(_world: &mut PlaceWorld) {}

#[given("an empty workplace")]
fn empty_workplace(world: &mut PlaceWorld) {
    write_file(world, "place.toml", String::new());
}

#[given(expr = "a workplace with the file {string}:")]
#[given(expr = "the workplace has the file {string}:")]
fn workplace_file(world: &mut PlaceWorld, file: String, step: &Step) {
    write_file(world, &file, docstring(step));
}

#[given(expr = "the workplace has a plain directory {string}")]
fn plain_directory(world: &mut PlaceWorld, dir: String) {
    let path = world.path(&dir);
    fs::create_dir_all(&path).expect("create plain directory");
    fs::write(path.join("notes.txt"), "Not a git checkout.\n").expect("write file");
}

#[given("the workplace is a git repository")]
fn workplace_is_git(world: &mut PlaceWorld) {
    let workplace = world.workplace.clone();
    world.git(&workplace, &["init", "-q"]);
}

#[given(expr = "the environment variable {string} is {string}")]
fn environment_variable(world: &mut PlaceWorld, name: String, value: String) {
    world.env.push((name, value));
}

#[then(expr = "the workplace file {string} is:")]
fn file_is(world: &mut PlaceWorld, file: String, step: &Step) {
    let actual = read_file(world, &file);
    let expected = docstring(step);
    assert_eq!(
        actual.trim_end(),
        expected.trim_end(),
        "{file} doesn't have the expected content"
    );
}

#[then(expr = "the workplace file {string} has no repos")]
fn file_has_no_repos(world: &mut PlaceWorld, file: String) {
    let listed = repos(world, &file);
    assert!(listed.is_empty(), "{file} lists repos: {listed:?}");
}

#[then(expr = "the workplace file {string} lists {string}")]
fn file_lists(world: &mut PlaceWorld, file: String, path: String) {
    let listed = repos(world, &file);
    assert!(
        listed.contains(&path),
        "{file} doesn't list {path}: {listed:?}"
    );
}

#[then(expr = "the workplace file {string} is unchanged")]
fn file_unchanged(world: &mut PlaceWorld, file: String) {
    let original = world
        .written
        .get(&file)
        .unwrap_or_else(|| panic!("{file} wasn't written by a setup step"))
        .clone();
    assert_eq!(read_file(world, &file), original, "{file} was changed");
}

#[then(expr = "the workplace file {string} does not exist")]
fn file_does_not_exist(world: &mut PlaceWorld, file: String) {
    assert!(!world.path(&file).exists(), "{file} exists");
}

#[then(expr = "the workplace directory {string} exists")]
fn directory_exists(world: &mut PlaceWorld, dir: String) {
    assert!(world.path(&dir).is_dir(), "{dir} doesn't exist");
}

#[then(expr = "the workplace directory {string} does not exist")]
fn directory_does_not_exist(world: &mut PlaceWorld, dir: String) {
    assert!(!world.path(&dir).exists(), "{dir} exists");
}
