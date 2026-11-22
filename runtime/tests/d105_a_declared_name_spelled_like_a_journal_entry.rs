//! D105: under `journal window`, the journal's entries are named
//! `journal@K`, and a program may declare a caveat, claim, commitment or
//! evidence spelled the same way. When journal entry `journal@K` departed,
//! the session looked its name up as a record's: it removed the declared
//! node of that name with its declared relations, compacted what cited
//! declared evidence of that name, and a pin on that evidence held the
//! entry retired past its window. A renewal relating a departed declared
//! caveat to its new occurrence then panicked, a later commit could be fatal
//! ("journal entry journal@1 departed out of order"), and restore refused
//! the session's own saves or restored a session that answered later events
//! differently.
//!
//! An entry is never pinned and has no node, so it now departs in the event
//! that retires it and takes nothing a declared name of its spelling holds.
//! Each program here is played beside its twin, the same program with
//! `journal` spelled `jrnl` in each name it declares, but where a test
//! restores a save as it is: a compact save main wrote, and a save release
//! 1.4.1 wrote when it took an entry's retirement for that of declared
//! evidence spelled like it, whose restored session keeps that evidence
//! retired, and edits of it. See spec/caveat-departure-0.1.md,
//! spec/caveat-windows-0.1.md and spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// The outcome of `event`, accepted or refused, as text.
fn send(session: &mut ReactiveSession, event: &str, payload: &str) -> String {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_string(&outcome).unwrap()
}

/// `text` with the hash of each archive reference blanked: an archive names
/// its history, so the twin's hashes differ.
fn hashless(text: &str) -> String {
    let key = "\"archive_ref\":\"";
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find(key) {
        let start = at + key.len();
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        rest = &rest[rest.find('"').expect("a closed string")..];
    }
    out.push_str(rest);
    out
}

fn source_ids(source: &str) -> (String, String) {
    let save = ReactiveSession::from_source(source)
        .unwrap()
        .save_json()
        .unwrap();
    let save: Value = serde_json::from_str(&save).unwrap();
    (
        save["source_id"].as_str().unwrap().to_string(),
        save["source_sha256"].as_str().unwrap().to_string(),
    )
}

/// Plays `events` on `template` with `NAME` spelled `journal`, and on its
/// twin with `NAME` spelled `jrnl`. After each event the outcome and the
/// save are the twin's, name for name; the save restores and saves the same
/// bytes again; and every session restored so far answers the events still
/// to come as the live one does. Returns the save after each event.
fn plays_as_its_twin(template: &str, events: &[(&str, &str)]) -> Vec<Value> {
    let journal = template.replace("NAME", "journal");
    let twin_source = template.replace("NAME", "jrnl");
    let (journal_id, journal_sha) = source_ids(&journal);
    let (twin_id, twin_sha) = source_ids(&twin_source);
    let renamed = |text: &str| {
        hashless(
            &text
                .replace(&twin_id, &journal_id)
                .replace(&twin_sha, &journal_sha)
                .replace("jrnl", "journal"),
        )
    };
    let mut live = ReactiveSession::from_source(&journal).unwrap();
    let mut twin = ReactiveSession::from_source(&twin_source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    let mut saves = Vec::new();
    for (index, (event, payload)) in events.iter().enumerate() {
        let answer = send(&mut live, event, payload);
        assert_eq!(
            hashless(&answer),
            renamed(&send(&mut twin, event, payload)),
            "{index}: {event}"
        );
        for run in &mut restored {
            assert_eq!(send(run, event, payload), answer, "{index}: {event}");
        }
        let save = live.save_json().unwrap();
        assert_eq!(
            hashless(&save),
            renamed(&twin.save_json().unwrap()),
            "{index}: {event}"
        );
        let again = ReactiveSession::restore_json(&journal, &save)
            .unwrap_or_else(|error| panic!("{index}: {event}: {error}"));
        assert_eq!(again.save_json().unwrap(), save, "{index}: {event}");
        restored.push(again);
        saves.push(serde_json::from_str(&save).unwrap());
    }
    let save = live.save_json().unwrap();
    for run in &restored {
        assert_eq!(run.save_json().unwrap(), save);
    }
    saves
}

fn plain<'a>(events: &[&'a str]) -> Vec<(&'a str, &'static str)> {
    events.iter().map(|event| (*event, "{}")).collect()
}

