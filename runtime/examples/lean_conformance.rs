//! Production-runtime observations for the bounded Lean comparison harness.
//! Reads one trace document and writes one result document. This is an adapter,
//! not an evaluator: source loading, dispatch and restore use the public runtime.
use caveat_runtime::reactive::{DispatchResult, ReactiveSession};
use serde::{Deserialize, Serialize};
use serde_json::value::{to_raw_value, RawValue};
use std::io::{self, Read, Write};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Trace {
    schema: String,
    source: String,
    events: Vec<String>,
}

#[derive(Serialize)]
struct Observation {
    // Keep the runtime's numeric text, including signed zero, intact.
    snapshot: Box<RawValue>,
    save: String,
    view: Box<RawValue>,
}

#[derive(Serialize)]
struct Event {
    outcome: Box<RawValue>,
    #[serde(flatten)]
    observation: Option<Observation>,
}

#[derive(Serialize)]
struct Output {
    initial: Observation,
    events: Vec<Event>,
}

fn observe(session: &ReactiveSession) -> Result<Observation, String> {
    Ok(Observation {
        snapshot: to_raw_value(&session.snapshot()).map_err(|error| error.to_string())?,
        save: session.save_json()?,
        view: to_raw_value(&session.view()).map_err(|error| error.to_string())?,
    })
}

fn same(expected: &Observation, actual: &Observation, context: &str) -> Result<(), String> {
    for (field, left, right) in [
        ("snapshot", expected.snapshot.get(), actual.snapshot.get()),
        ("save", expected.save.as_str(), actual.save.as_str()),
        ("view", expected.view.get(), actual.view.get()),
    ] {
        if left != right {
            return Err(format!("{context}: {field} differs"));
        }
    }
    Ok(())
}

fn restored(
    source: &str,
    expected: &Observation,
    context: &str,
) -> Result<ReactiveSession, String> {
    let session = ReactiveSession::restore_json(source, &expected.save)
        .map_err(|error| format!("{context}: {error}"))?;
    same(expected, &observe(&session)?, context)?;
    Ok(session)
}

struct Dispatched {
    raw: Box<RawValue>,
    fatal: bool,
    rejected: bool,
}

fn dispatch(session: &mut ReactiveSession, event: &str) -> Result<Dispatched, String> {
    match session.dispatch_outcome_json(event, "{}") {
        Ok(outcome) => Ok(Dispatched {
            rejected: matches!(outcome.result, DispatchResult::Rejected { .. }),
            raw: to_raw_value(&outcome).map_err(|error| error.to_string())?,
            fatal: false,
        }),
        Err(fatal) => Ok(Dispatched {
            raw: to_raw_value(&fatal).map_err(|error| error.to_string())?,
            fatal: true,
            rejected: false,
        }),
    }
}

fn run(input: &str) -> Result<Output, String> {
    let trace: Trace = serde_json::from_str(input).map_err(|error| format!("input: {error}"))?;
    if trace.schema != "caveat-native-trace/0.1" {
        return Err(format!("unsupported input schema: {}", trace.schema));
    }
    let mut original =
        ReactiveSession::from_source(&trace.source).map_err(|error| format!("source: {error}"))?;
    let initial = observe(&original)?;
    let mut resumed = restored(&trace.source, &initial, "initial restore")?;
    let mut events = Vec::new();
    for (index, event) in trace.events.iter().enumerate() {
        let context = format!("event {} ({event})", index + 1);
        let before = observe(&original)?;
        let resumed_before = observe(&resumed)?;
        same(
            &before,
            &resumed_before,
            &format!("{context} before resume"),
        )?;
        let result = dispatch(&mut original, event)?;
        let twin = dispatch(&mut resumed, event)?;
        if result.raw.get() != twin.raw.get() {
            return Err(format!("{context}: original/restored outcome differs"));
        }
        if result.fatal || twin.fatal {
            // Never inspect or resume either session after a fatal result.
            events.push(Event {
                outcome: result.raw,
                observation: None,
            });
            break;
        }
        let after = observe(&original)?;
        let resumed_after = observe(&resumed)?;
        same(&after, &resumed_after, &format!("{context} after resume"))?;
        if result.rejected {
            same(&before, &after, &format!("{context} rejected original"))?;
            same(
                &resumed_before,
                &resumed_after,
                &format!("{context} rejected restored"),
            )?;
        }
        // Each prefix supplies a fresh, genuine save; its restored twin will
        // consume the following event alongside the original session.
        resumed = restored(&trace.source, &after, &format!("{context} restore"))?;
        events.push(Event {
            outcome: result.raw,
            observation: Some(after),
        });
    }
    Ok(Output { initial, events })
}

