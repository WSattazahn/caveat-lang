//! D051: restore refused the save a session wrote once the evidence a
//! declaration reopened a commitment with had retired before any event
//! observed it: "cannot restore save: state seen: sensor is not observed".
//! `reopen hold because sensor;` cites `sensor` as the program loads, a
//! renewal can retire it while it is still unobserved, and once it has
//! retired `reopened(hold)` reads it as other retired evidence is read
//! (D045). What that read makes carries it: a lineage or grounds, a
//! commitment's basis with its `relies_on` relations and journal entry, a
//! reading, an outcome, and the qualifications records keep. Restore
//! required each of them to be observed. It now accepts a retired cause a
//! declaration's `reopen` names that no event observed wherever a record
//! cites it, and in a reading, a commitment's basis, an outcome or a
//! reopening's record only from the event the save says retired it, so each
//! such save restores to the same save and the restored session answers the
//! next event as the live one does. What an event names itself, such as a
//! withdrawal's reason or the key of an observation record, must still be
//! observed, a cause an event did observe is cited as other observed
//! evidence is, and a forged save that breaks either rule, or that dates
//! the retirement after such a record, is still refused. See
//! spec/caveat-windows-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// `hold` is declared committed and reopened because of `sensor`, which
/// `regrow` retires as it renews it. A rule that names `sensor` after that
/// names the new occurrence, so `see` then observes sensor@2, never the
/// declared cause. `seee` and `seef` observe `e` and `f` with a stance;
/// `seeg` observes `g` without one, which starts the observation ledger.
fn program(rules: &str) -> String {
    format!(
        "claim safe;\n\
         caveat calibration consequence low;\n\
         caveat stale consequence low;\n\
         evidence sensor from \"sensor\";\n\
         evidence e from \"e\";\n\
         evidence f from \"f\";\n\
         evidence g from \"g\";\n\
         calibration qualifies sensor;\n\
         stale qualifies e;\n\
         renewable sensor window 1;\n\
         commit hold because enough;\n\
         reopen hold because sensor;\n\
         state seen = 0;\n\
         event go;\n\
         event see;\n\
         event seee;\n\
         event seef;\n\
         event seeg;\n\
         event dec;\n\
         event regrow;\n\
         on see reveal sensor supports safe;\n\
         on seee reveal e supports safe;\n\
         on seef reveal f supports safe;\n\
         on seeg reveal g;\n\
         on regrow renew sensor;\n\
         {rules}\n"
    )
}

fn answer(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

fn send(session: &mut ReactiveSession, event: &str) {
    let outcome = answer(session, event);
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
}

/// Whether an event observed `name`: by a stance-bearing relation, in the
/// ledger, or with an observation record.
fn observed(save: &Value, name: &str) -> bool {
    let stance = save["graph"]["relations"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|relation| {
            relation[0] == name && (relation[1] == "supports" || relation[1] == "opposes")
        });
    let ledger = save["observations"]
        .as_array()
        .is_some_and(|names| names.contains(&json!(name)));
    stance || ledger || save["observation_qualifications"].get(name).is_some()
}

/// The save `rules` and `events` leave, checked to cite `sensor` where
/// `cites` says (a JSON pointer to a list, and the item in it), though
/// `sensor` has retired unobserved. It restores to the same save, and the
/// restored session answers the next `go` as the live one does.
fn restores(rules: &str, events: &[&str], cites: &[(&str, &str)]) -> Result<Value, String> {
    let source = program(rules);
    let mut live = ReactiveSession::from_source(&source).unwrap();
    for event in events {
        send(&mut live, event);
    }
    let text = live.save_json().unwrap();
    let save: Value = serde_json::from_str(&text).unwrap();
    assert!(save["retired"]["sensor"].is_u64(), "{save}");
    assert!(!observed(&save, "sensor"), "{save}");
    for (pointer, item) in cites {
        let item: Value = serde_json::from_str(item).unwrap();
        let list = save.pointer(pointer).and_then(Value::as_array);
        assert!(
            list.is_some_and(|list| list.contains(&item)),
            "{pointer} does not hold {item}: {save}"
        );
    }
    let mut restored = ReactiveSession::restore_json(&source, &text)?;
    assert_eq!(restored.save_json().unwrap(), text, "{rules}");
    let next = answer(&mut live, "go");
    assert_eq!(answer(&mut restored, "go"), next, "{rules}");
    assert_eq!(
        restored.save_json().unwrap(),
        live.save_json().unwrap(),
        "{rules}"
    );
    Ok(save)
}

const SENSOR: &str = "\"sensor\"";
const OTHER_RELIES: &str = r#"["other","relies_on","sensor"]"#;
const LINEAGE: &str = "/states/seen/lineage/evidence";

#[test]
fn the_reproduction_restores() {
    let rules = "on go when reopened(hold) set seen = seen + 10;";
    for events in [
        &["regrow", "go"][..],
        &["regrow", "see", "go"],
        &["regrow", "regrow", "go"],
    ] {
        let save = restores(rules, events, &[(LINEAGE, SENSOR)])
            .unwrap_or_else(|error| panic!("{events:?}: {error}"));
        assert_eq!(save["states"]["seen"]["value"], 10.0, "{events:?}");
    }
    // The control: `see` observes the declared cause before `regrow`
    // retires it, so it was observed when it was cited, and this restored
    // before.
    let source = program(rules);
    let mut live = ReactiveSession::from_source(&source).unwrap();
    for event in ["see", "regrow", "go"] {
        send(&mut live, event);
    }
    let text = live.save_json().unwrap();
    let save: Value = serde_json::from_str(&text).unwrap();
    assert!(observed(&save, "sensor"), "{save}");
    assert_eq!(
        save["states"]["seen"]["lineage"]["evidence"],
        json!(["sensor"])
    );
    let mut restored = ReactiveSession::restore_json(&source, &text).unwrap();
    assert_eq!(restored.save_json().unwrap(), text);
    assert_eq!(answer(&mut restored, "go"), answer(&mut live, "go"));
}

/// Where a save cites the cause: a JSON pointer to a list, and the item in it.
type Cites = &'static [(&'static str, &'static str)];

/// A record the cause can be left in: what it is, the rules that read
/// `reopened(hold)`, the events that lead there, and where the save cites it.
type Path = (&'static str, &'static str, &'static [&'static str], Cites);

