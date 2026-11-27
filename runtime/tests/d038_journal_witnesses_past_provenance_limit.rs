//! D038: restore refused a save the runtime itself wrote, with "cannot
//! restore save: value provenance exceeds limit ...", once a decision journal
//! entry's witnesses came to more names than one lineage may hold (1,024
//! identifiers, or 65,536 bytes of names). Restore checked each entry as one
//! lineage, and checked the caveats an entry other than a commit cites against
//! what reaches each of its causes, taken-back relations counted, as one
//! lineage per cause. Neither is a lineage. A reopening that ends a hold, or
//! breaks through one, cites every deferred cause and the caveats of each, and
//! a cause can gain caveats after the entry, or have had some taken back
//! before it, that no value reads. So every event was accepted and saved,
//! and every save after it was refused until the entry left the journal.
//! Restore now checks those names without the limits a lineage keeps to. See
//! spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::{check_save_with, CheckOptions, ReactiveSession};
use serde_json::{json, Value};

const RECORDS: CheckOptions = CheckOptions {
    unused: false,
    records: true,
    names: false,
    withdrawals: false,
};

fn outcome(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

/// `accepted`, or the code a refusal gives.
fn answer(outcome: &Value) -> &str {
    outcome["code"]
        .as_str()
        .or_else(|| outcome["outcome"].as_str())
        .unwrap()
}

/// Plays `events` on `source`, each accepted, and returns the session.
fn played(source: &str, events: &[&str]) -> ReactiveSession {
    let mut session = ReactiveSession::from_source(source).unwrap();
    for event in events {
        let outcome = outcome(&mut session, event);
        assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    }
    session
}

/// What `restores_and_plays_on` found.
struct Restored {
    /// The journal the save holds.
    journal: Vec<Value>,
    /// How the live session answered each of the next events, and the
    /// restored session the same: `accepted` or a refusal's code.
    next: Vec<String>,
    /// The journal entries that departed into the archive during the next
    /// events, the same in both.
    departed: Vec<Value>,
}

fn as_json<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap()
}

/// The journal entries among drained archive items.
fn journal_entries(items: &Value) -> impl Iterator<Item = &Value> {
    items
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item.get("journal_entry"))
}

/// Plays `events` on `source`, drains its archive (a restored session's
/// starts empty) and checks that the session's save restores, as JSON and as
/// a struct, to the same save, snapshot and views, and that the records
/// check of `caveat check --save` accepts it. Then the restored session must
/// answer each of `next` as the live one does, with the same views and the
/// same archive after each, and save the same, a save that restores in turn.
fn restores_and_plays_on(source: &str, events: &[&str], next: &[&str]) -> Restored {
    let mut live = played(source, events);
    live.drain_archive();
    let saved = live.save_json().unwrap();
    let mut restored = ReactiveSession::restore_json(source, &saved)
        .unwrap_or_else(|error| panic!("restore_json refused the runtime's own save: {error}"));
    assert_eq!(
        restored.save_json().unwrap(),
        saved,
        "save after restore_json"
    );
    assert_eq!(
        restored.snapshot(),
        live.snapshot(),
        "snapshot after restore_json"
    );
    assert_eq!(
        as_json(&restored.view()),
        as_json(&live.view()),
        "view after restore_json"
    );
    assert_eq!(
        as_json(&restored.view_v2()),
        as_json(&live.view_v2()),
        "View 0.2 after restore_json"
    );
    let structured = ReactiveSession::restore(source, &live.save().unwrap())
        .unwrap_or_else(|error| panic!("restore refused the runtime's own save: {error}"));
    assert_eq!(structured.save_json().unwrap(), saved, "save after restore");
    check_save_with(source, &saved, RECORDS)
        .unwrap_or_else(|error| panic!("the records check refused the save: {error}"));

    let mut answers = Vec::new();
    let mut departed = Vec::new();
    for event in next {
        let live_answer = outcome(&mut live, event);
        assert_eq!(
            outcome(&mut restored, event),
            live_answer,
            "{event} after restore"
        );
        answers.push(answer(&live_answer).to_string());
        assert_eq!(
            as_json(&restored.view()),
            as_json(&live.view()),
            "view after {event}"
        );
        assert_eq!(
            as_json(&restored.view_v2()),
            as_json(&live.view_v2()),
            "View 0.2 after {event}"
        );
        let items = as_json(&live.drain_archive());
        assert_eq!(
            as_json(&restored.drain_archive()),
            items,
            "archive after {event}"
        );
        departed.extend(journal_entries(&items).cloned());
    }
    let saved_next = live.save_json().unwrap();
    assert_eq!(
        restored.save_json().unwrap(),
        saved_next,
        "save after {next:?}"
    );
    ReactiveSession::restore_json(source, &saved_next)
        .unwrap_or_else(|error| panic!("the save after {next:?} was refused: {error}"));

    let saved: Value = serde_json::from_str(&saved).unwrap();
    Restored {
        journal: saved["decision_journal"].as_array().unwrap().clone(),
        next: answers,
        departed,
    }
}

