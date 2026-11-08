//! D070: `restore_json` refused a state save longer than 16 MiB with
//! "cannot restore save: larger than 16777216 bytes", though nothing bounds
//! the save a session writes, so the runtime wrote saves its own restore
//! refused. Restore now takes a state save of any length; a host that
//! restores saves from a party it does not trust bounds their size itself, as
//! the MCP server does at 2 MiB. A replay save keeps its 4,096-byte bound,
//! since the runtime never writes a longer one. See spec/caveat-save-0.1.md
//! ("Changes").
//!
//! No finite test catches every fixed limit. These restore a save a session
//! wrote past 64 MiB, four times the old limit, so any limit below that on the
//! save's length, its content or the size it parses to makes them fail; and a
//! save padded with whitespace to 64 MiB, so a limit on the raw text that
//! ignores content does too.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const OLD_LIMIT: usize = 16 * 1024 * 1024;
const PAST: usize = 4 * OLD_LIMIT;

/// Fill states, each saved with its lineage.
const STATES: usize = 1_050;

/// One caveat with a 65,000-byte name. A state's text value cannot carry the
/// bytes (a state holds a number), but its lineage can: a lineage holds up to
/// 65,536 bytes of names, so `e` and this caveat, 65,001 bytes, fit in one,
/// and 1,050 states holding it pass 64 MiB. A name has no length limit of its
/// own.
fn program() -> String {
    let doubt = format!("doubt_{}", "x".repeat(65_000 - "doubt_".len()));
    let mut source = format!(
        "claim ok;\n\
         evidence e from \"src\";\n\
         caveat {doubt} consequence low;\n\
         {doubt} qualifies e;\n\
         state t = 0;\n\
         event see;\n\
         event fill;\n\
         event bump;\n\
         on see reveal e supports ok;\n\
         on bump set t = t + 1;\n"
    );
    for k in 1..=STATES {
        source += &format!("state s{k} = 0;\non fill set s{k} = qualified({k}, e);\n");
    }
    source
}

fn send(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    outcome
}

/// The save is the runtime's own: `save_json` after two accepted events of a
/// loaded program. Nothing here builds or edits save JSON.
#[test]
fn d070_a_save_past_64_mib_that_a_session_writes_restores() {
    let source = program();
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "see");
    send(&mut live, "fill");
    let saved = live.save_json().unwrap();
    assert!(
        saved.len() > PAST,
        "the save is {} bytes, not past 64 MiB",
        saved.len()
    );
    let lineage = &serde_json::from_str::<Value>(&saved).unwrap()["states"]["s1"]["lineage"];
    assert_eq!(
        lineage["evidence"],
        serde_json::json!(["e"]),
        "{saved:.200}"
    );
    assert_eq!(lineage["caveats"][0].as_str().unwrap().len(), 65_000);

    let mut restored = ReactiveSession::restore_json(&source, &saved)
        .unwrap_or_else(|error| panic!("{} bytes: {error}", saved.len()));
    assert!(
        restored.save_json().unwrap() == saved,
        "the restored session saves other bytes"
    );

    // The next event gives the same outcome on both, and the same save after.
    let next = send(&mut live, "bump");
    assert!(
        send(&mut restored, "bump") == next,
        "bump answered otherwise"
    );
    let after = live.save_json().unwrap();
    assert!(after.len() > PAST);
    assert!(
        restored.save_json().unwrap() == after,
        "the saves after bump differ"
    );
}

/// A small save the runtime wrote, padded with whitespace to 64 MiB, is still
/// valid JSON for the same save, and restores the same session.
#[test]
fn d070_a_save_padded_to_64_mib_restores_the_same_session() {
    const SOURCE: &str = "claim ok;\n\
        evidence e from \"src\";\n\
        caveat stale consequence low;\n\
        stale qualifies e;\n\
        state t = 0;\n\
        event see;\n\
        event bump;\n\
        on see reveal e supports ok;\n\
        on bump set t = qualified(t + 1, e);\n";
    let mut live = ReactiveSession::from_source(SOURCE).unwrap();
    send(&mut live, "see");
    send(&mut live, "bump");
    let saved = live.save_json().unwrap();
    assert!(saved.len() < 4096, "{saved}");
    let plain = ReactiveSession::restore_json(SOURCE, &saved).unwrap();
    assert_eq!(plain.save_json().unwrap(), saved);

    // 32 MiB of whitespace before the save and 32 MiB inside it, after `{`.
    let pad = " \n\t\r".repeat(8 * 1024 * 1024);
    let body = saved.strip_prefix('{').unwrap();
    let padded = format!("{pad}{{{pad}{body}");
    assert_eq!(padded.len(), PAST + saved.len());
    drop(pad);
    let mut restored = ReactiveSession::restore_json(SOURCE, &padded)
        .unwrap_or_else(|error| panic!("{} bytes: {error}", padded.len()));
    drop(padded);
    assert_eq!(restored.save_json().unwrap(), saved);

    // The session that wrote the save and the one restored from the padded
    // text give the same outcome to the next event, and the same save after.
    let next = send(&mut live, "bump");
    assert_eq!(send(&mut restored, "bump"), next);
    assert_eq!(restored.save_json().unwrap(), live.save_json().unwrap());
}

/// The replay save's bound stays: the runtime never writes a replay save past
/// 4,096 bytes, so one padded past it is refused, as before.
#[test]
fn d070_a_replay_save_keeps_its_bound() {
    const SOURCE: &str = "state t = 0;\nevent bump;\non bump set t = t + 1;\n";
    let mut live = ReactiveSession::from_source(SOURCE).unwrap();
    send(&mut live, "bump");
    let compact = live.save_compact_json().unwrap();
    assert!(
        compact.starts_with("{\"schema\":\"caveat-replay-save"),
        "{compact}"
    );
    let restored = ReactiveSession::restore_json(SOURCE, &compact).unwrap();
    assert_eq!(restored.save_json().unwrap(), live.save_json().unwrap());
    let padded = format!("{compact}{}", " ".repeat(4096));
    let Err(error) = ReactiveSession::restore_json(SOURCE, &padded) else {
        panic!("a replay save past 4,096 bytes restored");
    };
    assert_eq!(
        error,
        "cannot restore save: a replay save is at most 4096 bytes"
    );
}
