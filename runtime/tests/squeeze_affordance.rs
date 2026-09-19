use caveat_runtime::reactive::{ReactiveSession, ReactiveSnapshot};
use caveat_runtime::source_library::{FunctionValue, SourceLibrary};
use serde_json::{json, Value};

const SOURCE: &str = include_str!("../../examples/squeeze_affordance.cav");
const FIXTURES: &str = include_str!("../../examples/squeeze_affordance_fixtures.json");
const REPLAY: &str = include_str!("../../examples/squeeze_affordance_replay.jsonl");

fn fixtures() -> Value {
    serde_json::from_str(FIXTURES).unwrap()
}

fn dispatch(session: &mut ReactiveSession, event: &str, payload: &Value) -> ReactiveSnapshot {
    session.dispatch_json(event, &payload.to_string()).unwrap()
}

fn probe(x: f64, y: f64, z: f64, to_x: f64, to_z: f64) -> Value {
    json!({"x":x,"y":y,"z":z,"to_x":to_x,"to_z":to_z})
}

fn code(snapshot: &ReactiveSnapshot) -> f64 {
    snapshot.bindings["decision"]["value"].as_number().unwrap()
}

fn replay(source: &str) -> Vec<ReactiveSnapshot> {
    let mut session = ReactiveSession::from_source(source).unwrap();
    REPLAY
        .lines()
        .map(|line| {
            let event: Value = serde_json::from_str(line).unwrap();
            dispatch(
                &mut session,
                event["event"].as_str().unwrap(),
                &event["payload"],
            )
        })
        .collect()
}

#[test]
fn fixture_payloads_preserve_the_authored_objects_and_declared_derivation() {
    let fixtures = fixtures();
    assert_eq!(fixtures["source"]["path"], "public/scenes/SlimeWorld1.json");
    assert_eq!(fixtures["source"]["sha256"].as_str().unwrap().len(), 64);
    for fixture in fixtures["fixtures"].as_array().unwrap() {
        let object = &fixture["object"];
        let payload = &fixture["passage_payload"];
        assert_eq!(object["assetId"], "primitives/cube");
        assert_eq!(object["role"], "squeeze");
        assert_eq!(object["physics"]["type"], "fixed");
        for (index, axis) in ["x", "y", "z"].iter().enumerate() {
            assert_eq!(payload[format!("c{axis}")], object["position"][index]);
            assert_eq!(payload[format!("r{axis}")], object["rotation"][index]);
            let actual = payload[format!("h{axis}")].as_f64().unwrap();
            let expected = object["scale"][index].as_f64().unwrap().abs() * 0.5;
            assert!((actual - expected).abs() <= f64::EPSILON * 4.0);
        }
    }
    assert_eq!(
        fixtures["fixtures"][0]["object"]["name"],
        "Cave Squeeze Box"
    );
    assert_eq!(
        fixtures["fixtures"][1]["object"]["rotation"][1],
        32.477603615619394
    );
    let events: Vec<Value> = REPLAY
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        events[0]["payload"],
        fixtures["fixtures"][0]["passage_payload"]
    );
    assert_eq!(
        events[5]["payload"],
        fixtures["fixtures"][1]["passage_payload"]
    );
}

#[test]
fn replay_explains_entry_retreat_and_unsupported_rotation_without_erasing_decisions() {
    let snapshots = replay(SOURCE);
    assert_eq!(
        snapshots.iter().map(code).collect::<Vec<_>>(),
        [-2.0, 0.0, 1.0, 2.0, 0.0, -2.0, -1.0]
    );
    assert_eq!(
        snapshots,
        replay(SOURCE),
        "Identical source and input must replay identically"
    );
    for pair in snapshots.windows(2) {
        for (name, basis) in &pair[0].commitment_bases {
            assert_eq!(&pair[1].commitment_bases[name], basis);
        }
        for (name, stream) in &pair[0].reading_streams {
            assert_eq!(
                &pair[1].reading_streams[name].occurrences[..stream.occurrences.len()],
                stream.occurrences
            );
        }
    }
    let last = snapshots.last().unwrap();
    assert_eq!(last.decision_series["traversal"].revisions.len(), 5);
    for basis in last.commitment_bases.values() {
        for caveat in ["authored_volume", "snapshot_age", "synthetic_intent"] {
            assert!(
                basis.provenance.caveats.contains(caveat),
                "Missing {caveat}"
            );
        }
        for evidence in ["half_x@1", "half_y@1", "half_z@1"] {
            assert!(basis.provenance.evidence.contains(evidence));
        }
    }
    assert!(last.bindings["reason"]["text"]
        .as_str()
        .unwrap()
        .starts_with("Unsupported rotation:"));
    assert!(last.bindings["qualification"]["text"]
        .as_str()
        .unwrap()
        .contains("No verified collision safety"));
    assert!(snapshots[5].bindings["reason"]["text"]
        .as_str()
        .unwrap()
        .contains("previous decision remains recorded and reopened"));
    let archive = serde_json::to_value(last).unwrap();
    assert!(archive["symbols"]
        .as_array()
        .unwrap()
        .iter()
        .any(|symbol| symbol["name"] == "half_x@1"
            && symbol["source"]
                .as_str()
                .unwrap_or("")
                .contains("squeeze_affordance_fixtures.json")));
}