/// The relations the save lists: those events made.
fn relations(save: &Value) -> Vec<Value> {
    save["graph"]["relations"]
        .as_array()
        .unwrap_or_else(|| panic!("{save}"))
        .clone()
}

/// A declared caveat `journal@1` on renewable evidence. Once journal entry
/// `journal@1` departed, it took the caveat's node with it, and the next
/// `renew` related the caveat to the new occurrence and panicked.
#[test]
fn a_declared_caveat_spelled_like_an_entry_qualifies_each_renewal() {
    let source = "claim c;
evidence e from \"e\";
caveat NAME@1 consequence low;
NAME@1 qualifies e;
renewable e limit 5;
journal window 1;
event see;
event go;
event go2;
event r;
on see reveal e supports c;
on go commit d1 because enough using qualified(1, e);
on go2 commit d2 because enough using qualified(1, e);
on r renew e;
";
    let saves = plays_as_its_twin(source, &plain(&["see", "go", "go2", "r", "r"]));
    assert_eq!(saves[2]["journal_departed"], json!(1), "{}", saves[2]);
    for occurrence in ["e@2", "e@3"] {
        assert!(
            relations(&saves[4]).contains(&json!(["journal@1", "qualifies", occurrence])),
            "{occurrence}: {}",
            saves[4]
        );
    }
}

/// Declared evidence `journal@1` and `journal@2`, related by declarations.
/// A save lists the graph's relations from the first one an event made, so
/// once the departed node took its declared relations, the save left out as
/// many of the event-made ones, and restore refused it ("relation d2
/// relies_on e: e is not observed").
#[test]
fn declared_evidence_spelled_like_entries_keeps_its_relations() {
    let source = "claim c;
evidence e from \"e\";
evidence NAME@1 from \"x\";
evidence NAME@2 from \"y\";
caveat k consequence low;
k qualifies NAME@1;
NAME@1 supports c;
NAME@2 supports c;
journal window 1;
event see;
event go;
event go2;
event go3;
event q;
on see reveal e supports c;
on go commit d1 because enough using qualified(1, e);
on go2 commit d2 because enough using qualified(1, e);
on go3 commit d3 because enough using qualified(1, e);
on q qualify e with k;
";
    let saves = plays_as_its_twin(source, &plain(&["see", "go", "q", "go2", "go3"]));
    assert_eq!(saves[4]["journal_departed"], json!(2), "{}", saves[4]);
    assert_eq!(
        relations(&saves[4]),
        vec![
            json!(["e", "supports", "c"]),
            json!(["d1", "relies_on", "e"]),
            json!(["k", "qualifies", "e"]),
            json!(["d2", "retains", "k"]),
            json!(["d2", "relies_on", "e"]),
            json!(["d3", "retains", "k"]),
            json!(["d3", "relies_on", "e"]),
        ],
        "{}",
        saves[4]
    );
}

/// A declared claim `journal@1` with a declared relation. Its departure left
/// a later reveal's relation out of the save: "reveal effect has no matching
/// graph relation".
#[test]
fn a_declared_claim_spelled_like_an_entry_stays() {
    let source = "claim c;
claim NAME@1;
evidence e from \"e\";
e supports NAME@1;
journal window 1;
event a;
event b;
event look;
on a commit x because enough;
on b commit z because enough;
on look reveal e supports c;
";
    plays_as_its_twin(source, &plain(&["a", "b", "look", "a"]));
}

/// Declared evidence `journal@1` that a declaration reopens a commitment
/// for. Journal entry `journal@1`'s departure took the evidence and the
/// reopening's cause, so a later `sx` read `hold` as reopened by nothing and
/// was accepted, where a session restored from a save written after that
/// departure refused it: "cannot qualify a value with unobserved evidence
/// journal@1".
#[test]
fn a_declared_reopening_cause_spelled_like_an_entry() {
    let source = "claim c;
evidence NAME@1 from \"g\";
commit hold because enough;
reopen hold because NAME@1;
journal window 1;
state x = 0;
event a;
event b;
event d;
event sx;
on a commit p because enough;
on b commit q because enough;
on d commit r because enough;
on sx set x = if(reopened(hold), 1, 0);
";
    plays_as_its_twin(source, &plain(&["sx", "a", "b", "sx", "d", "sx"]));
}