/// The journal's entry for `commitment` with `change`.
fn entry<'a>(journal: &'a [Value], commitment: &str, change: &str) -> &'a Value {
    journal
        .iter()
        .find(|entry| entry["commitment"] == commitment && entry["change"] == change)
        .unwrap_or_else(|| panic!("no {commitment} {change} entry in {journal:?}"))
}

/// A list of names a save holds; a save leaves an empty list out.
fn names(list: &Value) -> Vec<&str> {
    list.as_array()
        .map(|names| names.iter().map(|name| name.as_str().unwrap()).collect())
        .unwrap_or_default()
}

fn short_names(count: usize) -> Vec<String> {
    (0..count).map(|k| format!("c{k:04}")).collect()
}

/// Names of 1,000 bytes.
fn long_names(count: usize) -> Vec<String> {
    (0..count)
        .map(|k| format!("c{k:04}{}", "x".repeat(995)))
        .collect()
}

/// An event every program here accepts after the others.
const BUMP: &str = "state t = 0;\nevent bump;\non bump set t = t + 1;\n";

/// Evidence `e` that `caveats` come to qualify at `flag`, after the entries
/// that cite it, while no value reads it. `rules` adds declarations and rules.
fn late_caveats(caveats: &[String], rules: &str) -> String {
    let mut source = String::from(
        "claim ok;\n\
         evidence e from \"tool\";\n\
         decisions d limit 4;\n\
         event report;\n\
         event decide;\n\
         event doubt;\n\
         event flag;\n",
    );
    for caveat in caveats {
        source.push_str(&format!("caveat {caveat} consequence low;\n"));
    }
    source.push_str(
        "on report reveal e supports ok;\n\
         on decide commit d because enough using 1;\n\
         on doubt reopen d because e;\n",
    );
    for caveat in caveats {
        source.push_str(&format!("on flag qualify e with {caveat};\n"));
    }
    source.push_str(BUMP);
    source.push_str(rules);
    source
}

/// The next revision rests on the reopening's cause, which now carries more
/// caveats than one lineage may hold: refused live, and the same after
/// restore, while the session goes on.
#[test]
fn a_reopening_whose_cause_gains_1024_caveats_after_it_restores() {
    let source = late_caveats(&short_names(1024), "");
    let restored = restores_and_plays_on(
        &source,
        &["report", "decide", "doubt", "flag"],
        &["decide", "bump"],
    );
    // The entry cites `e` alone; `e` and what reaches it now are 1,025 names.
    let reopened = entry(&restored.journal, "d@1", "reopened");
    assert_eq!(names(&reopened["because"]), ["e"]);
    assert_eq!(names(&reopened["caveats"]), Vec::<&str>::new());
    assert_eq!(restored.next, ["provenance_limit", "accepted"]);
}

#[test]
fn a_reopening_whose_cause_gains_66000_bytes_of_caveats_after_it_restores() {
    let source = late_caveats(&long_names(66), "");
    let restored = restores_and_plays_on(
        &source,
        &["report", "decide", "doubt", "flag"],
        &["decide", "bump"],
    );
    assert_eq!(restored.next, ["provenance_limit", "accepted"]);
}

