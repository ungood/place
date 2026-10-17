mod steps;

use cucumber::World as _;
use cucumber::writer::Stats as _;
use steps::{templates, world::PlaceWorld};

#[tokio::main]
async fn main() {
    let summary = PlaceWorld::cucumber()
        .fail_on_skipped()
        .filter_run("features", |_, _, scenario| {
            !scenario.tags.iter().any(|tag| tag == "wip")
        })
        .await;
    templates::clean_up();
    if summary.execution_has_failed() {
        std::process::exit(1);
    }
}