/// A declared commitment `journal@1` that a declaration reopens. Journal
/// entry `journal@1`'s departure took the commitment's node, its reopening
/// and its relations with it.
#[test]
fn a_declared_commitment_spelled_like_an_entry_stays_open() {
    let source = "claim c;
evidence why from \"w\";
why supports c;
commit NAME@1 because enough;
reopen NAME@1 because why;
journal window 1;
event a;
event b;
event c2;
on a commit p because enough;
on b commit q because enough;
on c2 commit r because enough;
";
    let saves = plays_as_its_twin(source, &plain(&["a", "b", "c2"]));
    assert_eq!(saves[2]["journal_departed"], json!(2), "{}", saves[2]);
    assert_eq!(
        saves[2]["graph"]["open"],
        json!(["journal@1"]),
        "{}",
        saves[2]
    );
}

/// Renewable evidence named `journal`, with a limit, beside declared
/// evidence `journal@2`: every `renew` is refused, as Renewal 0.1 promises,
/// before and after journal entry `journal@2` departs. Its departure took the
/// declared evidence, and the next `renew` was accepted and made
/// `journal@2`, a save restore then refused ("journal@2 already exists").
#[test]
fn a_renewal_onto_a_declared_name_spelled_like_an_entry_stays_refused() {
    let source = "claim c;
evidence NAME from \"f\";
evidence NAME@2 from \"g\";
renewable NAME limit 5;
journal window 1;
event refresh;
event a;
event b;
event d;
on refresh renew NAME;
on a commit p because enough;
on b commit q because enough;
on d commit r because enough;
";
    let saves = plays_as_its_twin(
        source,
        &plain(&["refresh", "a", "b", "refresh", "d", "refresh", "refresh"]),
    );
    assert_eq!(saves[6]["renewals"], Value::Null, "{}", saves[6]);
}

/// Declared evidence `journal@1`, observed by its declared stance, that a
/// declaration reopens `act` for. From `sets` on, `s`'s lineage cites it,
/// and that pin held journal entry `journal@1` retired past its window, so
/// entry `journal@2` departed first at `cm3` and ended the event with a
/// fatal error ("journal entry journal@1 departed out of order").
#[test]
fn a_pin_on_declared_evidence_spelled_like_an_entry_holds_no_entry() {
    let source = "claim c;
claim d;
evidence why from \"w\";
why supports c;
commit act because enough;
evidence NAME@1 from \"j\";
NAME@1 supports d;
reopen act because NAME@1;
journal window 1;
state s = 0;
event cm;
event cm2;
event cm3;
event sets;
on cm commit act2 because enough;
on cm2 commit act3 because enough;
on cm3 commit act4 because enough;
on sets set s = if(reopened(act), 1, 0);
";
    let saves = plays_as_its_twin(source, &plain(&["sets", "cm", "cm2", "cm3"]));
    assert_eq!(
        saves[3]["states"]["s"]["lineage"]["evidence"],
        json!(["journal@1"]),
        "{}",
        saves[3]
    );
    assert_eq!(saves[3]["journal_departed"], json!(2), "{}", saves[3]);
}

/// Renewable evidence named `journal` beside declared claim `journal@2`, and
/// the compact save main wrote after `see`, `g1`, `g2`, `g3` and `r`: there
/// journal entry `journal@2`'s departure at `g3` took the claim, and `r` was
/// accepted. Restore replays the events a compact save lists under the
/// rules a session follows now, which refuse `r`, so it refuses the save
/// and names that event.
#[test]
fn a_replay_save_with_a_renewal_onto_a_declared_name_is_refused() {
    let source = "claim c;
claim journal@2;
evidence journal from \"x\";
renewable journal limit 5;
journal window 1;
event see;
event r;
event g1;
event g2;
event g3;
on see reveal journal supports c;
on r renew journal;
on g1 commit d1 because enough using 1;
on g2 commit d2 because enough using 1;
on g3 commit d3 because enough using 1;
";
    let older = r#"{"schema":"caveat-replay-save/0.1","source_id":"fnv1a64:e92c83d3fd848da8:322","source_sha256":"d423fa8baa3b633d870c9fc024590dc4436da8ec0c8dd30289be6abb362c0289","events":[["see"],["g1"],["g2"],["g3"],["r"]]}"#;
    let mut live = ReactiveSession::from_source(source).unwrap();
    for event in ["see", "g1", "g2", "g3"] {
        let outcome: Value = serde_json::from_str(&send(&mut live, event, "{}")).unwrap();
        assert_eq!(outcome["outcome"], json!("accepted"), "{event}: {outcome}");
    }
    let refused: Value = serde_json::from_str(&send(&mut live, "r", "{}")).unwrap();
    let message = "event r, rule 2: generated occurrence journal@2 already exists";
    assert_eq!(refused["outcome"], json!("rejected"), "{refused}");
    assert_eq!(refused["message"], json!(message), "{refused}");
    match ReactiveSession::restore_json(source, older) {
        Err(error) => assert_eq!(
            error,
            format!("cannot restore save: event 5 (r) is not accepted: {message}")
        ),
        Ok(_) => panic!("restored {older}"),
    }
}