#[test]
fn missing_passage_and_rejected_data_do_not_fabricate_or_replace_a_decision() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    let missing = dispatch(&mut session, "probe", &probe(0.0, 0.0, 0.0, 1.0, 0.0));
    assert_eq!(code(&missing), -2.0);
    assert!(missing.commitment_bases.is_empty());
    assert!(missing
        .reading_streams
        .values()
        .all(|stream| stream.occurrences.is_empty()));
    let payload = fixtures()["fixtures"][0]["passage_payload"].clone();
    dispatch(&mut session, "passage", &payload);
    dispatch(&mut session, "probe", &probe(0.0, 0.0, 0.0, 0.0, 0.0));
    let before = session.snapshot();
    for field in ["hx", "hy", "hz", "kind"] {
        let mut invalid = payload.clone();
        invalid[field] = if field == "kind" {
            json!(1.5)
        } else {
            json!(-1)
        };
        assert!(session
            .dispatch_json("passage", &invalid.to_string())
            .is_err());
        assert_eq!(
            session.snapshot(),
            before,
            "Rejected {field} changed the record"
        );
    }
    // NaN/infinities are invalid JSON numbers; JSON.stringify encodes them as
    // null, which must also fail instead of becoming an apparently real zero.
    for token in ["NaN", "Infinity", "-Infinity", "1e400", "null", "\"NaN\""] {
        let invalid = format!(r#"{{"x":{token},"y":0,"z":0,"to_x":0,"to_z":0}}"#);
        assert!(session.dispatch_json("probe", &invalid).is_err());
        assert_eq!(
            session.snapshot(),
            before,
            "Rejected {token} changed the record"
        );
    }
    assert!(session
        .dispatch_json(
            "probe",
            r#"{"x":0,"y":0,"z":0,"to_x":0,"to_z":0,"extra":1}"#
        )
        .is_err());
    assert_eq!(session.snapshot(), before);
}

#[test]
fn source_policy_revision_changes_entry_tolerance_but_preserves_reported_facts() {
    let changed = SOURCE.replace("fn entry_padding() = 0.22;", "fn entry_padding() = 0;");
    assert_ne!(changed, SOURCE);
    let payload = &fixtures()["fixtures"][0]["passage_payload"];
    let cx = payload["cx"].as_f64().unwrap();
    let cy = payload["cy"].as_f64().unwrap();
    let cz = payload["cz"].as_f64().unwrap();
    let x = cx + payload["hx"].as_f64().unwrap() + 0.45;
    let mut original = ReactiveSession::from_source(SOURCE).unwrap();
    let mut revised = ReactiveSession::from_source(&changed).unwrap();
    dispatch(&mut original, "passage", payload);
    dispatch(&mut revised, "passage", payload);
    let first = dispatch(&mut original, "probe", &probe(x, cy, cz, x, cz));
    let second = dispatch(&mut revised, "probe", &probe(x, cy, cz, x, cz));
    assert_eq!(code(&first), 1.0);
    assert_eq!(code(&second), 0.0);
    assert_eq!(first.reading_streams, second.reading_streams);
    assert_eq!(
        first.commitment_bases["traversal@1"].provenance,
        second.commitment_bases["traversal@1"].provenance
    );
}

