//! D104: restore accepted a relation listed more often than events add it.
//! D101 refused a stance listed twice or one a declaration gives, and D102 a
//! qualification a declaration gives, but a save that listed an event-made
//! `late qualifies e`, `d retains k` or `d relies_on e` twice restored, and so
//! did a `reopens` relation listed twice, or one a declaration gives, once the
//! decision journal had let entries depart, or a declared one beside a
//! journaled reopening that would make it. So did a `retains` or `relies_on`
//! relation from a commitment the program declares, which no event commits,
//! beside a basis the save gave it. The restored session related each copy,
//! so it showed the relation twice in every later snapshot and save, and a
//! commitment retained `["k", "k"]` where the live session retains `["k"]`.
//! A session lists a relation once, except a qualification: one taken back
//! can be added again, and a renewal or a reading takes a caveat as often as
//! the program declares it on the template. See spec/caveat-save-0.1.md
//! ("Restore validation", "Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn answer(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted a save refused for: {expected}"),
    }
}

/// Plays `events`, each accepted. After each, the save restores and saves the
/// same bytes again, and every session restored so far answers the events
/// still to come as the live one does. Returns the last save.
fn plays_and_restores(source: &str, events: &[&str]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    for (index, event) in events.iter().enumerate() {
        let outcome = answer(&mut live, event);
        assert_eq!(
            outcome["outcome"], "accepted",
            "{index}: {event}: {outcome}"
        );
        for run in &mut restored {
            assert_eq!(answer(run, event), outcome, "{index}: {event}");
        }
        let save = live.save_json().unwrap();
        let again = ReactiveSession::restore_json(source, &save)
            .unwrap_or_else(|error| panic!("{index}: {event}: {error}"));
        assert_eq!(again.save_json().unwrap(), save, "{index}: {event}");
        restored.push(again);
    }
    serde_json::from_str(&live.save_json().unwrap()).unwrap()
}

/// `save` with `relation` added to its graph.
fn with(save: &Value, relation: [&str; 3]) -> Value {
    let mut save = save.clone();
    let relations = &mut save["graph"]["relations"];
    if relations.is_null() {
        *relations = json!([]);
    }
    relations.as_array_mut().unwrap().push(json!(relation));
    save
}

/// How many times `save` lists `relation`.
fn listed(save: &Value, relation: [&str; 3]) -> usize {
    save["graph"]["relations"]
        .as_array()
        .map_or(0, |relations| {
            relations.iter().filter(|r| **r == json!(relation)).count()
        })
}

/// `late` qualifies `foo`, and `uq` takes it back.
const TAKE_BACK: &str = "claim c;
evidence foo from \"f\";
caveat late consequence low;
state x = 0;
event see;
event q;
event uq;
event sx;
on see reveal foo supports c;
on q qualify foo with late;
on uq unqualify foo with late;
on sx set x = qualified(1, foo);";

/// The program declares `late qualifies foo` twice, so every renewal of
/// `foo` and every reading of `runs` takes it twice.
const TWICE: &str = "claim c;
evidence foo from \"f\";
caveat late consequence low;
late qualifies foo;
late qualifies foo;
renewable foo limit 4;
readings runs from foo limit 4;
state v = 1;
event see;
event r;
event s1;
on see reveal foo supports c;
on r renew foo;
on s1 sample runs = v supports c;";

/// `d` is committed on `e`, which `k` qualifies, and `re` reopens it.
const COMMIT: &str = "claim c;
evidence e from \"e\";
caveat k consequence low;
k qualifies e;
event see;
event go;
event re;
on see reveal e supports c;
on go commit d because enough using qualified(1, e);
on re reopen d because e;";

/// As `COMMIT`, with a journal of one entry, a commitment `held` the
/// declarations reopen for `e`, `rh` reopening `held` for `f`, and `go2`
/// committing `d2`, so that the entries for `d` and `held` depart.
const PARTIAL: &str = "claim c;
evidence e from \"e\";
evidence f from \"f\";
caveat k consequence low;
k qualifies e;
journal window 1;
commit held because enough;
reopen held because e;
event see;
event see2;
event go;
event re;
event rh;
event go2;
on see reveal e supports c;
on see2 reveal f supports c;
on go commit d because enough using qualified(1, e);
on re reopen d because e;
on rh reopen held because f;
on go2 commit d2 because enough using 1;";

