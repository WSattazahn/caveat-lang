use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

const SOURCE: &str = include_str!("../../examples/thermostat_history.cav");

fn read(session: &mut ReactiveSession, value: f64) -> Value {
    serde_json::to_value(
        session
            .dispatch_json("read", &json!({"value": value}).to_string())
            .expect("a valid reading should revise the heating decision"),
    )
    .unwrap()
}

#[test]
fn thermostat_revises_opposing_readings_without_erasing_its_reasons() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    let initial = serde_json::to_value(session.snapshot()).unwrap();
    assert_eq!(initial["bindings"]["temperature"]["text"], "No reading");
    assert_eq!(
        initial["reading_streams"]["temperature"]["occurrences"],
        json!([])
    );

    let first = read(&mut session, 17.0);
    let first_basis = first["commitment_bases"]["heating@1"].clone();
    let first_reading = first["reading_streams"]["temperature"]["occurrences"][0].clone();
    assert_eq!(first_basis["value"], 1.0);
    assert_eq!(first["bindings"]["heating"]["text"], "100%");
    assert!(first_basis["provenance"]["evidence"]
        .as_array()
        .unwrap()
        .contains(&json!("temperature@1")));
    assert!(first_basis["provenance"]["caveats"]
        .as_array()
        .unwrap()
        .contains(&json!("calibration_offset")));

    let second = read(&mut session, 25.0);
    let second_basis = second["commitment_bases"]["heating@2"].clone();
    assert_eq!(second_basis["value"], 0.0);
    assert_eq!(second["bindings"]["heating"]["text"], "0%");
    assert_eq!(second["commitment_bases"]["heating@1"], first_basis);

    let third = read(&mut session, 17.0);
    assert_eq!(third["bindings"]["temperature"]["text"], "17 C");
    assert_eq!(third["bindings"]["heating"]["text"], "100%");
    assert_eq!(third["commitment_bases"]["heating@1"], first_basis);
    assert_eq!(third["commitment_bases"]["heating@2"], second_basis);
    assert_eq!(
        third["reading_streams"]["temperature"]["occurrences"][0],
        first_reading
    );
    assert_eq!(
        third["reading_streams"]["temperature"]["current"],
        "temperature@3"
    );
    assert_eq!(third["decision_series"]["heating"]["current"], "heating@3");
    assert_eq!(
        third["decision_series"]["heating"]["revisions"][2]["previous"],
        "heating@2"
    );
    for (id, relation) in [
        ("temperature@1", "opposes"),
        ("temperature@2", "supports"),
        ("temperature@3", "opposes"),
    ] {
        assert!(third["relations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| edge["from"] == id
                && edge["relation"] == relation
                && edge["to"] == "warm_enough"));
    }
    assert!(!third["relations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|edge| edge["from"] == "thermistor"
            && (edge["relation"] == "supports" || edge["relation"] == "opposes")));

    let mut replay = ReactiveSession::from_source(SOURCE).unwrap();
    read(&mut replay, 17.0);
    read(&mut replay, 25.0);
    assert_eq!(read(&mut replay, 17.0), third);
}

#[test]
fn rejected_heating_revision_cannot_publish_a_phantom_sensor_reading() {
    let bounded = SOURCE.replace("decisions heating limit 8", "decisions heating limit 2");
    let mut session = ReactiveSession::from_source(&bounded).unwrap();
    read(&mut session, 17.0);
    read(&mut session, 25.0);
    let before = session.snapshot();
    assert!(session.dispatch_json("read", r#"{"value":17}"#).is_err());
    assert_eq!(session.snapshot(), before);
}