/// Each change but a commit is checked against what reaches its causes now,
/// and a cause can come to carry any number of caveats after it.
#[test]
fn each_change_but_a_commit_restores_after_its_cause_gains_1024_caveats() {
    for (change, rules, events, next) in [
        (
            "held",
            "event freeze;\non freeze hold d because e;\n",
            &["report", "decide", "freeze", "flag"][..],
            "provenance_limit",
        ),
        (
            "deferred",
            "evidence h from \"hold reason\";\nevent freeze;\n\
             on report reveal h;\non freeze hold d because h;\n",
            &["report", "decide", "freeze", "doubt", "flag"],
            "provenance_limit",
        ),
        (
            "released",
            "evidence h from \"hold reason\";\nevent freeze;\nevent unfreeze;\n\
             on report reveal h;\non freeze hold d because h;\non unfreeze release d because e;\n",
            &["report", "decide", "freeze", "unfreeze", "flag"],
            "provenance_limit",
        ),
        (
            "resolved",
            "event judge;\non judge resolve d = 1 because e;\n",
            &["report", "decide", "judge", "flag"],
            "decision_in_force",
        ),
    ] {
        let source = late_caveats(&short_names(1024), rules);
        let restored = restores_and_plays_on(&source, events, &["decide", "bump"]);
        let cited = entry(&restored.journal, "d@1", change);
        assert_eq!(names(&cited["because"]), ["e"], "{change}");
        assert_eq!(restored.next, [next, "accepted"], "{change}");
    }
}

/// Evidence whose first caveat is taken back before a reopening cites it, and
/// whose last is added after the take-back and before the reopening. A
/// reopening may cite a caveat taken back after it, so restore counts the
/// relations taken back, and here it counted one more name than any value held.
fn taken_back(caveats: &[String]) -> String {
    let (last, first) = caveats.split_last().unwrap();
    let mut source = String::from(
        "claim ok;\n\
         evidence e from \"src\";\n\
         state x = 0;\n\
         decisions d limit 4;\n\
         event see;\n\
         event doubt;\n\
         event undoubt;\n\
         event more;\n\
         event decide;\n\
         event redo;\n",
    );
    for caveat in caveats {
        source.push_str(&format!("caveat {caveat} consequence low;\n"));
    }
    source.push_str(
        "on see reveal e supports ok;\n\
         on see set x = qualified(1, e);\n\
         on decide commit d because enough using x;\n\
         on redo reopen d because e;\n",
    );
    for caveat in first {
        source.push_str(&format!("on doubt qualify e with {caveat};\n"));
    }
    source.push_str(&format!("on undoubt unqualify e with {};\n", first[0]));
    source.push_str(&format!("on more qualify e with {last};\n"));
    source.push_str(BUMP);
    source
}

const TAKEN_BACK: [&str; 6] = ["see", "decide", "doubt", "undoubt", "more", "redo"];

/// Live, the next revision rests on `e` and the caveats it carries, within
/// the limits, and is made; the restored session makes it alike.
#[test]
fn a_reopening_after_a_take_back_restores_with_1025_names_counted() {
    let source = taken_back(&short_names(1024));
    let restored = restores_and_plays_on(&source, &TAKEN_BACK, &["decide", "bump"]);
    // Live, `e` and its 1,023 caveats are 1,024 names; restore also counted
    // the caveat taken back.
    let reopened = entry(&restored.journal, "d@1", "reopened");
    assert_eq!(names(&reopened["caveats"]).len(), 1023);
    assert!(!names(&reopened["caveats"]).contains(&"c0000"));
    assert_eq!(restored.next, ["accepted", "accepted"]);
}

