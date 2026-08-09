use std::{env, process::exit};

use gha_main::gha_output;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use build_package::{BuildStatus, build_package};
use common::gha_subscriber::GHALayer;

#[expect(
    clippy::unnecessary_wraps,
    reason = "gha_main requires main() to return a Result, but we're handling the error stuff ourselves and only returning Ok(())"
)]
#[gha_main::gha_main]
fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(GHALayer {})
        .with(LevelFilter::DEBUG)
        .init();

    let args: Vec<String> = env::args().collect();

    let (output, build_status) = build_package(
        args.get(1)
            .expect("Package directory must be first argument"),
    );

    if args.contains(&"--normal".to_string()) {
        println!("{output}\n\nStatus: {build_status}");
    } else {
        let result = output.replace('\n', "\\n").replace('"', "\\\"");
        let failed = build_status.failed();

        gha_output!(result);
        gha_output!(failed);

        if build_status == BuildStatus::Failure {
            exit(1);
        }
    }

    Ok(())
}
