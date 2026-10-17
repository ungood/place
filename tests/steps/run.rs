use cucumber::when;

use super::world::PlaceWorld;

fn arguments(command: &str) -> Vec<String> {
    let mut words = command.split_whitespace();
    assert_eq!(
        words.next(),
        Some("place"),
        "commands must start with place"
    );
    words.map(String::from).collect()
}

#[when(expr = "I run {string}")]
fn run(world: &mut PlaceWorld, command: String) {
    let workplace = world.workplace.clone();
    world.last = Some(world.run_place(&arguments(&command), &workplace));
}

#[when(expr = "I run {string} in {string}")]
fn run_in(world: &mut PlaceWorld, command: String, dir: String) {
    let dir = world.path(&dir);
    world.last = Some(world.run_place(&arguments(&command), &dir));
}