#[test]
fn a_reopening_after_a_take_back_restores_with_65537_bytes_counted() {
    // 256 names of 256 bytes. Live, `e` and 255 of them are 65,281 bytes;
    // with the one taken back, 65,537.
    let caveats = (1..=256)
        .map(|k| {
            let tail = format!("_{k}");
            format!("c{}{tail}", "x".repeat(256 - 1 - tail.len()))
        })
        .collect::<Vec<_>>();
    let source = taken_back(&caveats);
    let restored = restores_and_plays_on(&source, &TAKEN_BACK, &["decide", "bump"]);
    let reopened = entry(&restored.journal, "d@1", "reopened");
    assert_eq!(names(&reopened["caveats"]).len(), 255);
    assert_eq!(restored.next, ["accepted", "accepted"]);
}

/// `gate` is committed on `base` and held; causes `e1` and `e2` are deferred,
/// then `per_cause` caveats arrive for each. A release, or the withdrawal of
/// `base` breaking through the hold, reopens `gate` citing both causes and
/// the caveats of each: more names together than one lineage may hold,
/// though each cause's own are fewer.
fn deferred_causes(per_cause: usize, a: &str, b: &str) -> String {
    let mut source = String::from(
        "evidence base from \"tool\";\n\
         evidence correction from \"tool\";\n\
         evidence e1 from \"tool\";\n\
         evidence e2 from \"tool\";\n\
         evidence team from \"instruction\";\n\
         decisions gate limit 8;\n\
         state v = 0;\n\
         event see;\n\
         event decide;\n\
         event freeze;\n\
         event r1;\n\
         event r2;\n\
         event flag1;\n\
         event flag2;\n\
         event unfreeze;\n\
         event retract;\n",
    );
    for k in 1..=per_cause {
        source.push_str(&format!("caveat {a}{k} consequence low;\n"));
        source.push_str(&format!("caveat {b}{k} consequence low;\n"));
    }
    source.push_str(
        "on see reveal base;\n\
         on see set v = qualified(1, base);\n\
         on decide commit gate because enough using v;\n\
         on freeze reveal team;\n\
         on freeze hold gate because team;\n\
         on r1 reveal e1;\n\
         on r1 reopen gate because e1;\n\
         on r2 reveal e2;\n\
         on r2 reopen gate because e2;\n\
         on unfreeze release gate because team;\n\
         on retract reveal correction;\n\
         on retract withdraw base because correction;\n\
         on retract reopen gate because correction;\n",
    );
    for k in 1..=per_cause {
        source.push_str(&format!("on flag1 qualify e1 with {a}{k};\n"));
    }
    for k in 1..=per_cause {
        source.push_str(&format!("on flag2 qualify e2 with {b}{k};\n"));
    }
    source.push_str(BUMP);
    source
}

