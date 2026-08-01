use std::env;

use common::gha_subscriber::GHALayer;
use publish_package::publish;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(GHALayer {})
        .with(LevelFilter::DEBUG)
        .init();

    let args: Vec<String> = env::args().collect();

    let target_pkg_dir = args
        .get(1)
        .expect("Expected a package directory as the first argument.");

    publish(target_pkg_dir);
}
