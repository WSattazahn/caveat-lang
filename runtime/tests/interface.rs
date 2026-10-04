//! A program's interface (spec/caveat-interface-0.1.md): what it contains,
//! that whole interfaces are byte-stable, and that a program that does not
//! load gets its load error. scripts/test-interface.mjs compares this build
//! with the WASM build for every repository program.
//!
//! After an intended change, regenerate the fixtures with
//! `CAVEAT_UPDATE_INTERFACE=1 cargo test --test interface`.
use caveat_runtime::reactive::{interface_source, ReactiveSession};
use caveat_runtime::web::WebReactiveSession;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/interface")
}

fn updating() -> bool {
    std::env::var_os("CAVEAT_UPDATE_INTERFACE").is_some()
}

/// What the WASM export returns, from the native build of the same code.
fn interface_text(source: &str) -> Result<String, String> {
    WebReactiveSession::interface(source)
}

const EVERY_FIELD: &str = r#"claim frost_risk;
evidence probe from "a soil probe";
evidence grant from "the grower";
caveat uncalibrated consequence material;
uncalibrated qualifies probe;
readings soil from probe limit 12;
decisions cover limit 4 reopened by soil opposing frost_risk;
identifiers limit 8;
place yard kind yard;
entity north kind bed at yard;
entity south kind bed at yard;
state level = 0 min -40 max 60;
state count = 0;
event read celsius min -40 max 60;
event pick bed kind bed, size in small large, who id;
cue ping sound 440 0.2 0.5;
on read sample soil = celsius supports frost_risk;
on read set level = celsius;
on pick set count = count + 1;
bind gauge.value = level;
bind gauge.label = "cold" when level < 0;
bind gauge.label = "warm" when level >= 0;
bind gauge.alarm = level < -10 when has_sample(soil);
"#;

#[test]
fn every_field_of_a_small_program() {
    let text = interface_text(EVERY_FIELD).unwrap();
    let expected = r#"{
  "schema": "caveat-interface/0.1",
  "events": [
    {
      "name": "pick",
      "parameters": [
        {
          "name": "bed",
          "type": "entity",
          "kind": "bed",
          "members": [
            "north",
            "south"
          ]
        },
        {
          "name": "size",
          "type": "member",
          "members": [
            "small",
            "large"
          ]
        },
        {
          "name": "who",
          "type": "id"
        }
      ]
    },
    {
      "name": "read",
      "parameters": [
        {
          "name": "celsius",
          "type": "number",
          "min": -40.0,
          "max": 60.0
        }
      ]
    }
  ],
  "states": [
    {
      "name": "count",
      "min": -1000000000000.0,
      "max": 1000000000000.0
    },
    {
      "name": "level",
      "min": -40.0,
      "max": 60.0
    }
  ],
  "bindings": [
    {
      "target": "gauge",
      "property": "alarm",
      "type": "boolean",
      "always": false
    },
    {
      "target": "gauge",
      "property": "label",
      "type": "text",
      "always": false
    },
    {
      "target": "gauge",
      "property": "value",
      "type": "number",
      "always": true
    }
  ],
  "cues": [
    {
      "name": "ping",
      "kind": "sound"
    }
  ],
  "decisions": [
    {
      "name": "cover",
      "limit": 4
    }
  ],
  "readings": [
    {
      "name": "soil",
      "evidence": "probe",
      "limit": 12
    }
  ],
  "evidence": [
    "grant",
    "probe"
  ],
  "caveats": [
    "uncalibrated"
  ],
  "claims": [
    "frost_risk"
  ]
}"#;
    assert_eq!(text, expected);
}

#[test]
fn the_interface_does_not_change_with_events_or_restore() {
    let mut session = ReactiveSession::from_source(EVERY_FIELD).unwrap();
    let before = session.interface();
    session
        .dispatch_json("read", r#"{"celsius": -20}"#)
        .unwrap();
    session
        .dispatch_json("pick", r#"{"bed": "south", "size": "large", "who": "ana"}"#)
        .unwrap();
    assert_eq!(session.interface(), before);
    assert_eq!(interface_source(EVERY_FIELD).unwrap(), before);
    let saved = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(EVERY_FIELD, &saved).unwrap();
    assert_eq!(restored.interface(), before);
}

#[test]
fn a_program_that_does_not_load_gets_its_load_error() {
    for source in [
        "state level = 0 min 0 max 10;\n",
        "event poke;\nbind gauge.value = missing;\n",
        "event poke;\nbind gauge.value = 1;\nbind gauge.value = \"one\";\n",
    ] {
        let load = ReactiveSession::from_source(source).unwrap_err();
        assert_eq!(interface_text(source), Err(load.clone()), "{source}");
        assert_eq!(interface_source(source), Err(load), "{source}");
    }
}

/// Two round 7 programs whose adapters the kit's declarations are measured
/// against, and one with identifiers, as full text.
#[test]
fn whole_interfaces_are_byte_stable() {
    for (program, fixture) in [
        (
            "experiments/glowcap/round7/runs/C1/impl/glowcap.cav",
            "glowcap-c1.json",
        ),
        (
            "experiments/agent-ledger/ledger-identifiers.cav",
            "ledger-identifiers.json",
        ),
    ] {
        let source = std::fs::read_to_string(repo().join(program)).unwrap();
        let text = interface_text(&source).unwrap() + "\n";
        let path = fixtures().join(fixture);
        if updating() {
            std::fs::write(&path, &text).unwrap();
        }
        assert_eq!(text, std::fs::read_to_string(&path).unwrap(), "{program}");
    }
}