fn deferred_then(last: &'static str) -> Vec<&'static str> {
    let mut events = vec!["see", "decide", "freeze", "r1", "r2", "flag1", "flag2"];
    events.push(last);
    events
}

/// The next revision rests on both causes, past the limits together: refused
/// live, and the same after restore.
#[test]
fn a_release_reopening_on_two_causes_with_1200_caveats_restores() {
    let source = deferred_causes(600, "a", "b");
    let restored = restores_and_plays_on(&source, &deferred_then("unfreeze"), &["decide", "bump"]);
    let reopened = entry(&restored.journal, "gate@1", "reopened");
    assert_eq!(names(&reopened["because"]), ["e1", "e2"]);
    assert_eq!(names(&reopened["caveats"]).len(), 1200);
    assert_eq!(restored.next, ["provenance_limit", "accepted"]);
}

#[test]
fn a_release_reopening_on_two_causes_with_69084_bytes_of_caveats_restores() {
    // Each cause's 350 caveats of 97 to 99 bytes are 34,542 bytes; both are
    // 69,084.
    let source = deferred_causes(
        350,
        &format!("a{}", "x".repeat(95)),
        &format!("b{}", "y".repeat(95)),
    );
    let restored = restores_and_plays_on(&source, &deferred_then("unfreeze"), &["decide", "bump"]);
    let caveats = names(&entry(&restored.journal, "gate@1", "reopened")["caveats"])
        .iter()
        .map(|caveat| caveat.len())
        .sum::<usize>();
    assert_eq!(caveats, 69_084);
    assert_eq!(restored.next, ["provenance_limit", "accepted"]);
}

#[test]
fn a_withdrawal_breaking_through_a_hold_with_1200_caveats_restores() {
    let source = deferred_causes(600, "a", "b");
    let restored = restores_and_plays_on(&source, &deferred_then("retract"), &["decide", "bump"]);
    let reopened = entry(&restored.journal, "gate@1", "reopened");
    assert_eq!(names(&reopened["because"]), ["e1", "e2", "correction"]);
    assert_eq!(names(&reopened["caveats"]).len(), 1200);
    assert_eq!(restored.next, ["provenance_limit", "accepted"]);
}

/// No limit applies to an entry now, but each caveat it cites must still
/// reach one of its causes: one that reaches none is refused, at any size.
#[test]
fn a_large_reopening_citing_a_caveat_unrelated_to_its_causes_is_refused() {
    let source = deferred_causes(600, "a", "b").replace(
        "decisions gate limit 8;\n",
        "decisions gate limit 8;\ncaveat stray consequence low;\n",
    );
    let mut save = played(&source, &deferred_then("unfreeze")).save().unwrap();
    let reopened = save
        .decision_journal
        .iter_mut()
        .find(|entry| entry.change == "reopened")
        .unwrap();
    reopened.caveats.push("stray".into());
    reopened.caveats.sort();
    assert_eq!(
        ReactiveSession::restore(&source, &save).err().unwrap(),
        "cannot restore save: decision journal: reopening cites caveats unrelated to its witnesses"
    );
}

/// A journal window of two entries, and two more series, so that each of
/// their commits retires an older entry, which then departs.
const WINDOWED: &str = "journal window 2;\n\
    decisions z1 limit 4;\n\
    decisions z2 limit 4;\n\
    event zc1;\n\
    event zc2;\n\
    on zc1 commit z1 because enough using 1;\n\
    on zc2 commit z2 because enough using 1;\n";

/// After a restore, the window retires the oversized reopening, and it
/// departs into the archive with every name it cites, as it does live.
#[test]
fn a_release_reopening_with_1200_caveats_departs_after_restore_as_it_does_live() {
    let source = format!("{}{WINDOWED}", deferred_causes(600, "a", "b"));
    let restored = restores_and_plays_on(
        &source,
        &deferred_then("unfreeze"),
        &["decide", "zc1", "zc2", "bump"],
    );
    let departed = restored
        .departed
        .iter()
        .find(|entry| entry["change"] == "reopened")
        .unwrap_or_else(|| panic!("the reopening never departed: {:?}", restored.departed));
    assert_eq!(names(&departed["because"]), ["e1", "e2"]);
    assert_eq!(names(&departed["caveats"]).len(), 1200);
    assert_eq!(
        restored.next,
        ["provenance_limit", "accepted", "accepted", "accepted"]
    );
}

/// `d` is held because of `e`, which comes to carry 1,024 caveats after the
/// hold, with the window above.
fn held_then_flagged() -> String {
    late_caveats(
        &short_names(1024),
        &format!("event freeze;\non freeze hold d because e;\n{WINDOWED}"),
    )
}

/// A save whose held entry cites every caveat its cause carries now: no
/// event wrote it, but each caveat reaches the cause, so it restores (A3:
/// restore does not authenticate history). The window retires the entry
/// during the next events and it departs with all 1,025 names, and the save
/// after restores.
#[test]
fn a_hand_edited_held_entry_of_1025_names_departs_after_restore() {
    let source = held_then_flagged();
    let mut save = played(&source, &["report", "decide", "freeze", "flag"])
        .save()
        .unwrap();
    let held = save
        .decision_journal
        .iter_mut()
        .find(|entry| entry.change == "held")
        .unwrap();
    held.caveats = short_names(1024);
    let mut restored = ReactiveSession::restore(&source, &save)
        .unwrap_or_else(|error| panic!("the edited save was refused: {error}"));
    let mut answers = Vec::new();
    let mut departed = Vec::new();
    for event in ["zc1", "zc2", "bump"] {
        answers.push(answer(&outcome(&mut restored, event)).to_string());
        as_json(&restored.view());
        as_json(&restored.view_v2());
        departed.extend(journal_entries(&as_json(&restored.drain_archive())).cloned());
    }
    assert_eq!(answers, ["accepted", "accepted", "accepted"]);
    let held = departed
        .iter()
        .find(|entry| entry["change"] == "held")
        .unwrap_or_else(|| panic!("the held entry never departed: {departed:?}"));
    assert_eq!(names(&held["because"]), ["e"]);
    assert_eq!(names(&held["caveats"]).len(), 1024);
    let saved = restored.save_json().unwrap();
    ReactiveSession::restore_json(&source, &saved)
        .unwrap_or_else(|error| panic!("the save after was refused: {error}"));
}

/// The generator of the save fuzz in save_restore.rs.
struct Mulberry(u32);

impl Mulberry {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        t ^ (t >> 14)
    }

    fn below(&mut self, n: usize) -> usize {
        self.next() as usize % n.max(1)
    }
}

