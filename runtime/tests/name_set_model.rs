//! Compiler step 5: a lineage's name set keeps one or two names in its own
//! allocation and more in a list, and counts its bytes as it is built.
//! Random edits over a pool of sets, some sharing storage through clones,
//! must leave each set holding exactly what a `BTreeSet<String>` given the
//! same edits holds, in the same order, and must count its bytes as the
//! provenance limit reads them.
use caveat_runtime::reactive::{NameSet, Provenance};
use std::collections::BTreeSet;

const NAMES: [&str; 7] = ["a", "bb", "c", "dddd", "e", "ff", "ggg"];
const POOL: usize = 6;

fn same(actual: &NameSet, model: &BTreeSet<String>, context: &str) {
    let listed: Vec<&str> = actual.iter().map(|name| name.as_str()).collect();
    let expected: Vec<&str> = model.iter().map(String::as_str).collect();
    assert_eq!(listed, expected, "{context}");
    assert_eq!(actual.len(), model.len(), "{context}");
    assert_eq!(actual.is_empty(), model.is_empty(), "{context}");
    assert_eq!(
        actual.first().map(|name| name.as_str()),
        model.first().map(String::as_str),
        "{context}"
    );
    assert!(*actual == *model, "{context}");
    for name in NAMES {
        assert_eq!(
            actual.contains(name),
            model.contains(name),
            "{context}: {name}"
        );
        assert_eq!(
            actual.get(name).map(|held| held.as_str()),
            model.get(name).map(String::as_str),
            "{context}: {name}"
        );
    }
}

#[test]
fn name_sets_hold_what_a_tree_of_names_holds() {
    let mut seed: u64 = 0x5e75_0f0a_e511_cafe;
    let mut next = move |bound: usize| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize % bound
    };
    let mut sets = vec![NameSet::default(); POOL];
    let mut models = vec![BTreeSet::<String>::new(); POOL];
    for step in 0..40_000 {
        let at = next(POOL);
        let name = NAMES[next(NAMES.len())];
        let context = format!("step {step}");
        match next(9) {
            0 | 1 => {
                assert_eq!(
                    sets[at].insert(name),
                    models[at].insert(name.to_string()),
                    "{context}: insert {name}"
                );
            }
            2 => {
                assert_eq!(
                    sets[at].remove(name),
                    models[at].remove(name),
                    "{context}: remove {name}"
                );
            }
            3 => {
                // A clone shares the storage until either one changes.
                let from = next(POOL);
                sets[at] = sets[from].clone();
                models[at] = models[from].clone();
            }
            4 => {
                let drop = NAMES[next(NAMES.len())];
                sets[at].retain(|held| held.as_str() != name && held.as_str() != drop);
                models[at].retain(|held| held != name && held != drop);
            }
            5 => {
                let more = [name, NAMES[next(NAMES.len())], NAMES[next(NAMES.len())]];
                sets[at].extend(more);
                models[at].extend(more.map(String::from));
            }
            6 => {
                // Joined as a lineage joins them.
                let from = next(POOL);
                let mut joined = Provenance {
                    evidence: sets[at].clone(),
                    ..Provenance::default()
                };
                let other = Provenance {
                    evidence: sets[from].clone(),
                    ..Provenance::default()
                };
                joined.merge(&other).unwrap();
                sets[at] = joined.evidence;
                let other = models[from].clone();
                models[at].extend(other);
            }
            7 => {
                let built: NameSet = models[at].iter().cloned().collect();
                assert!(built == sets[at], "{context}: collected");
                assert_eq!(built.cmp(&sets[at]), std::cmp::Ordering::Equal);
                let text = serde_json::to_string(&sets[at]).unwrap();
                let read: NameSet = serde_json::from_str(&text).unwrap();
                same(&read, &models[at], &format!("{context}: read back"));
                let owned: Vec<String> = sets[at].clone().into_iter().map(String::from).collect();
                assert_eq!(owned, models[at].iter().cloned().collect::<Vec<_>>());
            }
            _ => {
                let other = next(POOL);
                assert_eq!(
                    sets[at].is_subset(&sets[other]),
                    models[at].is_subset(&models[other]),
                    "{context}: subset"
                );
                let difference: Vec<&str> = sets[at]
                    .difference(&sets[other])
                    .map(|name| name.as_str())
                    .collect();
                let expected: Vec<&str> = models[at]
                    .difference(&models[other])
                    .map(String::as_str)
                    .collect();
                assert_eq!(difference, expected, "{context}: difference");
                assert_eq!(
                    sets[at].cmp(&sets[other]),
                    models[at].cmp(&models[other]),
                    "{context}: order"
                );
            }
        }
        for (index, (set, model)) in sets.iter().zip(&models).enumerate() {
            same(set, model, &format!("{context}, set {index}"));
        }
    }
}

