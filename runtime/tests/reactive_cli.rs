use serde_json::{json, Value};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "caveat-reactive-cli-{}-{}-{}.cav",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed),
        ));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        file.write_all(source.as_bytes()).unwrap();
        Self(path)
    }
}

impl Drop for SourceFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn thermostat() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/thermostat_history.cav")
        .to_string_lossy()
        .into_owned()
}

fn run(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_caveat-reactive"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn records(output: &Output) -> Vec<Value> {
    String::from_utf8(output.stdout.clone())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn validate_checks_reactive_semantics_and_publishes_the_event_contract() {
    let valid = run(&["validate", &thermostat()], "");
    assert!(valid.status.success());
    let record = &records(&valid)[0];
    assert_eq!(record["kind"], "validated");
    assert_eq!(record["snapshot"]["sequence"], 0);
    assert_eq!(record["snapshot"]["events"][0]["name"], "read");
    let invalid = SourceFile::new("state x = 0; event run; on run set missing = 1;");
    let result = run(&["validate", &invalid.0.to_string_lossy()], "");
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    let error: Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["stage"], "source");
}

#[test]
fn replay_streams_independent_qualified_readings_and_frozen_decisions() {
    let input = "{\"event\":\"read\",\"payload\":{\"value\":17}}\n\n{\"event\":\"read\",\"payload\":{\"value\":25}}\n{\"event\":\"read\",\"payload\":{\"value\":17}}\n";
    let result = run(&["replay", &thermostat(), "-"], input);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let records = records(&result);
    assert_eq!(records.len(), 4);
    assert_eq!(records[0]["kind"], "initial");
    assert_eq!(records[2]["line"], 3);
    assert_eq!(records[3]["snapshot"]["sequence"], 3);
    let snapshot = &records[3]["snapshot"];
    assert_eq!(
        snapshot["reading_streams"]["temperature"]["occurrences"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(snapshot["commitment_bases"]["heating@1"]["value"], 1.0);
    assert_eq!(snapshot["commitment_bases"]["heating@2"]["value"], 0.0);
    assert_eq!(snapshot["commitment_bases"]["heating@3"]["value"], 1.0);
    assert_eq!(
        snapshot["commitment_bases"]["heating@1"],
        records[1]["snapshot"]["commitment_bases"]["heating@1"]
    );
    assert!(
        snapshot["commitment_bases"]["heating@3"]["provenance"]["caveats"]
            .as_array()
            .unwrap()
            .contains(&json!("calibration_offset"))
    );
    assert_eq!(
        run(&["replay", &thermostat(), "-"], input).stdout,
        result.stdout
    );
}

#[test]
fn duplicate_payload_fields_are_rejected_before_a_second_event_is_published() {
    let result = run(&["replay", &thermostat(), "-"],
        "{\"event\":\"read\",\"payload\":{\"value\":17}}\n{\"event\":\"read\",\"payload\":{\"value\":25,\"value\":17}}\n{\"event\":\"read\",\"payload\":{\"value\":25}}\n");
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(records(&result).len(), 2);
    let error: Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["stage"], "dispatch");
    assert_eq!(error["line"], 2);
    assert_eq!(error["sequence"], 1);
}

#[test]
fn malformed_envelopes_have_line_numbers_and_do_not_dispatch() {
    for input in [
        "{\"event\":\"read\",\"payload\":{\"value\":17},\"extra\":0}",
        "{\"event\":\"read\",\"event\":\"read\",\"payload\":{\"value\":17}}",
        "{\"event\":\"read\"}",
        "[]",
        "not json",
    ] {
        let result = run(&["replay", &thermostat(), "-"], input);
        assert!(!result.status.success(), "accepted {input}");
        assert_eq!(records(&result).len(), 1);
        let error: Value = serde_json::from_slice(&result.stderr).unwrap();
        assert_eq!(error["stage"], "input");
        assert_eq!(error["line"], 1);
        assert_eq!(error["sequence"], 0);
    }
}

#[test]
fn late_runtime_failure_emits_no_partial_snapshot() {
    let source = SourceFile::new("claim warm; evidence sensor from \"meter\"; readings history from sensor limit 2; state x = 0; event good; event bad; on good sample history = 1 supports warm; on bad sample history = 2 supports warm; on bad set x = 1 / 0;");
    let result = run(
        &["replay", &source.0.to_string_lossy(), "-"],
        "{\"event\":\"good\",\"payload\":{}}\n{\"event\":\"bad\",\"payload\":{}}\n",
    );
    assert!(!result.status.success());
    let records = records(&result);
    assert_eq!(records.len(), 2);
    assert_eq!(
        records[1]["snapshot"]["reading_streams"]["history"]["current"],
        "history@1"
    );
    let error: Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["sequence"], 1);
    assert_eq!(error["event"], "bad");
}

#[test]
fn usage_and_missing_files_report_machine_readable_errors() {
    let usage = run(&["validate"], "");
    assert_eq!(usage.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&usage.stderr).unwrap()["stage"],
        "usage"
    );
    let missing = run(&["validate", "missing-caveat-reactive-fixture.cav"], "");
    assert_eq!(missing.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&missing.stderr).unwrap()["stage"],
        "source_io"
    );
}

#[test]
fn oversized_source_and_event_lines_fail_with_explicit_transport_limits() {
    let too_large = SourceFile::new(&" ".repeat(1_048_577));
    let source_result = run(&["validate", &too_large.0.to_string_lossy()], "");
    assert!(!source_result.status.success());
    let error: Value = serde_json::from_slice(&source_result.stderr).unwrap();
    assert_eq!(error["stage"], "source_io");
    assert!(error["message"].as_str().unwrap().contains("1 MiB"));

    let event_result = run(
        &["replay", &thermostat(), &too_large.0.to_string_lossy()],
        "",
    );
    assert!(!event_result.status.success());
    assert_eq!(records(&event_result).len(), 1);
    let error: Value = serde_json::from_slice(&event_result.stderr).unwrap();
    assert_eq!(error["stage"], "input");
    assert_eq!(error["sequence"], 0);
    assert!(error["message"].as_str().unwrap().contains("1 MiB"));
}