#[test]
fn independent_boundary_cases_cover_contact_retreat_height_and_unsupported_data() {
    let library = SourceLibrary::from_source(SOURCE).unwrap();
    let contact = 1.0 + (0.35 + 0.22);
    let assess = |profile, rotation, x, y, z, tx, tz, hx| {
        library
            .call_numbers(
                "squeeze_assessment",
                &[
                    profile, rotation, x, y, z, tx, tz, 0.0, 0.0, 0.0, hx, 1.0, 1.0,
                ],
            )
            .unwrap()
            .value
    };
    for (x, y, z, tx, tz, expected) in [
        (1.0, 0.0, 0.0, 1.0, 0.0, 2.0),
        (contact, 0.0, 0.0, contact, 0.0, 1.0),
        (contact + 0.000001, 0.0, 0.0, 3.0, 0.0, 0.0),
        (2.0, 0.0, 0.0, 1.2, 0.0, 1.0),
        (2.0, 0.0, 0.0, 3.0, 0.0, 0.0),
        (2.0, contact, 0.0, 1.2, 0.0, 1.0),
        (2.0, contact + 0.000001, 0.0, 1.2, 0.0, 0.0),
        (2.0, 0.0, 2.0, 2.0, -2.0, 0.0),
    ] {
        assert_eq!(
            assess(1.0, 0.0, x, y, z, tx, tz, 1.0),
            FunctionValue::Number(expected)
        );
    }
    for (profile, rotation, hx) in [
        (0.0, 0.0, 1.0),
        (0.5, 0.0, 1.0),
        (1.0, 32.477603615619394, 1.0),
        (1.0, -32.477603615619394, 1.0),
        (1.0, 0.0, -1.0),
        (1.0, 0.0, 0.0),
    ] {
        assert_eq!(
            assess(profile, rotation, 0.0, 0.0, 0.0, 0.0, 0.0, hx),
            FunctionValue::Number(-1.0)
        );
    }
    assert!(matches!(
        library.call_numbers("squeeze_reason", &[-1.0, 1.0, -32.477603615619394, 1.0, 1.0, 1.0]).unwrap().value,
        FunctionValue::Text(reason) if reason.starts_with("Unsupported rotation:")
    ));
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(library
            .call_numbers(
                "squeeze_assessment",
                &[1.0, 0.0, invalid, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]
            )
            .is_err());
    }
}

// An independent slab implementation checks the source's separation-axis
// formula, including zero-length segments, parallel paths, edges, and corners.
fn slab_contact(from: [f64; 2], to: [f64; 2]) -> bool {
    let mut first: f64 = 0.0;
    let mut last: f64 = 1.0;
    for axis in 0..2 {
        let delta = to[axis] - from[axis];
        if delta == 0.0 {
            if from[axis].abs() > 1.0 {
                return false;
            }
        } else {
            let a = (-1.0 - from[axis]) / delta;
            let b = (1.0 - from[axis]) / delta;
            first = first.max(a.min(b));
            last = last.min(a.max(b));
            if first > last {
                return false;
            }
        }
    }
    true
}

#[test]
fn source_segment_geometry_matches_an_independent_oracle_on_625_paths() {
    let library = SourceLibrary::from_source(SOURCE).unwrap();
    for x in [-2.0, -1.0, 0.0, 1.0, 2.0] {
        for z in [-2.0, -1.0, 0.0, 1.0, 2.0] {
            for tx in [-2.0, -1.0, 0.0, 1.0, 2.0] {
                for tz in [-2.0, -1.0, 0.0, 1.0, 2.0] {
                    let result = library
                        .call_numbers("segment_contacts", &[x, z, tx, tz, 0.0, 0.0, 1.0, 1.0])
                        .unwrap();
                    assert_eq!(
                        result.value,
                        FunctionValue::Bool(slab_contact([x, z], [tx, tz])),
                        "[{x}, {z}] -> [{tx}, {tz}]"
                    );
                }
            }
        }
    }
}

#[test]
fn late_decision_capacity_failure_rolls_back_new_probe_readings() {
    let bounded = SOURCE.replace(
        "decisions traversal limit 16",
        "decisions traversal limit 1",
    );
    let mut session = ReactiveSession::from_source(&bounded).unwrap();
    dispatch(
        &mut session,
        "passage",
        &fixtures()["fixtures"][0]["passage_payload"],
    );
    dispatch(&mut session, "probe", &probe(0.0, 0.0, 0.0, 0.0, 0.0));
    let before = session.snapshot();
    assert!(session
        .dispatch_json("probe", &probe(1.0, 0.0, 0.0, 1.0, 0.0).to_string())
        .is_err());
    assert_eq!(session.snapshot(), before);
}
