use std::{env, process::exit};

use gha_main::{GitHubActionResult, gha_output};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use common::gha_subscriber::GHALayer;

use test_package::{TestPkgError, test_package};

#[gha_main::gha_main]
fn main() -> GitHubActionResult {
    tracing_subscriber::registry()
        .with(GHALayer {})
        .with(LevelFilter::DEBUG)
        .init();

    let args: Vec<String> = env::args().collect();

    let output;
    let failed;
    match test_package(
        args.get(1)
            .expect("Package directory must be first argument"),
    ) {
        Ok(o) => {
            output = o;
            failed = false;
        }
        Err(TestPkgError::Cmd { output: o }) => {
            output = o;
            failed = true;
        }
        Err(TestPkgError::Io(e)) => {
            return Err(e.into());
        }
        Err(TestPkgError::Parse(e)) => {
            return Err(e.into());
        }
    }

    if args.contains(&"--normal".to_string()) {
        println!("{output}\n\nFailed: {failed}");
    } else {
        let result = output.replace('\n', "\\n").replace('"', "\\\"");
        gha_output!(result);
        gha_output!(failed);

        if failed {
            exit(1);
        }
    }

    Ok(())
}
