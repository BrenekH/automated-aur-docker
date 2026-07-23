// TODO: Write a subscriber/layer for tracing which maps the info!, debug!, warn!, and error!. Optionally allow groups to be defined by spans.

use std::collections::HashMap;

use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{Layer, layer::Context};

pub struct GHALayer {}

impl<S: Subscriber> Layer<S> for GHALayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Triggered when an event (e.g., info!, error!) occurs
        match *event.metadata().level() {
            Level::TRACE => {}
            Level::DEBUG => {
                let mut visitor = Visitor::default();
                event.record(&mut visitor);

                // TODO: Print fields recorded by visitor as well
                output_gha_command("debug", None::<&HashMap<&str, &str>>, &visitor.message);
            }
            Level::INFO => {
                let mut visitor = Visitor::default();
                event.record(&mut visitor);

                // TODO: Print fields recorded by visitor as well
                println!("{}", visitor.message);
            }
            Level::WARN => {
                let mut visitor = FileContextVisitor::default();
                event.record(&mut visitor);

                // TODO: Print misc fields recorded by visitor as well
                output_gha_command("warning", Some(&visitor.parameters), &visitor.message);
            }
            Level::ERROR => {
                let mut visitor = FileContextVisitor::default();
                event.record(&mut visitor);

                // TODO: Print misc fields recorded by visitor as well
                output_gha_command("error", Some(&visitor.parameters), &visitor.message);
            }
        }
    }
}

#[derive(Default, Debug)]
struct Visitor {
    message: String,
    fields: Vec<(&'static str, String)>,
}

impl tracing::field::Visit for Visitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
            return;
        }

        self.fields.push((field.name(), format!("{value:?}")));
    }
}

#[derive(Default, Debug)]
struct FileContextVisitor {
    message: String,
    parameters: HashMap<&'static str, String>,
    fields: Vec<(&'static str, String)>,
}

impl tracing::field::Visit for FileContextVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let value_string = format!("{value:?}");

        match field.name() {
            "message" => {
                self.message = value_string;
            }
            "title" | "file" | "col" | "end_column" | "line" | "end_line" => {
                self.parameters.insert(field.name(), value_string);
            }
            _ => {
                self.fields.push((field.name(), value_string));
            }
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "message" => {
                value.clone_into(&mut self.message);
            }
            "title" | "file" | "col" | "end_column" | "line" | "end_line" => {
                self.parameters.insert(field.name(), value.to_owned());
            }
            _ => {
                self.fields.push((field.name(), value.to_owned()));
            }
        }
    }
}

fn output_gha_command(
    command: impl std::fmt::Display,
    parameters: Option<&HashMap<impl std::fmt::Display, impl std::fmt::Display>>,
    value: impl std::fmt::Display,
) {
    let param_str = if let Some(params) = parameters {
        let formatted_params: Vec<String> = params
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

        &(if formatted_params.is_empty() {
            String::new()
        } else {
            formatted_params.join(",")
        })
    } else {
        ""
    };

    println!(
        "::{command} {param_str}::{}",
        // Encode value (https://github.com/orgs/community/discussions/26736#discussioncomment-3253165)
        value
            .to_string()
            .replace('%', "%25")
            .replace('\r', "%0D")
            .replace('\n', "%0A")
    );
}
