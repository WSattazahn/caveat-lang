//! The decision journal: every commitment and reopening, in order.
//! See spec/caveat-decision-journal-0.1.md.
use caveat_runtime::reactive::{JournalEntry, ReactiveSession};

const PROGRAM: &str = r#"
claim safe;
evidence second from "second";
evidence first from "first";
evidence doubt from "doubt";
evidence later from "later";
caveat dim consequence low;
dim qualifies later;
decisions trust limit 4;
state recovery = 0;
event see;
event worry;
event recover;
on see reveal first supports safe;
on see reveal second supports safe;
on see commit trust because enough using qualified(1, second) + qualified(1, first);
on worry reveal doubt opposes safe;
on worry reopen trust because doubt;
on recover reveal later supports safe;
on recover set recovery = qualified(1, later);
on recover when reopened(trust) commit trust because enough using recovery;
"#;

fn summary(journal: &[JournalEntry]) -> Vec<(String, String, Vec<String>)> {
    journal
        .iter()
        .map(|e| (e.change.clone(), e.commitment.clone(), e.because.clone()))
        .collect()
}

fn entry(change: &str, commitment: &str, because: &[&str]) -> (String, String, Vec<String>) {
    (
        change.into(),
        commitment.into(),
        because.iter().map(|b| (*b).into()).collect(),
    )
}

#[test]
fn the_journal_records_each_decision_change_in_order() {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    game.dispatch_json("see", "{}").unwrap();
    game.dispatch_json("worry", "{}").unwrap();
    let shown = game.dispatch_json("recover", "{}").unwrap();
    assert_eq!(
        summary(&shown.decision_journal),
        vec![
            // Evidence in the order it was observed, not alphabetical order.
            entry("committed", "trust@1", &["first", "second"]),
            entry("reopened", "trust@1", &["doubt"]),
            // Grounded on what this revision used, not on its predecessor.
            entry("committed", "trust@2", &["later"]),
        ]
    );
    let last = shown.decision_journal.last().unwrap();
    assert_eq!(last.decision, "trust");
    assert_eq!(last.event, "recover");
    assert_eq!(last.sequence, 3);
    assert_eq!(last.caveats, vec!["dim".to_string()]);
}

#[test]
fn repeating_a_reopening_adds_no_entry_and_a_failed_event_adds_nothing() {
    let mut game = ReactiveSession::from_source(&format!(
        "{PROGRAM}\nevent broken;\non broken reopen trust because doubt;\non broken set recovery = 1 / 0;"
    ))
    .unwrap();
    game.dispatch_json("see", "{}").unwrap();
    game.dispatch_json("worry", "{}").unwrap();
    let shown = game.dispatch_json("worry", "{}").unwrap();
    assert_eq!(shown.decision_journal.len(), 2);
    let before = game.snapshot();
    assert!(game.dispatch_json("broken", "{}").is_err());
    assert_eq!(game.snapshot().decision_journal, before.decision_journal);
}

#[test]
fn the_view_carries_the_journal() {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    let view = game.dispatch_view_json("see", "{}").unwrap();
    assert_eq!(
        summary(view.decision_journal),
        vec![entry("committed", "trust@1", &["first", "second"])]
    );
    let json = serde_json::to_value(&view).unwrap();
    assert_eq!(json["decision_journal"][0]["change"], "committed");
}

#[test]
fn journal_changes_publish_their_time_and_the_commitments_frozen_value() {
    let source = format!(
        "{PROGRAM}\nevent advance dt min 0 max 30; clock advance every 1;\nevent overwrite; on overwrite set recovery = 99;"
    )
    .replace(
        "on see commit trust because enough using qualified(1, second) + qualified(1, first);",
        "on see set recovery = qualified(1, second) + qualified(1, first); on see commit trust because enough using recovery;",
    );
    let mut game = ReactiveSession::from_source(&source).unwrap();
    game.dispatch_json("advance", r#"{"dt":11}"#).unwrap();
    game.dispatch_json("see", "{}").unwrap();
    game.dispatch_json("advance", r#"{"dt":13}"#).unwrap();
    game.dispatch_json("overwrite", "{}").unwrap();
    let snapshot = game.dispatch_json("worry", "{}").unwrap();
    let journal = snapshot.decision_journal;
    assert_eq!(journal[0].elapsed, Some(11.0));
    assert_eq!(journal[0].value, Some(2.0));
    assert_eq!(journal[1].elapsed, Some(24.0));
    assert_eq!(journal[1].value, Some(2.0));
    assert_eq!(game.view().decision_journal, journal);
    let json = serde_json::to_value(game.view()).unwrap();
    assert_eq!(json["decision_journal"][1]["elapsed"], 24.0);
    assert_eq!(json["decision_journal"][1]["value"], 2.0);
}

#[test]
fn old_saved_journal_entries_remain_readable_without_inventing_time_or_values() {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    game.dispatch_json("see", "{}").unwrap();
    let mut save: serde_json::Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
    let first = save["decision_journal"][0].as_object_mut().unwrap();
    first.remove("elapsed");
    first.remove("value");
    let mut restored = ReactiveSession::restore_json(PROGRAM, &save.to_string()).unwrap();
    assert_eq!(restored.view().decision_journal[0].elapsed, None);
    assert_eq!(restored.view().decision_journal[0].value, None);
    let view = serde_json::to_value(restored.view()).unwrap();
    assert!(view["decision_journal"][0].get("elapsed").is_none());
    assert!(view["decision_journal"][0].get("value").is_none());
    restored.dispatch_json("worry", "{}").unwrap();
    assert_eq!(restored.view().decision_journal[1].elapsed, Some(0.0));
    assert_eq!(restored.view().decision_journal[1].value, Some(2.0));
}

#[test]
fn decisions_without_using_do_not_invent_a_numeric_value() {
    let mut game = ReactiveSession::from_source(
        r#"
claim safe;
evidence doubt from "doubt";
event decide;
event reconsider;
on decide commit choice because enough;
on reconsider reveal doubt opposes safe;
on reconsider reopen choice because doubt;
"#,
    )
    .unwrap();
    game.dispatch_json("decide", "{}").unwrap();
    game.dispatch_json("reconsider", "{}").unwrap();
    for entry in game.view().decision_journal {
        assert_eq!(entry.elapsed, Some(0.0));
        assert_eq!(entry.value, None);
        assert!(serde_json::to_value(entry).unwrap().get("value").is_none());
    }
}
