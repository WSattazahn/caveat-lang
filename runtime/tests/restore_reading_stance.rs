//! Restore refuses a reading whose event, relation or claim no `sample` of
//! its stream could have recorded: a name in a record must have the kind its
//! place needs, and the event a reading names must reach a sample of its
//! stream with that stance. See spec/caveat-save-0.1.md ("Restore
//! validation").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const UMBRELLA: &str = r#"
claim rain_likely;
claim sun_likely;
evidence forecast from "the morning forecast";
readings rain_chance from forecast limit 4;
readings quiet from forecast limit 4;
event read_forecast chance min 0 max 100;
event note chance min 0 max 100;
event leave;
on read_forecast sample rain_chance = chance supports rain_likely;
on note sample quiet = chance;
on leave reject "not yet";
"#;

fn saved() -> Value {
    let mut session = ReactiveSession::from_source(UMBRELLA).unwrap();
    session
        .dispatch_json("read_forecast", r#"{"chance":70}"#)
        .unwrap();
    session.dispatch_json("note", r#"{"chance":30}"#).unwrap();
    let save = session.save_json().unwrap();
    // The untouched save restores.
    ReactiveSession::restore_json(UMBRELLA, &save).unwrap();
    serde_json::from_str(&save).unwrap()
}

fn refused(edit: impl Fn(&mut Value)) -> String {
    let mut save = saved();
    edit(&mut save);
    match ReactiveSession::restore_json(UMBRELLA, &save.to_string()) {
        Ok(_) => panic!("restore accepted an edited reading"),
        Err(error) => error,
    }
}

fn reading<'a>(save: &'a mut Value, stream: &str) -> &'a mut Value {
    &mut save["reading_streams"][stream]["occurrences"][0]
}

#[test]
fn a_reading_supporting_an_undeclared_claim_is_refused() {
    let error = refused(|save| reading(save, "rain_chance")["claim"] = "nowhere".into());
    assert!(error.contains("rain_chance@1"), "{error}");
}

#[test]
fn a_reading_with_a_stance_its_sample_does_not_take_is_refused() {
    refused(|save| reading(save, "rain_chance")["claim"] = "sun_likely".into());
    refused(|save| reading(save, "rain_chance")["relation"] = "opposes".into());
    refused(|save| reading(save, "rain_chance")["relation"] = "qualifies".into());
    refused(|save| reading(save, "rain_chance")["relation"] = "anything".into());
}

#[test]
fn a_reading_naming_an_event_that_samples_no_such_stream_is_refused() {
    let error = refused(|save| reading(save, "rain_chance")["event"] = "undeclared".into());
    assert!(error.contains("rain_chance@1"), "{error}");
    refused(|save| reading(save, "rain_chance")["event"] = "leave".into());
    refused(|save| reading(save, "rain_chance")["event"] = "note".into());
    refused(|save| reading(save, "quiet")["event"] = "read_forecast".into());
}