/// Each record a read of `reopened(hold)` can leave the cause in.
const PATHS: &[Path] = &[
    (
        "a state's lineage, from a value",
        "on go set seen = if(reopened(hold), 1, 2);\n\
         on go when has_caveat(seen, calibration) set seen = seen + 100;",
        &["regrow", "go"],
        &[(LINEAGE, SENSOR)],
    ),
    (
        "a state's grounds",
        "on go set seen = qualified(if(reopened(hold), 1, 2), e) because reopened(hold);",
        &["regrow", "seee", "go"],
        &[("/states/seen/grounds/evidence", SENSOR)],
    ),
    (
        "a skipped set's guard",
        "on go when not reopened(hold) set seen = 1;",
        &["regrow", "go"],
        &[(LINEAGE, SENSOR)],
    ),
    (
        "a basis, its relies_on relation, grounds and journal entry",
        "on go when not committed(other) commit other because enough \
         using if(reopened(hold), 1, 2);",
        &["regrow", "go"],
        &[
            ("/graph/relations", OTHER_RELIES),
            ("/commitment_bases/other/provenance/evidence", SENSOR),
            ("/commitment_grounds/other/evidence", SENSOR),
            ("/decision_journal/0/because", SENSOR),
        ],
    ),
    (
        "a commitment's cited grounds",
        "on go when not committed(other) commit other because enough \
         citing if(reopened(hold), 1, 2);",
        &["regrow", "go"],
        &[
            ("/graph/relations", OTHER_RELIES),
            ("/commitment_grounds/other/evidence", SENSOR),
            ("/decision_journal/0/because", SENSOR),
        ],
    ),
    (
        "a commit's guard",
        "on go when reopened(hold) and not committed(other) commit other \
         because enough using 1;",
        &["regrow", "go"],
        &[
            ("/graph/relations", OTHER_RELIES),
            ("/commitment_bases/other/provenance/evidence", SENSOR),
        ],
    ),
    (
        "a journal entry after observed evidence",
        "on go when not committed(other) commit other because enough \
         using if(reopened(hold), 1, 2) retaining calibration because sensor;",
        &["regrow", "see", "go"],
        &[
            ("/graph/relations", OTHER_RELIES),
            ("/decision_journal/0/because", SENSOR),
        ],
    ),
    (
        "a journal entry after evidence with a stance",
        "on go when not committed(other) commit other because enough \
         using qualified(1, e) + if(reopened(hold), 1, 2);",
        &["regrow", "seee", "go"],
        &[("/decision_journal/0/because", SENSOR)],
    ),
    (
        "a journal entry after evidence in the ledger",
        "on go when not committed(other) commit other because enough \
         using qualified(1, g) + if(reopened(hold), 1, 2);",
        &["seeg", "regrow", "go"],
        &[("/decision_journal/0/because", SENSOR)],
    ),
    (
        "a permitted commit",
        "on go when reopened(hold) and not committed(other) commit other \
         because enough using 1 permitted by e;",
        &["regrow", "seee", "go"],
        &[("/graph/relations", OTHER_RELIES)],
    ),
    (
        "a commit with a retention reason",
        "on go when reopened(hold) and not committed(other) commit other \
         because enough using qualified(1, e) retaining stale because e;",
        &["regrow", "seee", "go"],
        &[("/graph/relations", OTHER_RELIES)],
    ),
    (
        "the bases of a decision series",
        "decisions d limit 3;\n\
         on dec when reopened(hold) and not committed(d) commit d \
         because enough using 1;\n\
         on go when committed(d) and not reopened(d) reopen d because e;\n\
         on go when reopened(d) commit d because enough using qualified(2, e);",
        &["regrow", "seee", "dec", "go", "go"],
        &[
            ("/graph/relations", r#"["d@1","relies_on","sensor"]"#),
            ("/graph/relations", r#"["d@3","relies_on","sensor"]"#),
        ],
    ),
    (
        "an outcome's lineage",
        "decisions d limit 2;\n\
         event judge;\n\
         on dec commit d because enough using 1;\n\
         on judge resolve d = if(reopened(hold), 1, 2) because e;",
        &["regrow", "seee", "dec", "judge"],
        &[(
            "/decision_series/d/revisions/0/outcome/lineage/evidence",
            SENSOR,
        )],
    ),
    (
        "an outcome of a revision made before the renewal",
        "decisions d limit 2;\n\
         event judge;\n\
         on dec commit d because enough using 1;\n\
         on judge resolve d = if(reopened(hold), 1, 2) because e;",
        &["seee", "dec", "regrow", "judge"],
        &[(
            "/decision_series/d/revisions/0/outcome/lineage/evidence",
            SENSOR,
        )],
    ),
    (
        "a skipped commit's predicate qualification",
        "on go when not reopened(hold) commit other because enough;",
        &["regrow", "go"],
        &[("/predicate_qualifications/committed/other/evidence", SENSOR)],
    ),
    (
        "a skipped reveal's predicate qualification",
        "on go when not reopened(hold) reveal e supports safe;",
        &["regrow", "go"],
        &[("/predicate_qualifications/observed/e/evidence", SENSOR)],
    ),
    (
        "a skipped examination's predicate qualification",
        "budget 10;\non go when not reopened(hold) examine stale cost 1;",
        &["regrow", "go"],
        &[("/predicate_qualifications/examined/stale/evidence", SENSOR)],
    ),
    (
        "a skipped reopening's predicate qualification",
        "on dec commit other because enough using qualified(1, e);\n\
         on go when not reopened(hold) reopen other because e;",
        &["regrow", "seee", "dec", "go"],
        &[("/predicate_qualifications/reopened/other/evidence", SENSOR)],
    ),
    (
        "a skipped withdrawal's predicate qualification",
        "on go when not reopened(hold) withdraw e because f;",
        &["regrow", "seee", "seef", "go"],
        &[("/predicate_qualifications/withdrawn/e/evidence", SENSOR)],
    ),
    (
        "a reading and its observation record",
        "evidence feed from \"feed\";\n\
         readings runs from feed window 3;\n\
         on go when reopened(hold) sample runs = 1 supports safe;",
        &["regrow", "go"],
        &[
            (
                "/reading_streams/runs/occurrences/0/provenance/evidence",
                SENSOR,
            ),
            ("/observation_qualifications/runs@1/evidence", SENSOR),
        ],
    ),
    (
        "a reading in the observation ledger",
        "evidence feed from \"feed\";\n\
         readings runs from feed window 3;\n\
         on go when reopened(hold) sample runs = 1 supports safe;",
        &["regrow", "seeg", "go"],
        &[(
            "/reading_streams/runs/occurrences/0/provenance/evidence",
            SENSOR,
        )],
    ),
    (
        "a reading in the ledger after an observation before the renewal",
        "evidence feed from \"feed\";\n\
         readings runs from feed window 3;\n\
         on go when reopened(hold) sample runs = 1 supports safe;",
        &["seeg", "regrow", "go"],
        &[(
            "/reading_streams/runs/occurrences/0/provenance/evidence",
            SENSOR,
        )],
    ),
    (
        "a reading in the event that retires the cause",
        "evidence feed from \"feed\";\n\
         readings runs from feed window 3;\n\
         on go renew sensor;\n\
         on go when reopened(hold) sample runs = 1 supports safe;",
        &["seeg", "go"],
        &[(
            "/reading_streams/runs/occurrences/0/provenance/evidence",
            SENSOR,
        )],
    ),
    (
        "a skipped sample's stream selection",
        "evidence feed from \"feed\";\n\
         readings runs from feed window 3;\n\
         on go when not reopened(hold) sample runs = 1 supports safe;",
        &["regrow", "go"],
        &[(
            "/reading_streams/runs/selection_qualifications/evidence",
            SENSOR,
        )],
    ),
    (
        "a skipped commit's series selection",
        "decisions d limit 2;\non go when not reopened(hold) commit d because enough;",
        &["regrow", "go"],
        &[(
            "/decision_series/d/selection_qualifications/evidence",
            SENSOR,
        )],
    ),
    (
        "a scheduled qualification's guard",
        "event tick dt min 0 max 0.1;\n\
         clock tick every 0.1;\n\
         on go when reopened(hold) qualify e with stale after 0.2;",
        &["regrow", "seee", "go"],
        &[("/scheduled_qualifications/0/guard/evidence", SENSOR)],
    ),
    (
        "an observation's qualification",
        "on go when reopened(hold) reveal e supports safe;",
        &["regrow", "go"],
        &[("/observation_qualifications/e/evidence", SENSOR)],
    ),
    (
        "an examination's qualification",
        "budget 10;\non go when reopened(hold) examine stale cost 1;",
        &["regrow", "go"],
        &[("/examination_qualifications/stale/evidence", SENSOR)],
    ),
    (
        "a reopening's qualification",
        "on dec commit other because enough using qualified(1, e);\n\
         on go when reopened(hold) and committed(other) and not reopened(other) \
         reopen other because e;",
        &["regrow", "seee", "dec", "go"],
        &[("/reopening_qualifications/other/evidence", SENSOR)],
    ),
    (
        "a hold's qualification",
        "decisions d limit 2;\n\
         on dec commit d because enough using qualified(1, e);\n\
         on go when reopened(hold) and committed(d) hold d because e;",
        &["regrow", "seee", "dec", "go"],
        &[("/reopening_qualifications/d@1/evidence", SENSOR)],
    ),
    (
        "a deferral's qualification, after a hold made before the renewal",
        "decisions d limit 2;\n\
         event pause;\n\
         on dec commit d because enough using qualified(1, e);\n\
         on pause hold d because e;\n\
         on go when reopened(hold) and committed(d) reopen d because f;",
        &["seee", "seef", "dec", "pause", "regrow", "go"],
        &[
            ("/reopening_qualifications/d@1/evidence", SENSOR),
            ("/decision_series/d/deferred", "\"f\""),
        ],
    ),
    (
        "a release's qualification, after a hold made before the renewal",
        "decisions d limit 2;\n\
         event pause;\n\
         on dec commit d because enough using qualified(1, e);\n\
         on pause hold d because e;\n\
         on go when reopened(hold) and committed(d) release d because f;",
        &["seee", "seef", "dec", "pause", "regrow", "go"],
        &[
            ("/reopening_qualifications/d@1/evidence", SENSOR),
            ("/decision_journal/2/because", "\"f\""),
        ],
    ),
    (
        "a reopening's qualification once its journal entries have departed",
        "journal window 1;\n\
         on dec commit other because enough using qualified(1, e);\n\
         on go when reopened(hold) and committed(other) and not reopened(other) \
         reopen other because e;\n\
         on seef commit third because enough using qualified(1, f);",
        &["regrow", "seee", "dec", "go", "seef"],
        &[
            ("/reopening_qualifications/other/evidence", SENSOR),
            ("/decision_journal/0/because", "\"f\""),
        ],
    ),
    (
        "a cue's qualification",
        "cue alert toast \"x\" 1;\non go when reopened(hold) emit alert;",
        &["regrow", "go"],
        &[("/cue_qualifications/0/evidence", SENSOR)],
    ),
];

#[test]
fn each_record_that_cites_the_cause_restores() {
    // Every path is tried, so a run that fails names each one that does.
    let refused = PATHS
        .iter()
        .filter_map(|(what, rules, events, cites)| {
            restores(rules, events, cites)
                .err()
                .map(|error| format!("{what}: {error}"))
        })
        .collect::<Vec<_>>();
    assert!(refused.is_empty(), "\n{}", refused.join("\n"));
}

fn saved(source: &str, events: &[&str]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    for event in events {
        send(&mut live, event);
    }
    serde_json::from_str(&live.save_json().unwrap()).unwrap()
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted {save}"),
    }
}

/// Sets what `pointer` names in `save` to `value`: a field, added if the
/// save leaves it out, or an item of a list, where `-` appends one.
fn set(save: &mut Value, pointer: &str, value: Value) {
    let missing = format!("{pointer} is not in {save}");
    let (parent, last) = pointer.rsplit_once('/').expect(&missing);
    match save.pointer_mut(parent).expect(&missing) {
        Value::Object(fields) => {
            fields.insert(last.to_string(), value);
        }
        Value::Array(items) if last == "-" => items.push(value),
        Value::Array(items) => items[last.parse::<usize>().expect(&missing)] = value,
        _ => panic!("{missing}"),
    }
}

const HELD: &str = "decisions d limit 2;\n\
                    event pause;\n\
                    on dec commit d because enough using qualified(1, e);\n\
                    on pause hold d because e;\n\
                    on go when committed(d) and not reopened(d) reopen d because e;";

#[test]
fn what_an_event_names_must_still_be_observed() {
    // Each save restores as written; naming the retired cause where an
    // event names evidence itself is refused as it was before (D051).
    let cases: [(&str, &[&str], &str, Value, &str); 11] = [
        (
            "on go withdraw e because f;",
            &["regrow", "seee", "seef", "go"],
            "/withdrawals/0/because",
            json!("sensor"),
            "the withdrawal of e: sensor is not observed",
        ),
        (
            "",
            &["regrow"],
            "/observation_qualifications",
            json!({"sensor": {}}),
            "observation of sensor: sensor is not observed evidence or a renewed occurrence",
        ),
        (
            "on dec commit other because enough using qualified(1, e);",
            &["regrow", "seee", "dec"],
            "/graph/relations/-",
            json!(["sensor", "reopens", "other"]),
            "relation sensor reopens other: sensor is not observed",
        ),
        (
            HELD,
            &["regrow", "seee", "dec", "pause"],
            "/decision_series/d/on_hold",
            json!(["sensor"]),
            "sensor is not observed",
        ),
        (
            HELD,
            &["regrow", "seee", "dec", "pause"],
            "/decision_journal/1/because",
            json!(["sensor"]),
            "decision journal: sensor is not observed",
        ),
        (
            HELD,
            &["regrow", "seee", "dec", "pause", "go"],
            "/decision_series/d/deferred",
            json!(["sensor"]),
            "sensor is not observed",
        ),
        (
            "on go when not committed(other) commit other because enough \
             using 1 permitted by e;",
            &["regrow", "seee", "go"],
            "/commitment_permissions/other/grant",
            json!("sensor"),
            "the permission of other names a grant that was never observed, sensor",
        ),
        (
            "on go when not committed(other) commit other because enough \
             using qualified(1, e) retaining stale because e;",
            &["regrow", "seee", "go"],
            "/retention_reasons/other/stale",
            json!("sensor"),
            "the reason for stale on other, sensor, is not observed evidence in its grounds",
        ),
        (
            "decisions d limit 2;\n\
             event judge;\n\
             on dec commit d because enough using 1;\n\
             on judge resolve d = 1 because e;",
            &["regrow", "seee", "dec", "judge"],
            "/decision_series/d/revisions/0/outcome/because",
            json!(["sensor"]),
            "sensor is not observed",
        ),
        (
            "event tick dt min 0 max 0.1;\n\
             clock tick every 0.1;\n\
             on go qualify e with stale after 0.2;",
            &["regrow", "seee", "go"],
            "/scheduled_qualifications/0/evidence",
            json!("sensor"),
            "sensor is not observed",
        ),
        (
            "",
            &["regrow", "seee"],
            "/effects/0/evidence",
            json!("sensor"),
            "sensor is not observed",
        ),
    ];
    for (rules, events, pointer, value, expected) in cases {
        let source = program(rules);
        let save = saved(&source, events);
        assert!(save["retired"]["sensor"].is_u64(), "{save}");
        ReactiveSession::restore_json(&source, &save.to_string())
            .unwrap_or_else(|error| panic!("{pointer}: {error}"));
        let mut forged = save.clone();
        set(&mut forged, pointer, value);
        refused(&source, &forged, expected);
    }
}

#[test]
fn a_record_cites_only_a_declared_cause_and_only_once_it_has_retired() {
    // Without the declaration, nothing reopens with `sensor`, and a lineage
    // that cites it once it has retired unobserved is refused as before.
    let source =
        program("on go set seen = seen + 10;").replace("reopen hold because sensor;\n", "");
    assert!(!source.contains("reopen hold"));
    let mut forged = saved(&source, &["regrow", "go"]);
    set(
        &mut forged,
        "/states/seen/lineage",
        json!({"caveats": ["calibration"], "evidence": ["sensor"]}),
    );
    refused(&source, &forged, "state seen: sensor is not observed");
    // Nor before it has retired: until then reading `reopened(hold)` is
    // refused (D045), and a lineage that cites the cause is refused as before.
    let source = program("on go when reopened(hold) set seen = seen + 10;");
    let mut forged = saved(&source, &["dec"]);
    assert!(forged.get("retired").is_none(), "{forged}");
    set(
        &mut forged,
        "/states/seen",
        json!({
            "lineage": {"caveats": ["calibration"], "evidence": ["sensor"]},
            "value": 10.0,
        }),
    );
    refused(&source, &forged, "state seen: sensor is not observed");
    // A reading in the ledger cites the cause only once it has retired: a
    // save that has it retire after the reading is refused as before.
    let source = program(
        "evidence feed from \"feed\";\n\
         readings runs from feed window 3;\n\
         on go when reopened(hold) sample runs = 1 supports safe;",
    );
    let mut forged = saved(&source, &["regrow", "seeg", "go", "dec"]);
    assert_eq!(
        forged["reading_streams"]["runs"]["occurrences"][0]["sequence"],
        3
    );
    set(&mut forged, "/retired/sensor", json!(4));
    refused(
        &source,
        &forged,
        "reading runs@1 cites evidence not yet observed: sensor",
    );
    // A cause a rule's `reopen` names was observed when the rule cited it,
    // with a stance or by a neutral reveal, so a reading in the ledger
    // cites it only once the ledger observes it, retired or not: a save
    // whose ledger observes it after the reading, or leaves it out, is
    // refused as before.
    let cases = [
        (
            "e",
            &["seeg", "seee", "dec", "go", "again", "read"][..],
            json!(["g", "e", "runs@1"]),
            json!(["g", "runs@1", "e"]),
        ),
        (
            "g",
            &["seeg", "dec", "go", "again", "read"],
            json!(["g", "runs@1"]),
            json!(["runs@1"]),
        ),
    ];
    for (cause, events, ledger, forged_ledger) in cases {
        let source = program(&format!(
            "evidence feed from \"feed\";\n\
             readings runs from feed window 3;\n\
             renewable {cause} window 1;\n\
             event again;\n\
             event read;\n\
             on dec commit other because enough using 1;\n\
             on go when committed(other) and not reopened(other) \
             reopen other because {cause};\n\
             on again renew {cause};\n\
             on read when reopened(other) sample runs = 1 supports safe;"
        ));
        let save = saved(&source, events);
        assert!(save["retired"][cause].is_u64(), "{save}");
        assert_eq!(save["observations"], ledger, "{save}");
        let cites = save.pointer("/reading_streams/runs/occurrences/0/provenance/evidence");
        assert!(
            cites
                .and_then(Value::as_array)
                .is_some_and(|cites| cites.contains(&json!(cause))),
            "{save}"
        );
        ReactiveSession::restore_json(&source, &save.to_string()).unwrap();
        let mut forged = save.clone();
        set(&mut forged, "/observations", forged_ledger);
        refused(
            &source,
            &forged,
            &format!("reading runs@1 cites evidence not yet observed: {cause}"),
        );
    }
}

/// A stream `go` samples once `reopened(hold)` reads, with a stance.
const READINGS: &str = "evidence feed from \"feed\";\n\
                        readings runs from feed window 3;\n\
                        on go when reopened(hold) sample runs = 1 supports safe;";

#[test]
fn a_ledger_that_holds_the_declared_cause_orders_it() {
    // An event can observe the declared cause before the renewal retires
    // it: a neutral reveal, or a stance in a session that keeps the
    // observation ledger. The ledger then holds it, and a reading cites it
    // only once the ledger observes it, as other evidence: a save that
    // moves it after the reading is refused as before.
    let source = program(&format!("event seen;\non seen reveal sensor;\n{READINGS}"));
    for (events, ledger, moved) in [
        (
            &["seen", "regrow", "go"][..],
            json!(["sensor", "runs@1"]),
            json!(["runs@1", "sensor"]),
        ),
        (
            &["seeg", "see", "regrow", "go"],
            json!(["g", "sensor", "runs@1"]),
            json!(["g", "runs@1", "sensor"]),
        ),
    ] {
        let save = saved(&source, events);
        assert_eq!(save["observations"], ledger, "{events:?}: {save}");
        assert_eq!(
            save["reading_streams"]["runs"]["occurrences"][0]["provenance"]["evidence"],
            json!(["runs@1", "sensor"]),
            "{events:?}: {save}"
        );
        ReactiveSession::restore_json(&source, &save.to_string())
            .unwrap_or_else(|error| panic!("{events:?}: {error}"));
        let mut forged = save.clone();
        set(&mut forged, "/observations", moved);
        refused(
            &source,
            &forged,
            "reading runs@1 cites evidence not yet observed: sensor",
        );
    }
}

#[test]
fn a_record_made_before_an_observed_cause_retires_restores() {
    // Once an event observes the declared cause, with a stance or a neutral
    // reveal, `reopened(hold)` may be read before the renewal retires it.
    // A reading, a commitment's basis and an outcome made then cite it as
    // other observed evidence, and restore does not date them by its
    // retirement.
    let source = program(&format!(
        "event seen;\n\
         event judge;\n\
         decisions d limit 2;\n\
         on seen reveal sensor;\n\
         on dec when not committed(other) commit other because enough \
         using if(reopened(hold), 1, 2);\n\
         on dec when not committed(d) commit d because enough using qualified(1, e);\n\
         on judge resolve d = if(reopened(hold), 1, 2) because e;\n\
         {READINGS}"
    ));
    for observe in ["see", "seen"] {
        let mut live = ReactiveSession::from_source(&source).unwrap();
        for event in [observe, "seee", "go", "dec", "judge", "regrow"] {
            send(&mut live, event);
        }
        let text = live.save_json().unwrap();
        let save: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(save["retired"]["sensor"], 6, "{observe}: {save}");
        for pointer in [
            "/reading_streams/runs/occurrences/0/provenance/evidence",
            "/commitment_bases/other/provenance/evidence",
            "/decision_series/d/revisions/0/outcome/lineage/evidence",
        ] {
            let list = save.pointer(pointer).and_then(Value::as_array);
            assert!(
                list.is_some_and(|list| list.contains(&json!("sensor"))),
                "{observe}: {pointer} does not hold sensor: {save}"
            );
        }
        let mut restored = ReactiveSession::restore_json(&source, &text)
            .unwrap_or_else(|error| panic!("{observe}: {error}"));
        assert_eq!(restored.save_json().unwrap(), text, "{observe}");
        assert_eq!(
            answer(&mut restored, "go"),
            answer(&mut live, "go"),
            "{observe}"
        );
    }
}

#[test]
fn a_record_the_save_dates_cites_the_cause_only_once_it_has_retired() {
    // Until the cause retires, reading `reopened(hold)` is refused (D045),
    // so a reading, a commitment's basis, an outcome or a reopening's record
    // that cites it was made at or after the event that retired it. A
    // reopening's record, which a hold, a deferral and a release join too,
    // is dated by the latest such entry the journal holds for its
    // commitment. A save whose `retired` dates the cause after such a
    // record is still refused, with or without the observation ledger.
    // Each case below has no ledger.
    let cases: [(&str, &str, &[&str], u64, &str); 9] = [
        (
            "a reading",
            READINGS,
            &["regrow", "go", "dec"],
            3,
            "reading runs@1: sensor is not observed",
        ),
        (
            "a commitment's basis, dated by its journal entry",
            "on go when not committed(other) commit other because enough \
             using if(reopened(hold), 1, 2);",
            &["regrow", "go", "dec"],
            3,
            "commitment other: sensor is not observed",
        ),
        (
            "a basis that cites the cause through the commit's guard alone",
            "on go when reopened(hold) and not committed(other) commit other \
             because enough using 1;",
            &["regrow", "go", "dec"],
            3,
            "commitment other: sensor is not observed",
        ),
        (
            "a revision's basis, dated by its revision",
            "decisions d limit 2;\n\
             on dec when not committed(d) commit d because enough \
             using if(reopened(hold), 1, 2);",
            &["regrow", "dec", "go"],
            3,
            "commitment d@1: sensor is not observed",
        ),
        (
            "a revision's basis once its journal entry has departed",
            "journal window 1;\n\
             decisions d limit 2;\n\
             on dec when not committed(d) commit d because enough \
             using if(reopened(hold), 1, 2);\n\
             on go when committed(d) and not reopened(d) reopen d because e;",
            &["regrow", "dec", "seee", "go"],
            3,
            "commitment d@1: sensor is not observed",
        ),
        (
            "an outcome",
            "decisions d limit 2;\n\
             event judge;\n\
             on dec commit d because enough using 1;\n\
             on judge resolve d = if(reopened(hold), 1, 2) because e;",
            &["regrow", "seee", "dec", "judge", "go"],
            5,
            "outcome of d@1: sensor is not observed",
        ),
        (
            "a reopening's record, dated by its journal entry",
            "on dec commit other because enough using qualified(1, e);\n\
             on go when reopened(hold) and committed(other) and not reopened(other) \
             reopen other because e;",
            &["regrow", "seee", "dec", "go", "seef"],
            5,
            "reopening of other: sensor is not observed",
        ),
        (
            "a hold's record, dated by its journal entry",
            "decisions d limit 2;\n\
             on dec commit d because enough using qualified(1, e);\n\
             on go when reopened(hold) and committed(d) hold d because e;",
            &["regrow", "seee", "dec", "go", "seef"],
            5,
            "reopening of d@1: sensor is not observed",
        ),
        (
            "a hold's record, dated by the hold's entry, not the outcome's after it",
            "decisions d limit 2;\n\
             event judge;\n\
             on dec commit d because enough using qualified(1, e);\n\
             on go when reopened(hold) and committed(d) hold d because e;\n\
             on judge resolve d = 1 because e;",
            &["regrow", "seee", "dec", "go", "judge", "seef"],
            5,
            "reopening of d@1: sensor is not observed",
        ),
    ];
    for (what, rules, events, retired, expected) in cases {
        let source = program(rules);
        let save = saved(&source, events);
        assert!(save.get("observations").is_none(), "{what}: {save}");
        assert_eq!(save["retired"]["sensor"], 1, "{what}: {save}");
        ReactiveSession::restore_json(&source, &save.to_string())
            .unwrap_or_else(|error| panic!("{what}: {error}"));
        let mut forged = save.clone();
        set(&mut forged, "/retired/sensor", json!(retired));
        refused(&source, &forged, expected);
    }
}

#[test]
fn what_an_event_names_stays_refused_where_a_record_could_cite_the_cause() {
    // In each save below a record could cite the retired cause, but what
    // an event names is checked against the observed set alone, so each
    // forgery is still refused for what the event names.
    // A withdrawal's evidence: the record names sensor@2, and the forgery
    // names the retired declared cause. The last `go` leaves no effects.
    let source = program("event wd;\non wd withdraw sensor because f;");
    let save = saved(&source, &["regrow", "see", "seef", "wd", "go"]);
    ReactiveSession::restore_json(&source, &save.to_string()).unwrap();
    let renamed = r#"["withdrawn","qualifies","sensor@2"]"#;
    assert!(save.to_string().contains(renamed), "{save}");
    let mut forged: Value = serde_json::from_str(
        &save
            .to_string()
            .replace(renamed, r#"["withdrawn","qualifies","sensor"]"#),
    )
    .unwrap();
    set(&mut forged, "/withdrawals/0/evidence", json!("sensor"));
    refused(
        &source,
        &forged,
        "the withdrawal of sensor: sensor is not observed",
    );
    // A retention reason, where the commitment's grounds do cite the cause.
    let source = program(
        "on go when not committed(other) commit other because enough \
         using if(reopened(hold), qualified(1, e), 2) retaining stale because e;",
    );
    let save = saved(&source, &["regrow", "seee", "go"]);
    assert_eq!(
        save["commitment_grounds"]["other"]["evidence"],
        json!(["e", "sensor"]),
        "{save}"
    );
    ReactiveSession::restore_json(&source, &save.to_string()).unwrap();
    let mut forged = save.clone();
    set(
        &mut forged,
        "/retention_reasons/other/stale",
        json!("sensor"),
    );
    refused(
        &source,
        &forged,
        "the reason for stale on other, sensor, is not observed evidence in its grounds",
    );
    // A reopening the last event's effects report, which the declared
    // `sensor reopens hold` relation would otherwise match.
    let source = program(
        "on go when not reopened(hold) reopen hold because sensor;\n\
         on go when reopened(hold) set seen = seen + 10;",
    );
    let save = saved(&source, &["regrow", "go"]);
    ReactiveSession::restore_json(&source, &save.to_string()).unwrap();
    let mut forged = save.clone();
    set(
        &mut forged,
        "/effects",
        json!([{"kind": "reopen", "action": "hold", "because": "sensor"}]),
    );
    refused(&source, &forged, "reopen effect sensor is not observed");
}
