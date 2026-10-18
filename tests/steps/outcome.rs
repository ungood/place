use cucumber::gherkin::Step;
use cucumber::then;
use serde_json::Value;

use super::world::{PlaceWorld, clean, docstring, normalize};

fn describe(world: &PlaceWorld) -> String {
    let last = world.last();
    format!(
        "exit status: {}\nstdout: {}\nstderr: {}",
        last.status,
        world.stdout(),
        world.stderr()
    )
}

#[then("the command succeeds")]
async fn succeeds(world: &mut PlaceWorld) {
    assert!(
        world.last().status.success(),
        "the command failed\n{}",
        describe(world)
    );
}

#[then("the command fails")]
async fn fails(world: &mut PlaceWorld) {
    assert!(
        !world.last().status.success(),
        "the command succeeded\n{}",
        describe(world)
    );
}

#[then(expr = "the command fails with exit code {int}")]
async fn fails_with_code(world: &mut PlaceWorld, code: i32) {
    assert_eq!(
        world.last().status.code(),
        Some(code),
        "wrong exit code\n{}",
        describe(world)
    );
}

fn assert_contains(world: &PlaceWorld, haystack: &str, step: &Step, what: &str) {
    for line in docstring(step).lines().filter(|l| !l.trim().is_empty()) {
        let expected = normalize(line);
        assert!(
            haystack.contains(&expected),
            "the {what} doesn't say:\n  {expected}\n{}",
            describe(world)
        );
    }
}

#[then("the error says:")]
async fn error_says(world: &mut PlaceWorld, step: &Step) {
    let stderr = world.stderr();
    assert_contains(world, &stderr, step, "error");
}

#[then("the help says:")]
async fn help_says(world: &mut PlaceWorld, step: &Step) {
    let stderr = world.stderr();
    let help = stderr
        .split_once("help:")
        .map(|(_, help)| help.to_string())
        .unwrap_or_else(|| panic!("the error has no help\n{}", describe(world)));
    assert_contains(world, &help, step, "help");
}

#[then("place warns:")]
async fn place_warns(world: &mut PlaceWorld, step: &Step) {
    let output = format!("{} {}", world.stdout(), world.stderr());
    assert_contains(world, &output, step, "warning");
}

#[then("the output says:")]
async fn output_says(world: &mut PlaceWorld, step: &Step) {
    let stdout = world.stdout();
    assert_contains(world, &stdout, step, "output");
}

#[then("place prints:")]
async fn place_prints(world: &mut PlaceWorld, step: &Step) {
    let output = normalize(&format!("{} {}", world.stdout(), world.stderr()));
    assert_contains(world, &output, step, "output");
}

/// The states `place status --json` reports for a repo. The JSON has the
/// shape `{"repos": [{"path": "place", "states": ["on ref", "dirty"]}]}`.
async fn states(world: &PlaceWorld, path: &str) -> Vec<String> {
    let output = world
        .run_place(&["status".into(), "--json".into()], &world.workplace)
        .await;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "place status --json didn't print JSON ({e})\nstdout: {stdout}\nstderr: {}",
            clean(&output.stderr)
        )
    });
    let repo = json["repos"]
        .as_array()
        .and_then(|repos| repos.iter().find(|r| r["path"] == path))
        .unwrap_or_else(|| panic!("place status --json doesn't list {path}: {json}"));
    repo["states"]
        .as_array()
        .unwrap_or_else(|| panic!("{path} has no states: {repo}"))
        .iter()
        .filter_map(|s| s.as_str().map(String::from))
        .collect()
}

#[then(expr = "place reports {string} as {string}")]
async fn reports(world: &mut PlaceWorld, path: String, state: String) {
    let states = states(world, &path).await;
    assert!(
        states.contains(&state),
        "{path} isn't reported as {state}: {states:?}"
    );
}

#[then(expr = "place doesn't report {string} as {string}")]
async fn does_not_report(world: &mut PlaceWorld, path: String, state: String) {
    let states = states(world, &path).await;
    assert!(
        !states.contains(&state),
        "{path} is reported as {state}: {states:?}"
    );
}