/// A program with declared evidence `journal@1`, never observed, that a
/// declaration's `reopen` names, read through `reopened(act)` by `rd`,
/// whose journal entry retires entry `journal@1`.
const KEPT: &str = "claim c;
evidence e from \"e\";
evidence journal@1 from \"j\";
commit act because enough;
reopen act because journal@1;
decisions d limit 4;
journal window 1;
state s = 0;
event see;
event cd;
event rd;
event tt;
event ts;
event rx;
event ck;
on see reveal e supports c;
on cd commit d because enough using 1;
on rd reopen d because e;
on rd set s = if(reopened(act), 1, 0);
on tt set s = s;
on ts set s = 0;
on rx set s = if(reopened(act), 1, 0);
on ck commit k because enough using if(reopened(act), 1, 0);
bind view.r = if(reopened(act), 1, 0) when s > 0;
";

/// The save release 1.4.1 wrote for `KEPT` after `see`, `cd` and `rd`.
const KEPT_AFTER_RD: &str = r#"{"schema":"caveat-reactive-save/0.1","source_id":"fnv1a64:3ade5d8980f170b3:555","source_sha256":"57278cdaa77387a7c9d73179757b1300ab3bebd63a55d0eb8c4be5414ab2a9f2","sequence":3,"last_event":"rd","elapsed":0.0,"states":{"s":{"value":1.0,"lineage":{"evidence":["journal@1"]}}},"graph":{"nodes":[{"kind":"commitment","name":"d@1","reason":"enough"}],"relations":[["e","supports","c"],["e","reopens","d@1"]],"open":["act","d@1"]},"commitment_bases":{"d@1":{"value":1.0,"provenance":{"evidence":[],"caveats":[]}}},"commitment_grounds":{"d@1":{}},"decision_series":{"d":{"limit":4,"current":"d@1","revisions":[{"id":"d@1","previous":null,"ordinal":1,"sequence":2,"event":"cd"}],"selection_qualifications":{"evidence":[],"caveats":[]}}},"observation_qualifications":{"e":{}},"reopening_qualifications":{"d@1":{"evidence":["e"]}},"decision_journal":[{"decision":"d","commitment":"d@1","change":"committed","sequence":2,"event":"cd","elapsed":0.0,"value":1.0,"because":[],"caveats":[]},{"decision":"d","commitment":"d@1","change":"reopened","sequence":3,"event":"rd","elapsed":0.0,"value":1.0,"because":["e"],"caveats":[]}],"retired":{"journal@1":3},"effects":[{"kind":"retire","history":"journal","record":"journal@1"},{"kind":"reopen","action":"d@1","because":"e"}]}"#;

