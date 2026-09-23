//! JSONL transport for the existing reactive interpreter. No application policy
//! or alternate evaluator lives in this command-line adapter.
use caveat_runtime::reactive::ReactiveSession;
use serde::Deserialize;
use serde_json::{json, value::RawValue, Value};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::{env, process};

const MAX_INPUT_BYTES: usize = 1_048_576;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EventInput {
    event: String,
    // Keeping the original JSON prevents duplicate payload keys from being
    // silently collapsed before the interpreter validates the numeric map.
    payload: Box<RawValue>,
}

fn failure(stage: &str, message: impl ToString) -> Value {
    json!({"kind": "error", "stage": stage, "message": message.to_string()})
}

fn write_record(output: &mut impl Write, record: &Value) -> Result<(), Value> {
    serde_json::to_writer(&mut *output, record).map_err(|error| failure("output", error))?;
    writeln!(output).map_err(|error| failure("output", error))?;
    output.flush().map_err(|error| failure("output", error))
}

fn load_source(path: &str) -> Result<String, Value> {
    // The limit applies to the whole bundle, so imports cannot be used to get
    // past it one module at a time.
    let source = caveat_runtime::link::disk::load(std::path::Path::new(path))
        .map_err(|error| failure("source_io", error))?;
    if source.len() > MAX_INPUT_BYTES {
        return Err(failure("source_io", "source exceeds 1 MiB input limit"));
    }
    Ok(source)
}

fn replay(
    mut input: impl BufRead,
    session: &mut ReactiveSession,
    output: &mut impl Write,
) -> Result<(), Value> {
    write_record(
        output,
        &json!({"kind": "initial", "snapshot": session.snapshot()}),
    )?;
    let mut line_number = 0_u64;
    loop {
        let mut bytes = Vec::new();
        let count = input
            .by_ref()
            .take((MAX_INPUT_BYTES + 1) as u64)
            .read_until(b'\n', &mut bytes)
            .map_err(|error| failure("input_io", error))?;
        if count == 0 {
            return Ok(());
        }
        line_number += 1;
        let input_error = |message: String| {
            let mut error = failure("input", message);
            error["line"] = json!(line_number);
            error["sequence"] = json!(session.snapshot().sequence);
            error
        };
        if count > MAX_INPUT_BYTES {
            return Err(input_error("event line exceeds 1 MiB input limit".into()));
        }
        if bytes.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let event: EventInput =
            serde_json::from_slice(&bytes).map_err(|error| input_error(error.to_string()))?;
        let snapshot = session
            .dispatch_json(&event.event, event.payload.get())
            .map_err(|message| {
                json!({
                    "kind": "error", "stage": "dispatch", "message": message,
                    "line": line_number, "event": event.event,
                    "sequence": session.snapshot().sequence,
                })
            })?;
        write_record(
            output,
            &json!({
                "kind": "event", "line": line_number,
                "event": event.event, "snapshot": snapshot,
            }),
        )?;
    }
}

fn run(args: &[String]) -> Result<(), Value> {
    let usage = || {
        failure("usage", "usage: caveat-reactive validate FILE.cav | caveat-reactive replay FILE.cav EVENTS.jsonl|- (JSON output; 1 MiB source/line limit)")
    };
    let (command, path, input_path) = match args {
        [command, path] if command == "validate" => (command, path, None),
        [command, path, input] if command == "replay" => (command, path, Some(input)),
        _ => return Err(usage()),
    };
    let source = load_source(path)?;
    let mut session =
        ReactiveSession::from_source(&source).map_err(|error| failure("source", error))?;
    let stdout = io::stdout();
    let mut output = stdout.lock();
    if command == "validate" {
        return write_record(
            &mut output,
            &json!({"kind": "validated", "snapshot": session.snapshot()}),
        );
    }
    let input_path = input_path.expect("validated replay arguments");
    if input_path == "-" {
        replay(io::stdin().lock(), &mut session, &mut output)
    } else {
        let input = File::open(input_path).map_err(|error| failure("input_io", error))?;
        replay(BufReader::new(input), &mut session, &mut output)
    }
}

fn main() {
    if let Err(error) = run(&env::args().skip(1).collect::<Vec<_>>()) {
        let code = if error["stage"] == "usage" { 2 } else { 1 };
        eprintln!("{error}");
        process::exit(code);
    }
}