/// The name numbered `index`, `bytes` bytes long.
fn long_name(index: usize, bytes: usize) -> String {
    format!("{index:0>bytes$}")
}

/// A lineage of `count` names of `bytes` bytes each, built one name at a
/// time in place.
fn inserted(count: usize, bytes: usize) -> Provenance {
    let mut provenance = Provenance::default();
    for index in 0..count {
        provenance.evidence.insert(long_name(index, bytes));
    }
    provenance
}

/// The same lineage built as lineages are, by joining one-name lineages; a
/// join that would pass the limit is refused.
fn joined(count: usize, bytes: usize) -> Result<Provenance, String> {
    let mut provenance = Provenance::default();
    for index in 0..count {
        let mut one = Provenance::default();
        one.evidence.insert(long_name(index, bytes));
        provenance.merge(&one).map_err(|error| error.to_string())?;
    }
    Ok(provenance)
}

#[test]
fn a_set_counts_its_bytes_as_the_provenance_limit_reads_them() {
    let limit = "value provenance exceeds limit 65536 name bytes";
    // 64 names of 1,024 bytes are 65,536 bytes, the limit: accepted.
    for provenance in [inserted(64, 1_024), joined(64, 1_024).unwrap()] {
        provenance.validate().unwrap();
        // One more name passes it, and taking one away gives its bytes back.
        let mut over = provenance.clone();
        over.caveats.insert("x");
        assert!(over.validate().unwrap_err().to_string().contains(limit));
        let mut shared = over.clone();
        over.evidence.remove(&long_name(0, 1_024));
        over.validate().unwrap();
        // The clone it shared its names with still counts them all.
        assert!(shared.validate().unwrap_err().to_string().contains(limit));
        shared.caveats.remove("x");
        shared.validate().unwrap();
    }
    // 63 names of 1,041 bytes are 65,583 bytes: past the limit, and the join
    // that passes it is refused.
    let provenance = inserted(63, 1_041);
    assert_eq!(provenance.evidence.len(), 63);
    assert!(provenance
        .validate()
        .unwrap_err()
        .to_string()
        .contains(limit));
    assert!(joined(63, 1_041).unwrap_err().contains(limit));
    joined(62, 1_041).unwrap().validate().unwrap();
}

/// Whether the provenance limit refuses `provenance` for its name bytes.
fn refused(provenance: &Provenance) -> bool {
    match provenance.validate() {
        Ok(()) => false,
        Err(error) => {
            let error = error.to_string();
            assert!(
                error.contains("value provenance exceeds limit 65536 name bytes"),
                "{error}"
            );
            true
        }
    }
}

/// Names of 1, 10, 100 and 1,000 bytes: counting another name's bytes for
/// the one taken out miscounts.
fn sized_name(index: usize) -> String {
    ["a", "b", "c", "d"][index].repeat(10_usize.pow(index as u32))
}

#[test]
fn a_name_taken_out_in_place_gives_back_its_own_bytes() {
    for held in 2..=4 {
        for removed in 0..held {
            let context = format!("{held} names, removing {removed}");
            // Built in place: no other value shares the set.
            let mut provenance = Provenance::default();
            for index in 0..held {
                provenance.evidence.insert(sized_name(index));
            }
            assert!(
                provenance.evidence.remove(&sized_name(removed)),
                "{context}"
            );
            let left: usize = (0..held)
                .filter(|&index| index != removed)
                .map(|index| sized_name(index).len())
                .sum();
            // Caveats that bring the lineage to the limit exactly ...
            provenance.caveats.insert("k".repeat(65_536 - left));
            assert!(!refused(&provenance), "{context}");
            // ... and one more byte past it.
            provenance.caveats.insert("l");
            assert!(refused(&provenance), "{context}");
        }
    }
}

#[test]
fn a_name_added_to_a_shared_set_counts_its_bytes() {
    for held in 1..=3 {
        // `held` names, 65,536 bytes in all: the limit.
        let mut provenance = Provenance::default();
        for index in 0..held {
            let bytes = if index == 0 { 65_536 - (held - 1) } else { 1 };
            provenance.evidence.insert(format!("{index}").repeat(bytes));
        }
        assert!(!refused(&provenance), "{held}");
        // A clone shares the set; adding a name copies it, with the name.
        let shared = provenance.clone();
        assert!(provenance.evidence.insert("z"));
        assert!(refused(&provenance), "{held}");
        // The clone it shared the set with still counts only its own names.
        assert!(!refused(&shared), "{held}");
    }
}