/// The saves a session writes restore, though a qualification taken back
/// and added again is listed once for each time it was added, and one the
/// program declares twice on a template is listed twice for each renewal
/// and reading.
#[test]
fn the_saves_a_session_writes_restore() {
    let save = plays_and_restores(
        TAKE_BACK,
        &["see", "q", "uq", "q", "sx", "uq", "q", "sx", "uq", "sx"],
    );
    assert_eq!(listed(&save, ["late", "qualifies", "foo"]), 3, "{save}");
    assert_eq!(save["taken_back"].as_array().unwrap().len(), 3, "{save}");

    let save = plays_and_restores(TWICE, &["see", "r", "see", "s1", "r", "s1"]);
    // The declared ones are the program's, and the save lists none.
    assert_eq!(listed(&save, ["late", "qualifies", "foo"]), 0, "{save}");
    for occurrence in ["foo@2", "foo@3", "runs@1", "runs@2"] {
        assert_eq!(
            listed(&save, ["late", "qualifies", occurrence]),
            2,
            "{occurrence}: {save}"
        );
    }

    let save = plays_and_restores(COMMIT, &["see", "go", "re", "re"]);
    for relation in [
        ["d", "retains", "k"],
        ["d", "relies_on", "e"],
        ["e", "reopens", "d"],
    ] {
        assert_eq!(listed(&save, relation), 1, "{relation:?}: {save}");
    }
    plays_and_restores(
        PARTIAL,
        &["see", "see2", "go", "re", "rh", "go2", "re", "rh"],
    );
}

/// A qualification listed again without the take-back that lets an event
/// add it again is refused, and so is one listed more often than the
/// program declares it on the template.
#[test]
fn a_qualification_listed_again_is_refused() {
    let late = "claim c;
evidence e from \"e\";
caveat late consequence low;
event see;
event q;
on see reveal e supports c;
on q qualify e with late;";
    let save = plays_and_restores(late, &["see", "q", "q"]);
    refused(
        late,
        &with(&save, ["late", "qualifies", "e"]),
        "relation late qualifies e is listed twice",
    );

    // `withdrawn`, which a withdrawal applies once and nothing takes back.
    let drop = "claim c;
evidence e from \"e\";
evidence f from \"f\";
event see;
event drop;
on see reveal e supports c;
on see reveal f supports c;
on drop withdraw e because f;";
    let save = plays_and_restores(drop, &["see", "drop", "drop"]);
    assert_eq!(listed(&save, ["withdrawn", "qualifies", "e"]), 1, "{save}");
    refused(
        drop,
        &with(&save, ["withdrawn", "qualifies", "e"]),
        "relation withdrawn qualifies e is listed twice",
    );

    // A caveat the program takes back, with no take-back saved.
    let save = plays_and_restores(TAKE_BACK, &["see", "q"]);
    refused(
        TAKE_BACK,
        &with(&save, ["late", "qualifies", "foo"]),
        "relation late qualifies foo is listed twice, and no take-back of it is saved",
    );
    // With take-backs saved, the older check counts them, as before.
    let save = plays_and_restores(TAKE_BACK, &["see", "q", "uq", "q"]);
    refused(
        TAKE_BACK,
        &with(&save, ["late", "qualifies", "foo"]),
        "a qualification has two live relations",
    );

    // However many copies past the limit the save lists, the message names
    // the first, as D101's does.
    let save = plays_and_restores(TWICE, &["see", "r", "s1"]);
    for occurrence in ["foo@2", "runs@1"] {
        let relation = ["late", "qualifies", occurrence];
        for extra in [
            &with(&save, relation),
            &with(&with(&save, relation), relation),
        ] {
            refused(
                TWICE,
                extra,
                &format!(
                    "relation late qualifies {occurrence} is listed more than the 2 times the \
                     program declares late qualifies foo"
                ),
            );
        }
    }
    let once = TWICE.replacen("late qualifies foo;\n", "", 1);
    let save = plays_and_restores(&once, &["see", "r", "s1"]);
    for occurrence in ["foo@2", "runs@1"] {
        let relation = ["late", "qualifies", occurrence];
        for extra in [
            &with(&save, relation),
            &with(&with(&save, relation), relation),
        ] {
            refused(
                &once,
                extra,
                &format!("relation late qualifies {occurrence} is listed twice"),
            );
        }
    }
}