fn main() {
    let result = (|| -> Result<(), String> {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|error| format!("stdin: {error}"))?;
        let output = run(&input)?;
        let mut encoded = serde_json::to_vec(&output).map_err(|error| error.to_string())?;
        encoded.push(b'\n');
        io::stdout()
            .lock()
            .write_all(&encoded)
            .map_err(|error| format!("stdout: {error}"))
    })();
    if let Err(error) = result {
        eprintln!("lean_conformance: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn input(source: &str, events: &[&str]) -> String {
        json!({"schema": "caveat-native-trace/0.1", "source": source, "events": events}).to_string()
    }

    #[test]
    fn resumes_every_prefix_and_preserves_full_rejected_state() {
        let source = "state n = 0; event advance; event refuse; \
            on advance set n = n + 1; on refuse set n = 99; \
            on refuse reject \"no\"; bind counter.value = n;";
        let output = run(&input(source, &["advance", "refuse", "advance"])).unwrap();
        assert_eq!(output.events.len(), 3);
        let first = output.events[0].observation.as_ref().unwrap();
        let refused = output.events[1].observation.as_ref().unwrap();
        same(first, refused, "test rejection").unwrap();
        assert_ne!(first.save, output.initial.save);
        assert_ne!(
            first.save,
            output.events[2].observation.as_ref().unwrap().save
        );
        let raw: Value = serde_json::from_str(output.events[1].outcome.get()).unwrap();
        assert_eq!(raw["outcome"], "rejected");
        assert_eq!(raw["origin"], "policy");
        assert_eq!(raw["code"], "reject");
    }

    #[test]
    fn fatal_has_only_raw_outcome_and_stops_the_trace() {
        let source = "state n = 0; event crash; event later; \
            on crash set n = 1; bind hud.label = id_text(n + 5) when n > 0; \
            on later set n = 2;";
        let output = run(&input(source, &["crash", "later"])).unwrap();
        let raw = serde_json::to_value(output).unwrap();
        assert_eq!(raw["events"].as_array().unwrap().len(), 1);
        let event = raw["events"][0].as_object().unwrap();
        assert_eq!(event.len(), 1);
        assert_eq!(event["outcome"]["outcome"], "fatal");
        assert_eq!(event["outcome"]["code"], "unclassified");
    }

    #[test]
    fn invalid_input_and_source_are_infrastructure_errors() {
        for text in [
            "{}",
            r#"{"schema":"other","source":"event e;","events":[]}"#,
            r#"{"schema":"caveat-native-trace/0.1","source":"event e;","events":[],"extra":1}"#,
            r#"{"schema":"caveat-native-trace/0.1","source":"event e;","events":[]} {}"#,
            r#"{"schema":"caveat-native-trace/0.1","source":"invalid syntax","events":[]}"#,
        ] {
            assert!(run(text).is_err(), "{text}");
        }
    }

    #[test]
    fn observation_comparison_preserves_numeric_text() {
        let observation = |number: &str| Observation {
            snapshot: RawValue::from_string(format!("{{\"n\":{number}}}")).unwrap(),
            save: "unchanged".into(),
            view: RawValue::from_string("{}".into()).unwrap(),
        };
        assert!(same(&observation("0.0"), &observation("-0.0"), "signed zero").is_err());
    }
}
