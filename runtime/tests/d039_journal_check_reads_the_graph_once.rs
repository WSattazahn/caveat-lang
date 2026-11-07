//! D039: restoring a save took minutes where loading its program took
//! milliseconds, though spec/caveat-save-0.1.md promises that restoring costs
//! what loading costs plus the size of the save. The decision journal check
//! read the whole graph again for each entry: every relation once for each
//! node a reopening's walk reached, the observation ledger and every
//! relation again for each name it checked as observed, and every effect of
//! the program for each series a record asked about. A 2 MB save the runtime
//! wrote took 15 minutes. Restore now reads what the check needs out of the
//! graph once, and walks what reaches each cause once for the whole journal,
//! keeping those walks up to the size of the graph. Every save restores and
//! is refused as before; these tests play the case where the walks outgrow
//! what restore keeps. See spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;

/// Evidence `e1` to `e{causes}`, each qualified by the caveat `hub`, which
/// `k1` to `k{caveats}` qualify in turn: every caveat reaches every cause.
/// Each `r{i}` reopens `d` because `e{i}`, so each reopening cites all of
/// them, and the walks from all the causes together hold more caveats than
/// restore keeps, which is as many as the graph has nodes and relations.
fn hub(causes: usize, caveats: usize) -> String {
    let mut source = format!(
        "claim ok;\ncaveat hub consequence low;\ncaveat stray consequence low;\n\
         decisions d limit {};\nevent decide;\non decide commit d because enough using 1;\n",
        causes + 1
    );
    for k in 1..=caveats {
        source.push_str(&format!(
            "caveat k{k} consequence low;\nk{k} qualifies hub;\n"
        ));
    }
    for i in 1..=causes {
        source.push_str(&format!(
            "evidence e{i} from \"tool\";\nhub qualifies e{i};\nevent r{i};\n\
             on r{i} reveal e{i} supports ok;\non r{i} reopen d because e{i};\n"
        ));
    }
    source.push_str("state t = 0;\nevent bump;\non bump set t = t + 1;\n");
    source
}

/// `d` committed once, then reopened by each cause in turn.
fn played(source: &str, causes: usize) -> ReactiveSession {
    let mut session = ReactiveSession::from_source(source).unwrap();
    let events = std::iter::once("decide".to_string()).chain((1..=causes).map(|i| format!("r{i}")));
    for event in events {
        let outcome = session
            .dispatch_outcome_json(&event, "{}")
            .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
        let outcome = serde_json::to_value(outcome).unwrap();
        assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    }
    session
}

#[test]
fn reopenings_whose_caveats_outgrow_what_restore_keeps_restore_alike() {
    let source = hub(24, 40);
    let mut live = played(&source, 24);
    let saved = live.save_json().unwrap();
    let mut restored = ReactiveSession::restore_json(&source, &saved)
        .unwrap_or_else(|error| panic!("restore refused the runtime's own save: {error}"));
    assert_eq!(restored.save_json().unwrap(), saved);
    assert_eq!(restored.snapshot(), live.snapshot());
    for event in ["decide", "bump"] {
        let live_outcome = serde_json::to_value(live.dispatch_outcome_json(event, "{}").unwrap());
        let restored_outcome =
            serde_json::to_value(restored.dispatch_outcome_json(event, "{}").unwrap());
        assert_eq!(restored_outcome.unwrap(), live_outcome.unwrap(), "{event}");
    }
    assert_eq!(restored.save_json().unwrap(), live.save_json().unwrap());
    let save = live.save().unwrap();
    let reopenings = save
        .decision_journal
        .iter()
        .filter(|entry| entry.change == "reopened")
        .collect::<Vec<_>>();
    assert_eq!(reopenings.len(), 24);
    for entry in &reopenings {
        assert_eq!(entry.caveats.len(), 41, "{}", entry.commitment);
    }
}

/// A caveat that reaches none of a reopening's causes is refused, whether
/// restore kept the walk from that cause or walked it again.
#[test]
fn a_caveat_reaching_no_cause_is_refused_in_the_first_and_the_last_reopening() {
    let source = hub(24, 40);
    let save = played(&source, 24).save().unwrap();
    let reopenings = save
        .decision_journal
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.change == "reopened")
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    for index in [reopenings[0], reopenings[23]] {
        let mut edited = save.clone();
        let entry = &mut edited.decision_journal[index];
        entry.caveats.push("stray".into());
        entry.caveats.sort();
        assert_eq!(
            ReactiveSession::restore(&source, &edited).err().as_deref(),
            Some(
                "cannot restore save: decision journal: reopening cites caveats unrelated to its \
                 witnesses"
            ),
            "{}",
            edited.decision_journal[index].commitment
        );
    }
}
