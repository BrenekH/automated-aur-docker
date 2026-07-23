// TODO: Write a subscriber/layer for tracing which maps the info!, debug!, warn!, and error!. Optionally allow groups to be defined by spans.

use std::collections::HashMap;

use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{Layer, layer::Context};

pub struct GHALayer {}

impl<S: Subscriber> Layer<S> for GHALayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Triggered when an event (e.g., info!, error!) occurs
        println!("Event received: {event:?}");

        let mut visitor = Visitor::default();
        event.record(&mut visitor);

        for field in event.fields() {
            if field.name() == "message" {
                continue;
            }

            println!("{}: {}", field.name(), visitor.other_values[field.name()]);
        }

        match *event.metadata().level() {
            Level::TRACE => {}
            Level::DEBUG => output_gha_command("debug", &HashMap::new(), &visitor.message),
            Level::INFO => println!("{}", visitor.message),
            Level::WARN => {
                output_gha_command("warning", &HashMap::new(), &visitor.message);
            }
            Level::ERROR => output_gha_command("error", &HashMap::new(), &visitor.message),
        }
    }
}

#[derive(Default, Debug)]
struct Visitor {
    message: String,
    other_values: HashMap<&'static str, String>,
}

impl tracing::field::Visit for Visitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            // Extract the field value if the name matches our searched field name
            self.message = format!("{value:?}");
            return;
        }

        self.other_values.insert(field.name(), format!("{value:?}"));
    }
}

fn output_gha_command<S: std::fmt::Display>(command: S, parameters: &HashMap<S, S>, value: S) {
    let formatted_params: Vec<String> = parameters
        .iter()
        .map(|(key, val)| {
            format!(
                "{key}={}",
                // Encode value (https://github.com/orgs/community/discussions/26736#discussioncomment-3253165)
                val.to_string()
                    .replace('%', "%25")
                    .replace('\r', "%0D")
                    .replace('\n', "%0A")
                    .replace(':', "%3A")
                    .replace(',', "%2C")
            )
        })
        .collect();

    let param_str = if formatted_params.is_empty() {
        String::new()
    } else {
        formatted_params.join(",")
    };

    println!(
        "::{command}{param_str}::{}",
        // Encode value (https://github.com/orgs/community/discussions/26736#discussioncomment-3253165)
        value
            .to_string()
            .replace('%', "%25")
            .replace('\r', "%0D")
            .replace('\n', "%0A")
    );
}