/// Every place in `value`, as the keys and indexes that lead to it.
fn places(value: &Value, at: Vec<Value>, out: &mut Vec<Vec<Value>>) {
    out.push(at.clone());
    match value {
        Value::Object(fields) => {
            for (key, child) in fields {
                let mut next = at.clone();
                next.push(key.clone().into());
                places(child, next, out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let mut next = at.clone();
                next.push(index.into());
                places(child, next, out);
            }
        }
        _ => {}
    }
}

fn place<'a>(value: &'a mut Value, path: &[Value]) -> Option<&'a mut Value> {
    let pointer = path
        .iter()
        .map(|step| match step {
            Value::String(key) => format!("/{}", key.replace('~', "~0").replace('/', "~1")),
            other => format!("/{other}"),
        })
        .collect::<String>();
    value.pointer_mut(&pointer)
}

fn remove(value: &mut Value, path: &[Value]) {
    let Some((last, parent)) = path.split_last() else {
        return;
    };
    match (place(value, parent), last) {
        (Some(Value::Object(fields)), Value::String(key)) => {
            fields.remove(key);
        }
        (Some(Value::Array(items)), Value::Number(index)) => {
            if let Some(index) = index.as_u64().filter(|index| *index < items.len() as u64) {
                items.remove(index as usize);
            }
        }
        _ => {}
    }
}

/// One to three alterations of a copy of `save`, half of them inside the
/// oversized entry: the save fuzz's (a value replaced or removed), and a name
/// added to a list, one of its names repeated, or the list reversed.
fn altered(
    save: &Value,
    everywhere: &[Vec<Value>],
    inside: &[Vec<Value>],
    words: &[&str],
    random: &mut Mulberry,
) -> Value {
    let mut altered = save.clone();
    for _ in 0..1 + random.below(3) {
        let pool = if random.below(2) == 0 {
            inside
        } else {
            everywhere
        };
        let path = &pool[random.below(pool.len())];
        let alteration = random.below(11);
        if alteration == 7 {
            remove(&mut altered, path);
            continue;
        }
        let Some(target) = place(&mut altered, path) else {
            continue;
        };
        let word = words[random.below(words.len())];
        match (alteration, target) {
            (0, target) => *target = Value::Null,
            (1, target) => *target = word.into(),
            (2, target) => *target = (random.below(2000) as f64 - 1000.0).into(),
            (3, target) => *target = json!([]),
            (4, target) => *target = json!({}),
            (5, target) => *target = json!([word]),
            (6, target) => *target = u64::MAX.into(),
            (8, Value::Array(items)) => {
                let at = random.below(items.len() + 1);
                items.insert(at, word.into());
            }
            (9, Value::Array(items)) if !items.is_empty() => {
                let copy = items[random.below(items.len())].clone();
                items.push(copy);
            }
            (10, Value::Array(items)) => items.reverse(),
            _ => {}
        }
    }
    altered
}