/// A commitment's `retains` and `relies_on` relations, and a `reopens`
/// relation, listed twice are refused, and so is a `reopens` relation a
/// declaration gives. While the journal holds every entry it refuses such a
/// `reopens` relation first, as before, unless the save journals a
/// reopening that would make it.
#[test]
fn a_commitment_relation_listed_twice_is_refused() {
    let save = plays_and_restores(COMMIT, &["see", "go", "re"]);
    refused(
        COMMIT,
        &with(&save, ["d", "retains", "k"]),
        "relation d retains k is listed twice",
    );
    refused(
        COMMIT,
        &with(&save, ["d", "relies_on", "e"]),
        "relation d relies_on e is listed twice",
    );
    refused(
        COMMIT,
        &with(&save, ["e", "reopens", "d"]),
        "decision journal: reopening history differs from the graph",
    );

    // Once the journal entries for `d` and `held` have departed.
    let save = plays_and_restores(PARTIAL, &["see", "see2", "go", "re", "rh", "go2"]);
    refused(
        PARTIAL,
        &with(&save, ["e", "reopens", "d"]),
        "relation e reopens d is listed twice",
    );
    refused(
        PARTIAL,
        &with(&save, ["f", "reopens", "held"]),
        "relation f reopens held is listed twice",
    );
    refused(
        PARTIAL,
        &with(&save, ["e", "reopens", "held"]),
        "relation e reopens held: a declaration gives it, and no event adds it again",
    );

    // A declared reopening listed beside a journal entry and a reopening
    // record that would make it, though the journal holds every entry.
    let declared = "claim c;
evidence e from \"e\";
commit held because enough;
reopen held because e;
event see;
event rh;
on see reveal e supports c;
on rh reopen held because e;";
    let save = plays_and_restores(declared, &["see", "rh"]);
    let mut forged = with(&save, ["e", "reopens", "held"]);
    refused(
        declared,
        &forged,
        "decision journal: reopening history differs from the graph",
    );
    forged["decision_journal"] = json!([{
        "because": ["e"],
        "caveats": [],
        "change": "reopened",
        "commitment": "held",
        "decision": "held",
        "elapsed": 0.0,
        "event": "rh",
        "sequence": 2
    }]);
    forged["reopening_qualifications"] = json!({"held": {"evidence": ["e"]}});
    refused(
        declared,
        &forged,
        "relation e reopens held: a declaration gives it, and no event adds it again",
    );
}

/// `held`, a commitment the program declares retaining `k`, and a rule that
/// would commit it on `e`, which lets a basis of `held` hold `e` though the
/// rule's commit is refused.
const HELD: &str = "claim c;
evidence e from \"e\";
caveat k consequence low;
k qualifies e;
commit held because enough retaining k;
event see;
event g2;
on see reveal e supports c;
on g2 commit held because enough using qualified(1, e);";

/// No event commits a commitment the program declares, so none relates it
/// by `retains` or `relies_on`. Where the save gives it no basis, the basis
/// check refuses such a relation first, as before; beside a basis the save
/// gives it, this check does.
#[test]
fn a_declared_commitments_relations_are_refused() {
    let mut live = ReactiveSession::from_source(HELD).unwrap();
    assert_eq!(answer(&mut live, "see")["outcome"], "accepted");
    assert_eq!(answer(&mut live, "g2")["code"], "already_committed");
    let basis = json!({"provenance": {"caveats": ["k"], "evidence": ["e"]}, "value": 1.0});

    let save = plays_and_restores(HELD, &["see"]);
    refused(
        HELD,
        &with(&save, ["held", "retains", "k"]),
        "relation held retains k: k is not in held's basis",
    );
    let mut based = save.clone();
    based["commitment_bases"] = json!({ "held": basis });
    refused(
        HELD,
        &with(&based, ["held", "retains", "k"]),
        "relation held retains k: a declaration gives it, and no event adds it again",
    );
    refused(
        HELD,
        &with(&based, ["held", "relies_on", "e"]),
        "relation held relies_on e: the program declares held, so no event commits it",
    );

    // Declared retaining nothing.
    let plain = HELD.replace(" retaining k;", ";");
    let save = plays_and_restores(&plain, &["see"]);
    let mut based = save.clone();
    based["commitment_bases"] = json!({ "held": basis });
    refused(
        &plain,
        &with(&based, ["held", "retains", "k"]),
        "relation held retains k: the program declares held, so no event commits it",
    );
}

/// A save an older check refuses keeps that check's message, though it also
/// lists a relation more often than events add it.
#[test]
fn older_checks_keep_their_messages() {
    let save = plays_and_restores(COMMIT, &["see", "go", "re"]);
    let save = with(&with(&save, ["d", "retains", "k"]), ["e", "opposes", "c"]);
    refused(
        COMMIT,
        &save,
        "relation e opposes c: no rule of this program, or procedure one calls, reveals e \
         opposes c",
    );
    // A declared reopening listed, with no reopening record of `held`.
    let save = plays_and_restores(PARTIAL, &["see", "go", "re", "go2"]);
    refused(
        PARTIAL,
        &with(&save, ["e", "reopens", "held"]),
        "reopened commitment held has no reopening record",
    );
    // A declared qualification listed again, twice: D102's message.
    let save = plays_and_restores(COMMIT, &["see"]);
    let save = with(
        &with(&save, ["k", "qualifies", "e"]),
        ["k", "qualifies", "e"],
    );
    refused(
        COMMIT,
        &save,
        "relation k qualifies e: a declaration gives it, and no event adds it again",
    );
}
