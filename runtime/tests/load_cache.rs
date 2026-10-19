//! A program taken from the load cache is the program loaded fresh: the same
//! snapshot, view and save before and after the same events, and events in
//! one session never reach another opened from the same source.
use caveat_runtime::reactive::ReactiveSession;

const SOURCE: &str = r#"
claim ready;
evidence chart from "chart";
caveat age consequence low;
state basis = 0;
event consult;
event decide;
event qualify_later;
on consult reveal chart supports ready;
on consult set basis = qualified(1, chart);
on decide commit act because enough using basis;
on qualify_later qualify chart with age;
"#;

const EVENTS: [&str; 3] = ["consult", "decide", "qualify_later"];

/// Everything a host can read from a session, as text.
fn shown(session: &ReactiveSession) -> [String; 3] {
    [
        serde_json::to_string(&session.snapshot()).unwrap(),
        serde_json::to_string(&session.view()).unwrap(),
        session.save_json().unwrap(),
    ]
}

fn played(mut session: ReactiveSession) -> ReactiveSession {
    for event in EVENTS {
        session.dispatch_json(event, "{}").unwrap();
    }
    session
}

#[test]
fn a_cached_program_is_the_fresh_program() {
    let fresh = ReactiveSession::from_source(SOURCE).unwrap();
    let first = ReactiveSession::from_source_cached(SOURCE).unwrap();
    let second = ReactiveSession::from_source_cached(SOURCE).unwrap();
    assert_eq!(shown(&first), shown(&fresh));
    assert_eq!(shown(&second), shown(&fresh));
    assert_eq!(
        shown(&played(second)),
        shown(&played(ReactiveSession::from_source(SOURCE).unwrap()))
    );
}

#[test]
fn events_in_one_session_do_not_reach_the_cache() {
    let fresh = ReactiveSession::from_source(SOURCE).unwrap();
    let _played = played(ReactiveSession::from_source_cached(SOURCE).unwrap());
    let reopened = ReactiveSession::from_source_cached(SOURCE).unwrap();
    assert_eq!(shown(&reopened), shown(&fresh));
}

#[test]
fn a_restore_through_the_cache_matches_a_restore_from_fresh() {
    let save = played(ReactiveSession::from_source(SOURCE).unwrap())
        .save_json()
        .unwrap();
    let compact = played(ReactiveSession::from_source(SOURCE).unwrap())
        .save_compact_json()
        .unwrap();
    // The first restore loads the program; the second takes it from the cache.
    let once = ReactiveSession::restore_json(SOURCE, &save).unwrap();
    let again = ReactiveSession::restore_json(SOURCE, &save).unwrap();
    let replayed = ReactiveSession::restore_json(SOURCE, &compact).unwrap();
    assert_eq!(shown(&again), shown(&once));
    assert_eq!(shown(&replayed), shown(&once));
}

#[test]
fn a_source_that_does_not_load_fails_the_same_way_each_time() {
    let broken = "state x = ;";
    let fresh = ReactiveSession::from_source(broken).err().unwrap();
    assert_eq!(
        ReactiveSession::from_source_cached(broken).err().unwrap(),
        fresh
    );
    assert_eq!(
        ReactiveSession::from_source_cached(broken).err().unwrap(),
        fresh
    );
}

#[test]
fn a_program_pushed_out_of_the_cache_loads_again() {
    let fresh = ReactiveSession::from_source(SOURCE).unwrap();
    ReactiveSession::from_source_cached(SOURCE).unwrap();
    for n in 0..8 {
        ReactiveSession::from_source_cached(&format!("state other{n} = {n};\nevent poke;"))
            .unwrap();
    }
    let reloaded = ReactiveSession::from_source_cached(SOURCE).unwrap();
    assert_eq!(shown(&reloaded), shown(&fresh));
}