/// Restores `text` and plays `next` on it, reading the snapshot, both views
/// and the archive after each event: `Ok(false)` if restore refused it,
/// `Ok(true)` if every event was answered and the save after restores.
fn plays_on(source: &str, text: &str, next: &[&str]) -> Result<bool, String> {
    let Ok(mut session) = ReactiveSession::restore_json(source, text) else {
        return Ok(false);
    };
    for event in next {
        session
            .dispatch_outcome_json(event, "{}")
            .map_err(|fatal| format!("{event} was fatal: {}", fatal.message))?;
        session.snapshot();
        as_json(&session.view());
        as_json(&session.view_v2());
        session.drain_archive();
    }
    let saved = session.save_json()?;
    ReactiveSession::restore_json(source, &saved)
        .map_err(|error| format!("the save after {next:?} was refused: {error}"))?;
    Ok(true)
}

/// Alters `save` round after round from each seed, half the time inside its
/// journal entry past the limits: whatever restore accepts must play on with
/// no panic and no fatal outcome, and its save must restore.
/// D038_FUZZ_ROUNDS runs it longer.
fn fuzz_past_the_limits(source: &str, save: &str, words: &[&str], next: &[&str], seeds: [u32; 2]) {
    let save: Value = serde_json::from_str(save).unwrap();
    let entry = save["decision_journal"]
        .as_array()
        .unwrap()
        .iter()
        .position(|entry| names(&entry["because"]).len() + names(&entry["caveats"]).len() > 1024)
        .expect("an entry past the limits");
    let mut everywhere = Vec::new();
    places(&save, Vec::new(), &mut everywhere);
    let prefix = [json!("decision_journal"), json!(entry)];
    let inside = everywhere
        .iter()
        .filter(|path| path.starts_with(&prefix))
        .cloned()
        .collect::<Vec<_>>();
    let rounds = std::env::var("D038_FUZZ_ROUNDS")
        .ok()
        .and_then(|rounds| rounds.parse().ok())
        .unwrap_or(40);
    for seed in seeds {
        let mut random = Mulberry(seed);
        let mut accepted = 0;
        for round in 0..rounds {
            let text = altered(&save, &everywhere, &inside, words, &mut random).to_string();
            match std::panic::catch_unwind(|| plays_on(source, &text, next)) {
                Ok(Ok(played)) => accepted += u32::from(played),
                Ok(Err(failure)) => panic!("seed {seed}, round {round}: {failure}"),
                Err(_) => panic!("seed {seed}, round {round}: panicked"),
            }
        }
        eprintln!("seed {seed}, {rounds} rounds: accepted {accepted}");
    }
}

#[test]
fn altered_saves_with_a_release_reopening_of_1200_caveats_never_crash() {
    let source = format!("{}{WINDOWED}", deferred_causes(600, "a", "b"));
    let mut live = played(&source, &deferred_then("unfreeze"));
    let save = live.save_json().unwrap();
    let words = [
        "e1",
        "e2",
        "base",
        "team",
        "a1",
        "b600",
        "gate@1",
        "gate",
        "journal@3",
        "released",
        "reopened",
        "",
        "z1@1",
    ];
    let next = [
        "decide", "zc1", "retract", "zc2", "unfreeze", "freeze", "bump",
    ];
    fuzz_past_the_limits(&source, &save, &words, &next, [1, 2]);
    // Unaltered, it plays on.
    assert_eq!(answer(&outcome(&mut live, "zc1")), "accepted");
}

#[test]
fn altered_saves_with_a_hand_edited_held_entry_of_1025_names_never_crash() {
    let source = held_then_flagged();
    let mut save = played(&source, &["report", "decide", "freeze", "flag"])
        .save()
        .unwrap();
    let held = save
        .decision_journal
        .iter_mut()
        .find(|entry| entry.change == "held")
        .unwrap();
    held.caveats = short_names(1024);
    let save = serde_json::to_string(&save).unwrap();
    let words = [
        "e",
        "c0000",
        "c1023",
        "d@1",
        "d",
        "held",
        "reopened",
        "journal@2",
        "",
        "z2@1",
    ];
    let next = ["zc1", "doubt", "decide", "zc2", "flag", "bump"];
    fuzz_past_the_limits(&source, &save, &words, &next, [3, 4]);
}