/// Release 1.4.1 took the entry's retirement for the evidence's, so it
/// accepted `rd`'s read; the state cites the evidence, and the save lists
/// `journal@1` as retired, the entry held by the citation. While the
/// citation held it, the next journal entry ended its event with a fatal
/// error ("journal entry journal@1 departed out of order"); once `ts` let
/// the entry go, its departure took the evidence, and later reads of
/// `reopened(act)` cited nothing.
///
/// The 1.4.1 save restores to the same bytes and the same view. The
/// restored session departs the entry at its next accepted event and keeps
/// the evidence retired, where 1.4.1 lost it with the entry, so a later
/// state and a commitment's basis cite it, and it lists `journal@1` as
/// retired once the entry has gone. Each save it writes restores to the
/// same bytes, and the sessions restored from them answer what follows as
/// it does. A session started now refuses `rd`.
#[test]
fn an_older_save_keeps_a_declared_cause_spelled_like_an_entry_retired() {
    let mut fresh = ReactiveSession::from_source(KEPT).unwrap();
    send(&mut fresh, "see", "{}");
    send(&mut fresh, "cd", "{}");
    let refused: Value = serde_json::from_str(&send(&mut fresh, "rd", "{}")).unwrap();
    assert_eq!(
        refused["message"],
        json!("event rd, rule 4: cannot qualify a value with unobserved evidence journal@1"),
        "{refused}"
    );

    let mut live = ReactiveSession::restore_json(KEPT, KEPT_AFTER_RD).unwrap();
    assert_eq!(live.save_json().unwrap(), KEPT_AFTER_RD);
    let view =
        |session: &ReactiveSession| serde_json::to_value(&session.snapshot().bindings).unwrap();
    assert_eq!(view(&live), json!({"view": {"r": 1.0}}));
    let mut restored: Vec<ReactiveSession> = Vec::new();
    let mut saves = Vec::new();
    for event in ["tt", "rx", "ts", "rx", "ck", "cd"] {
        let answer = send(&mut live, event, "{}");
        for run in &mut restored {
            assert_eq!(send(run, event, "{}"), answer, "{event}");
        }
        let outcome: Value = serde_json::from_str(&answer).unwrap();
        assert_eq!(outcome["outcome"], json!("accepted"), "{event}: {answer}");
        let save = live.save_json().unwrap();
        let again = ReactiveSession::restore_json(KEPT, &save)
            .unwrap_or_else(|error| panic!("{event}: {error}"));
        assert_eq!(again.save_json().unwrap(), save, "{event}");
        restored.push(again);
        let save: Value = serde_json::from_str(&save).unwrap();
        assert_eq!(save["retired"], json!({"journal@1": 3}), "{event}: {save}");
        saves.push((outcome, save));
    }
    assert_eq!(
        saves[0].0["snapshot"]["effects"],
        json!([{"history": "journal", "kind": "depart", "record": "journal@1"}]),
        "{}",
        saves[0].0
    );
    assert_eq!(view(&live), json!({"view": {"r": 1.0}}));
    assert_eq!(
        saves[4].1["commitment_bases"]["k"]["provenance"]["evidence"],
        json!(["journal@1"]),
        "{}",
        saves[4].1
    );
    assert_eq!(saves[5].1["journal_departed"], json!(3), "{}", saves[5].1);
}

/// A kept cause is dated as its entry's retirement was: before the event
/// that departed the entry, at the sequence of the entry that took its
/// place while the journal holds that one. A save that dates it otherwise,
/// or that lists the evidence as retired after an entry that departed in
/// the event that retired it, is refused as before.
#[test]
fn a_kept_cause_is_dated_as_its_entry_was() {
    let refused =
        "cannot restore save: retired journal@1 is no record of a windowed history it holds";
    let mut live = ReactiveSession::restore_json(KEPT, KEPT_AFTER_RD).unwrap();
    send(&mut live, "tt", "{}");
    let after: Value = serde_json::from_str(&live.save_json().unwrap()).unwrap();
    assert_eq!(after["retired"], json!({"journal@1": 3}), "{after}");
    assert_eq!(after["journal_departed"], json!(1), "{after}");
    for at in [0, 1, 2, 4, 9] {
        let mut save = after.clone();
        save["retired"]["journal@1"] = json!(at);
        match ReactiveSession::restore_json(KEPT, &save.to_string()) {
            Err(error) => assert_eq!(error, refused, "{at}"),
            Ok(_) => panic!("restored with journal@1 retired at {at}"),
        }
    }
    let source = KEPT
        .replace("event ck;\n", "event ck;\nevent rd0;\n")
        .replace(
            "on rd reopen d because e;\n",
            "on rd reopen d because e;\non rd0 reopen d because e;\n",
        );
    let mut fresh = ReactiveSession::from_source(&source).unwrap();
    for event in ["see", "cd", "rd0"] {
        let outcome: Value = serde_json::from_str(&send(&mut fresh, event, "{}")).unwrap();
        assert_eq!(outcome["outcome"], json!("accepted"), "{event}: {outcome}");
    }
    let mut save: Value = serde_json::from_str(&fresh.save_json().unwrap()).unwrap();
    assert_eq!(save["journal_departed"], json!(1), "{save}");
    for at in [2, 3] {
        save["retired"] = json!({"journal@1": at});
        match ReactiveSession::restore_json(&source, &save.to_string()) {
            Err(error) => assert_eq!(error, refused, "{at}"),
            Ok(_) => panic!("restored with journal@1 retired at {at}"),
        }
    }
}
