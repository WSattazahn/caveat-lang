use caveat_runtime::reactive::{ReactiveSession, ReactiveSnapshot};

const SOURCE: &str = include_str!("../../examples/ventilation_controller.cav");

fn read(program: &mut ReactiveSession, value: u8) -> ReactiveSnapshot {
    program
        .dispatch_json("read", &format!(r#"{{"value":{value}}}"#))
        .unwrap()
}

#[test]
fn missing_data_does_not_invent_a_reading_or_a_command() {
    let mut program = ReactiveSession::from_source(SOURCE).unwrap();
    let initial = program.snapshot();
    assert_eq!(initial.bindings["fan"]["value"].as_number(), Some(0.0));
    assert_eq!(initial.bindings["status"]["text"].as_str(), Some("waiting"));
    for _ in 0..2 {
        let missing = program.dispatch_json("unavailable", "{}").unwrap();
        assert!(missing.reading_streams["temperature"]
            .occurrences
            .is_empty());
        assert!(missing.decision_series["ventilation"].revisions.is_empty());
        assert_eq!(missing.bindings["status"]["text"].as_str(), Some("waiting"));
        assert!(missing.relations.iter().any(|edge| edge.from == "outage"
            && edge.to == "comfortable"
            && edge.relation == "opposes"));
    }
    let observed = read(&mut program, 40);
    assert_eq!(observed.bindings["fan"]["value"].as_number(), Some(1.0));
    assert_eq!(
        observed.reading_streams["temperature"].current.as_deref(),
        Some("temperature@1")
    );
}

#[test]
fn outage_reopens_without_changing_the_frozen_command_and_recovery_keeps_its_reasons() {
    let mut program = ReactiveSession::from_source(SOURCE).unwrap();
    let first = read(&mut program, 30);
    let missing = program.dispatch_json("unavailable", "{}").unwrap();
    assert_eq!(missing.bindings["fan"]["value"].as_number(), Some(1.0));
    assert_eq!(
        missing.bindings["status"]["text"].as_str(),
        Some("reopened")
    );
    assert_eq!(
        missing.reading_streams["temperature"].occurrences,
        first.reading_streams["temperature"].occurrences
    );
    assert_eq!(missing.commitment_bases, first.commitment_bases);
    let recovered = read(&mut program, 26);
    assert_eq!(recovered.bindings["fan"]["value"].as_number(), Some(0.0));
    assert_eq!(
        recovered.bindings["status"]["text"].as_str(),
        Some("current")
    );
    assert_eq!(
        recovered.commitment_bases["ventilation@1"],
        first.commitment_bases["ventilation@1"]
    );
    let basis = &recovered.commitment_bases["ventilation@2"].provenance;
    assert!(basis.evidence.contains("outage"));
    assert!(basis.evidence.contains("temperature@2"));
    assert!(basis.caveats.contains("calibration"));
    assert!(basis.caveats.contains("missing_data"));
    assert!(recovered
        .relations
        .iter()
        .any(|edge| edge.from == "temperature@1"
            && edge.to == "comfortable"
            && edge.relation == "opposes"));
    assert!(recovered
        .relations
        .iter()
        .any(|edge| edge.from == "temperature@2"
            && edge.to == "comfortable"
            && edge.relation == "supports"));
}

#[test]
fn all_256_four_event_histories_preserve_observations_and_decision_bases() {
    for mut route in 0..256 {
        let mut program = ReactiveSession::from_source(SOURCE).unwrap();
        let mut previous = program.snapshot();
        let mut readings = 0;
        let mut last_reading = None;
        for _ in 0..4 {
            let event = route % 4;
            route /= 4;
            let snapshot = if event == 3 {
                program.dispatch_json("unavailable", "{}").unwrap()
            } else {
                let value = [0, 26, 40][event];
                readings += 1;
                last_reading = Some(value);
                read(&mut program, value)
            };
            assert_eq!(
                snapshot.reading_streams["temperature"].occurrences.len(),
                readings
            );
            assert_eq!(
                snapshot.decision_series["ventilation"].revisions.len(),
                readings
            );
            let expected_fan = if last_reading.is_some_and(|value| value > 26) {
                1.0
            } else {
                0.0
            };
            assert_eq!(
                snapshot.bindings["fan"]["value"].as_number(),
                Some(expected_fan)
            );
            let status = if readings == 0 {
                "waiting"
            } else if event == 3 {
                "reopened"
            } else {
                "current"
            };
            assert_eq!(snapshot.bindings["status"]["text"].as_str(), Some(status));
            for (id, basis) in &previous.commitment_bases {
                assert_eq!(&snapshot.commitment_bases[id], basis);
            }
            for (index, occurrence) in previous.reading_streams["temperature"]
                .occurrences
                .iter()
                .enumerate()
            {
                assert_eq!(
                    &snapshot.reading_streams["temperature"].occurrences[index],
                    occurrence
                );
            }
            previous = snapshot;
        }
    }
}

#[test]
fn ninth_reading_fails_without_reopening_or_relabeling_the_eighth() {
    let mut program = ReactiveSession::from_source(SOURCE).unwrap();
    for _ in 0..8 {
        read(&mut program, 27);
    }
    let before = program.snapshot();
    assert!(program.dispatch_json("read", r#"{"value":20}"#).is_err());
    assert_eq!(program.snapshot(), before);
}
