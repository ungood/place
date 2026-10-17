mod steps;

use cucumber::World as _;
use steps::world::PlaceWorld;

#[tokio::main]
async fn main() {
    PlaceWorld::cucumber()
        .fail_on_skipped()
        .filter_run_and_exit("features", |_, _, scenario| {
            !scenario.tags.iter().any(|tag| tag == "wip")
        })
        .await;
}
