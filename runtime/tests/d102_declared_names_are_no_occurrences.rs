//! D102: restore read a name spelled like an occurrence of renewable
//! evidence as one even where the program declares that name itself. A
//! program may declare evidence `foo@2` beside `renewable foo`, or a claim,
//! caveat or commitment so named (load refuses such a name spelled like a
//! record of a reading stream or a decision series, but not this one), and
//! no renewal ever makes `foo@2`: a renewal that would fails, since the name
//! is taken, and renewals are made in order, so none gets past it. Restore
//! took `foo@2` for `foo`'s second occurrence wherever it read a name as a
//! record of a history. So it refused saves the session itself wrote, once
//! a caveat reached `foo@2` through its own declarations and was looked for
//! on `foo` instead, and it accepted edited saves that listed `foo@2` among
//! `foo`'s renewals, or a windowed list past it, or gave `foo@2` `foo`'s
//! records: an observation record, a qualification `foo` can take, a
//! scheduled qualification, the flows that carry `foo`'s records. Restore
//! now reads a name the program declares as itself, however it is spelled,
//! and refuses a renewals list that holds one, or a number past it, after
//! every older check. The same reading held for a declared `foo@2` beside
//! `foo` with no renewal at all, in what the last event reported. And a
//! declared qualification of evidence listed again in a save, which restore
//! refused for `foo@2` beside renewable `foo` only by reading it as `foo`,
//! and accepted otherwise, is now refused, as D101 does for a stance.
//! See spec/caveat-save-0.1.md ("Restore validation", "Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn answer(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
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
/// still to come as the live one does. Returns the save after each event.
fn plays_and_restores(source: &str, events: &[(&str, &str)]) -> Vec<Value> {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    let mut saves = Vec::new();
    for (index, (event, payload)) in events.iter().enumerate() {
        let outcome = answer(&mut live, event, payload);
        assert_eq!(
            outcome["outcome"], "accepted",
            "{index}: {event}: {outcome}"
        );
        for run in &mut restored {
            assert_eq!(answer(run, event, payload), outcome, "{index}: {event}");
        }
        let save = live.save_json().unwrap();
        let again = ReactiveSession::restore_json(source, &save)
            .unwrap_or_else(|error| panic!("{index}: {event}: {error}"));
        assert_eq!(again.save_json().unwrap(), save, "{index}: {event}");
        restored.push(again);
        saves.push(serde_json::from_str(&save).unwrap());
    }
    saves
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

/// The finding's program, with evidence `declared` beside `renewal` (a
/// `renewable foo` directive) and `extra` added. `declared` supports `d`,
/// and is the cause the declarations reopen `hold` for, so `sx` carries it
/// into `x`. `peek` reveals `foo` supporting `c`,
/// `refresh` renews `foo`, `q` qualifies `foo` with `k` and `uq` takes that
/// back, `sq` qualifies `foo` with `late` after 0.2, `look` reveals `foo`,
/// `see_why` reveals `why`, `wd` withdraws `foo` because of it, and `sy`
/// reads whether `foo` is observed.
fn program(declared: &str, renewal: &str, extra: &str) -> String {
    format!(
        "claim c;
claim d;
evidence foo from \"f\";
evidence {declared} from \"g\";
evidence why from \"w\";
{declared} supports d;
caveat k consequence low;
caveat late consequence low;
commit hold because enough;
reopen hold because {declared};
{renewal};
state x = 0;
state y = 0;
event peek;
event refresh;
event q;
event uq;
event sq;
event tick dt min 0 max 0.1;
event look;
event see_why;
event wd;
event sx;
event sy;
on peek reveal foo supports c;
on refresh renew foo;
on q qualify foo with k;
on uq unqualify foo with k;
on sq qualify foo with late after 0.2;
on look reveal foo;
on see_why reveal why supports d;
on wd withdraw foo because why;
on sx set x = if(reopened(hold), 1, 0);
on sy set y = if(observed(foo), 1, 0);
{extra}"
    )
}

const LIMIT: &str = "renewable foo limit 3";
const TICK: &str = r#"{"dt": 0.1}"#;

/// `events`, each with no payload but `tick`'s.
fn plain(events: &[&'static str]) -> Vec<(&'static str, &'static str)> {
    events
        .iter()
        .map(|event| (*event, if *event == "tick" { TICK } else { "{}" }))
        .collect()
}

/// The saves a session of such a program writes restore, save the same bytes
/// again and answer later events as the live session does. Before, once
/// `kd` qualifies `d`, the save after `sx` was refused ("state x: kd cannot
/// qualify any of its evidence"): `kd` reaches `foo@2` through `d`, and
/// restore looked for it on `foo`. So were a commitment and a reading that
/// rest on `x`'s cause.
#[test]
fn the_saves_a_session_writes_restore() {
    let source = program(
        "foo@2",
        LIMIT,
        "caveat kd consequence low;
kd qualifies d;
evidence e from \"e\";
readings r from e limit 3;
event cm;
event rs;
on cm commit act because enough using if(reopened(hold), 1, 0);
on rs sample r = if(reopened(hold), 1, 0) supports c;",
    );
    let saves = plays_and_restores(
        &source,
        &plain(&[
            "peek", "q", "sq", "see_why", "sx", "sy", "cm", "rs", "uq", "tick", "tick", "look",
            "wd",
        ]),
    );
    assert_eq!(
        saves[4]["states"]["x"]["lineage"],
        json!({"caveats": ["kd"], "evidence": ["foo@2"]})
    );
    // No renewal makes `foo@2`: the name is taken, and the event is refused
    // (D103).
    let mut session = ReactiveSession::from_source(&source).unwrap();
    let outcome = answer(&mut session, "refresh", "{}");
    assert_eq!(outcome["code"], "renewal_limit", "{outcome}");
    assert_eq!(
        outcome["message"],
        "event refresh, rule 2: generated occurrence foo@2 already exists"
    );
    // Declared as `foo@7`, the name leaves `foo` its renewals, and the
    // occurrences they make take `foo`'s records.
    let source = program("foo@7", LIMIT, "");
    plays_and_restores(
        &source,
        &plain(&[
            "peek", "refresh", "peek", "q", "see_why", "sx", "refresh", "peek", "sq", "tick",
            "tick", "look", "uq", "wd", "sy",
        ]),
    );
}

const DECLARED_RENEWAL: &str =
    "renewable foo occurrence foo@2: no renewal makes foo@2, which the program declares";

/// A renewals list that names the declared `foo@2`, with or without a window,
/// is refused. Before, the restored session took `foo@2` for `foo`'s current
/// occurrence: `peek` gave it `foo`'s stance, and `refresh`, which no live
/// session of the program can play, renewed `foo` into `foo@3`; with a
/// window, `refresh` retired and departed the declared evidence. The same
/// list let a stance on `foo@2` pass D101's check, and a report of the
/// renewal pass as the last event's: each is now refused by the check that
/// reads `foo@2` as itself.
#[test]
fn a_renewals_list_naming_declared_evidence_is_refused() {
    let source = program("foo@2", LIMIT, "");
    let saves = plays_and_restores(&source, &plain(&["peek"]));
    let mut renewed = saves[0].clone();
    renewed["renewals"] = json!({"foo": ["foo", "foo@2"]});
    refused(&source, &renewed, DECLARED_RENEWAL);
    let mut stance = with(&renewed, ["foo@2", "supports", "c"]);
    stance["observation_qualifications"]["foo@2"] = json!({});
    refused(
        &source,
        &stance,
        "observation of foo@2: no rule or procedure of this program reveals foo@2",
    );
    let mut report = renewed.clone();
    report["last_event"] = json!("refresh");
    report["sequence"] = json!(2);
    report["effects"] = json!([{"kind": "renew", "evidence": "foo", "occurrence": "foo@2"}]);
    refused(
        &source,
        &report,
        "renew effect is not one the last event can make",
    );
    let source = program("foo@2", "renewable foo window 1", "");
    let saves = plays_and_restores(&source, &plain(&["peek", "look"]));
    let mut retired = saves[1].clone();
    retired["renewals"] = json!({"foo": ["foo", "foo@2"]});
    retired["retired"] = json!({"foo": 1});
    refused(&source, &retired, DECLARED_RENEWAL);
}

/// Nor does a windowed list get past the declared name, which it could
/// skip: the gap read as a departed record of `foo`. Before, the restored
/// session revealed `foo@3`, and its `refresh` made `foo@4`, where every
/// live one fails. A claim, evidence, caveat or commitment so named stops
/// the renewals, and the least one declared counts: with `foo@3` declared,
/// a live session renews `foo` once. A place is no node, and stops none.
#[test]
fn a_windowed_list_past_a_declared_name_is_refused() {
    let source = program("foo@2", "renewable foo window 1", "");
    let saves = plays_and_restores(&source, &plain(&["peek", "peek"]));
    let past = |save: &Value, occurrence: &str| {
        let mut past = save.clone();
        past["graph"]["nodes"] = json!([{"kind": "occurrence", "name": occurrence}]);
        past["renewals"] = json!({"foo": ["foo", occurrence]});
        past["retired"] = json!({"foo": 1});
        past
    };
    for occurrence in ["foo@3", "foo@9"] {
        refused(
            &source,
            &past(&saves[1], occurrence),
            &format!(
                "renewable foo occurrence {occurrence}: no renewal gets past foo@2, which the \
                 program declares"
            ),
        );
    }
    let claim = "claim c;
claim foo@2;
evidence foo from \"f\";
renewable foo window 1;
event peek;
event refresh;
on peek reveal foo supports c;
on refresh renew foo;";
    let saves = plays_and_restores(claim, &plain(&["peek", "peek"]));
    refused(
        claim,
        &past(&saves[1], "foo@3"),
        "renewable foo occurrence foo@3: no renewal gets past foo@2, which the program declares",
    );
    let source = program("foo@3", "renewable foo window 1", "");
    let saves = plays_and_restores(&source, &plain(&["peek", "refresh", "peek"]));
    assert_eq!(saves[2]["renewals"], json!({"foo": ["foo", "foo@2"]}));
    let saves = plays_and_restores(&source, &plain(&["peek", "peek"]));
    refused(
        &source,
        &past(&saves[1], "foo@4"),
        "renewable foo occurrence foo@4: no renewal gets past foo@3, which the program declares",
    );
    // With two such names declared, the least counts, whichever is found first.
    let two = "claim c;
claim foo@5;
evidence foo from \"f\";
evidence foo@3 from \"g\";
renewable foo window 1;
event peek;
event refresh;
on peek reveal foo supports c;
on refresh renew foo;";
    let saves = plays_and_restores(two, &plain(&["peek", "peek"]));
    refused(
        two,
        &past(&saves[1], "foo@4"),
        "renewable foo occurrence foo@4: no renewal gets past foo@3, which the program declares",
    );
    let place = "claim c;
place foo@2 kind room;
evidence foo from \"f\";
renewable foo window 1;
event peek;
event refresh;
on peek reveal foo supports c;
on refresh renew foo;";
    let saves = plays_and_restores(
        place,
        &plain(&["peek", "refresh", "peek", "refresh", "peek"]),
    );
    assert_eq!(saves[4]["renewals"], json!({"foo": ["foo", "foo@3"]}));
}

/// A `qualifies` relation a declaration gives is listed again in no save, as
/// no event adds it again: a live `qualify` of the declared pair, now or
/// after a delay, adds no relation. Before, restore refused the declared `k
/// qualifies foo@2` listed again beside renewable `foo` only by looking for
/// `k` on `foo`, and accepted such a repeat otherwise; the restored session
/// showed the relation twice in the snapshot of every later accepted
/// outcome. A repeat of one a declaration gives a claim meets the kind check
/// first.
#[test]
fn a_declared_qualification_listed_again_is_refused() {
    for (declared, renewal) in [
        ("foo@2", "renewable foo limit 3;"),
        ("foo@2", ""),
        ("bar", "renewable foo limit 3;"),
    ] {
        let source = format!(
            "claim c;
claim d;
evidence foo from \"f\";
evidence {declared} from \"g\";
{declared} supports d;
caveat k consequence low;
k qualifies {declared};
{renewal}
state y = 0;
event peek;
event sy;
on peek reveal foo supports c;
on sy set y = qualified(1, foo);"
        );
        let saves = plays_and_restores(&source, &plain(&["peek", "sy"]));
        refused(
            &source,
            &with(&saves[1], ["k", "qualifies", declared]),
            &format!(
                "relation k qualifies {declared}: a declaration gives it, and no event adds it \
                 again"
            ),
        );
    }
    let source = "claim c;
claim d;
evidence bar from \"g\";
bar supports d;
caveat k consequence low;
k qualifies bar;
caveat kd consequence low;
kd qualifies d;
event qd;
event qa;
event tick dt min 0 max 0.1;
on qd qualify bar with k;
on qa qualify bar with k after 0.2;";
    let saves = plays_and_restores(source, &plain(&["qd", "qa", "tick", "tick", "tick", "qd"]));
    for save in &saves {
        assert_eq!(save["graph"]["relations"], Value::Null, "{save}");
    }
    refused(
        source,
        &with(&saves[5], ["kd", "qualifies", "d"]),
        "relation kd qualifies d: d is a claim, not evidence",
    );
}

/// Declared evidence takes none of `foo`'s records: an observation record,
/// which only a reveal of it could leave, a qualification or a withdrawal
/// `foo` takes, or a scheduled qualification of `foo`. Before, each was
/// accepted as one of `foo`'s, except the record beside a declared stance,
/// which D064's check refused; and the scheduled one, once due, qualified
/// `foo@2` where the live session qualifies `foo`. The same holds for a
/// declared `foo@1` or `foo@7`.
#[test]
fn declared_evidence_takes_none_of_foo_s_records() {
    let source = program("foo@2", LIMIT, "");
    let saves = plays_and_restores(&source, &plain(&["peek", "q", "sq", "see_why", "wd"]));
    let (peeked, scheduled, withdrawn) = (&saves[0], &saves[2], &saves[4]);
    let bare = "claim c;
evidence foo from \"f\";
evidence foo@2 from \"g\";
renewable foo limit 3;
event peek;
on peek reveal foo supports c;";
    let mut record = plays_and_restores(bare, &plain(&["peek"])).remove(0);
    record["observation_qualifications"]["foo@2"] = json!({});
    refused(
        bare,
        &record,
        "observation of foo@2: foo@2 is not observed evidence or a renewed occurrence",
    );
    // Beside its declared stance, D064's check refused this record before,
    // as a record of evidence with no stance but its declarations'; D052's
    // now refuses it first, reading `foo@2` as itself.
    let mut record = peeked.clone();
    record["observation_qualifications"]["foo@2"] = json!({});
    refused(
        &source,
        &record,
        "observation of foo@2: no rule or procedure of this program reveals foo@2",
    );
    let mut later = scheduled.clone();
    later["scheduled_qualifications"][0]["evidence"] = json!("foo@2");
    refused(
        &source,
        &later,
        "scheduled qualification of foo@2 with late: no qualify ... after of this program \
         schedules it",
    );
    let mut withdrawal = with(withdrawn, ["withdrawn", "qualifies", "foo@2"]);
    withdrawal["withdrawals"]
        .as_array_mut()
        .unwrap()
        .push(json!({"evidence": "foo@2", "because": "why", "event": "wd", "sequence": 5}));
    refused(
        &source,
        &withdrawal,
        "relation withdrawn qualifies foo@2: no rule, declaration, reading, renewal or \
         withdrawal of this program qualifies foo@2 with withdrawn",
    );
    for declared in ["foo@1", "foo@2", "foo@7"] {
        let source = program(declared, LIMIT, "");
        let saves = plays_and_restores(&source, &plain(&["peek", "q"]));
        refused(
            &source,
            &with(&saves[1], ["k", "qualifies", declared]),
            &format!(
                "relation k qualifies {declared}: no rule, declaration, reading, renewal or \
                 withdrawal of this program qualifies {declared} with k"
            ),
        );
    }
}

/// Declared evidence carries its own records, not `foo`'s. The flows that
/// carry `foo@2` into `x`, through `hold`'s reopening, do not carry `foo`,
/// and those that carry `foo` into `y`, through `observed(foo)`, do not
/// carry `foo@2`; a caveat that can qualify `foo` cannot qualify `foo@2`.
/// Before, restore read `foo@2` as `foo` on both sides, and accepted each.
#[test]
fn declared_evidence_carries_its_own_records() {
    let source = program("foo@2", LIMIT, "");
    let saves = plays_and_restores(&source, &plain(&["peek", "q", "sx", "sy"]));
    let [reopened, observed] = [&saves[2], &saves[3]];
    let mut foo = reopened.clone();
    foo["states"]["x"]["lineage"] = json!({"evidence": ["foo"]});
    refused(
        &source,
        &foo,
        "state x: no rule or declaration of this program carries records of foo into it",
    );
    assert_eq!(
        observed["states"]["y"]["lineage"]["evidence"],
        json!(["foo"])
    );
    let mut declared = observed.clone();
    declared["states"]["y"]["lineage"] = json!({"evidence": ["foo@2"]});
    refused(
        &source,
        &declared,
        "state y: no rule or declaration of this program carries records of foo@2 into it",
    );
    let mut caveat = reopened.clone();
    caveat["states"]["x"]["lineage"] = json!({"caveats": ["k"], "evidence": ["foo@2"]});
    refused(
        &source,
        &caveat,
        "state x: k cannot qualify any of its evidence",
    );
}

/// What the last event reported reads a declared name as itself too, with
/// or without a renewal: `look` revealed `foo`, not `foo@2`, and `q`
/// qualified `foo`. And a declared name is no departed record of `foo`,
/// whether the last event's report names it or not.
#[test]
fn reports_name_declared_evidence_as_itself() {
    for renewal in ["", "renewable foo limit 3;"] {
        let source = format!(
            "claim c;
claim d;
evidence foo from \"f\";
evidence foo@2 from \"g\";
foo@2 supports d;
caveat k consequence low;
k qualifies foo@2;
{renewal}
event look;
event q;
on look reveal foo;
on q qualify foo with k;"
        );
        let saves = plays_and_restores(&source, &plain(&["look", "q"]));
        let mut revealed = saves[0].clone();
        revealed["effects"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind": "reveal", "evidence": "foo@2"}));
        refused(
            &source,
            &revealed,
            "neutral reveal effect names unreachable evidence foo@2",
        );
        let mut qualified = saves[1].clone();
        qualified["effects"][0]["evidence"] = json!("foo@2");
        refused(
            &source,
            &qualified,
            "qualify effect is not one the last event can make",
        );
    }
    let source = program("foo@2", "renewable foo window 1", "");
    let saves = plays_and_restores(&source, &plain(&["peek", "peek", "peek"]));
    let mut departed = saves[2].clone();
    departed["graph"]["nodes"] = json!([{"kind": "occurrence", "name": "foo@3"}]);
    departed["renewals"] = json!({"foo": ["foo", "foo@3"]});
    departed["retired"] = json!({"foo": 1});
    refused(
        &source,
        &departed,
        "renewable foo occurrence foo@3: no renewal gets past foo@2, which the program declares",
    );
    departed["effects"] = json!([{"kind": "depart", "history": "foo", "record": "foo@2"}]);
    refused(
        &source,
        &departed,
        "depart effect names no departed record of its history",
    );
}

/// The new checks run last, so a save an older check refuses keeps that
/// check's message even where they would refuse it too: a saved occurrence
/// node named like the declared evidence is refused as before, since the
/// name is taken; a renewals list naming it, of a program with no `renew`
/// (D063), or beside a retirement dated at the last event, which did not
/// make it (D060); and a declared name of another kind in a renewals list.
#[test]
fn an_older_check_refuses_first() {
    let source = program("foo@2", LIMIT, "");
    let saves = plays_and_restores(&source, &plain(&["peek"]));
    let mut node = saves[0].clone();
    node["graph"]["nodes"] = json!([{"kind": "occurrence", "name": "foo@2"}]);
    node["renewals"] = json!({"foo": ["foo", "foo@2"]});
    refused(&source, &node, "foo@2 already exists");
    for renewal in ["renewable foo limit 3", "renewable foo window 1"] {
        let source = format!(
            "claim c;
evidence foo from \"f\";
evidence foo@2 from \"g\";
{renewal};
event peek;
on peek reveal foo supports c;"
        );
        let saves = plays_and_restores(&source, &plain(&["peek"]));
        let mut listed = saves[0].clone();
        listed["renewals"] = json!({"foo": ["foo", "foo@2"]});
        if renewal.contains("window") {
            listed["retired"] = json!({"foo": 1});
            refused(
                &source,
                &listed,
                "retired foo is dated at the last event, which did not retire it",
            );
        } else {
            refused(
                &source,
                &listed,
                "renewable foo occurrence foo@2: no rule or procedure of this program renews foo",
            );
        }
    }
    let claim = "claim c;
claim foo@2;
evidence foo from \"f\";
renewable foo limit 3;
event peek;
event refresh;
on peek reveal foo supports c;
on refresh renew foo;";
    let saves = plays_and_restores(claim, &plain(&["peek"]));
    let mut listed = saves[0].clone();
    listed["renewals"] = json!({"foo": ["foo", "foo@2"]});
    refused(claim, &listed, "foo@2 must name a declared evidence");
}

/// The journal names its entries `journal@K` whatever the program declares:
/// it has no node to take the name from. So restore reads a journal record by
/// its spelling, beside evidence declared as `journal@1` too, and a
/// retirement of entry `journal@1` dated where no entry was added is refused
/// as before.
#[test]
fn a_journal_entry_keeps_its_name() {
    let source = "claim c;
claim d0;
evidence why from \"w\";
why supports c;
evidence journal@1 from \"j\";
journal@1 supports d0;
caveat k consequence low;
k qualifies journal@1;
commit act because enough;
reopen act because journal@1;
decisions d limit 4;
journal window 1;
state s = 0;
state t = 0;
event sets;
event cd;
event tt;
event sel;
on sets set s = if(reopened(act), 1, 0);
on cd commit d because enough using 1;
on tt set t = 1;
on sel resolve d = 1 because caveated(s, k);";
    let saves = plays_and_restores(source, &plain(&["sets", "cd", "tt", "sel", "tt"]));
    assert_eq!(saves[4]["retired"], json!({"journal@1": 4}));
    for at in [2, 3] {
        let mut dated = saves[4].clone();
        dated["retired"]["journal@1"] = json!(at);
        refused(
            source,
            &dated,
            "retired journal@1 is not dated at the event that added journal@2",
        );
    }
}
