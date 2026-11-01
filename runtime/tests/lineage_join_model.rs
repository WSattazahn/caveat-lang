//! Compiler step 6: a lineage join finds the names one side adds and builds
//! the joined set in one walk, looking each name up by halves when what is
//! left of the set it joins is much larger. Random joins of lineages of many
//! sizes, some holding copies of one name and some sharing whole sets, must
//! hold what a union of name trees holds. Near the provenance limits they
//! must refuse with the text that counting each added name in turn gives,
//! evidence in name order and then caveats, and leave the lineage as it was.
use caveat_runtime::reactive::{Marker, Name, NameSet, Provenance};
use std::collections::{BTreeMap, BTreeSet};

const IDENTIFIERS: usize = 1024;
const BYTES: usize = 65_536;
const OVER_IDENTIFIERS: &str = "value provenance exceeds limit 1024 identifiers";
const OVER_BYTES: &str = "value provenance exceeds limit 65536 name bytes";

struct Random(u64);

impl Random {
    fn below(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize % bound
    }
}

/// The name numbered `index`, `bytes` bytes long: no two numbers give the
/// same text.
fn text(index: usize, bytes: usize) -> String {
    format!("{index:0>bytes$}")
}

/// The names a lineage should hold.
#[derive(Clone, Default)]
struct Model {
    evidence: BTreeSet<String>,
    caveats: BTreeSet<String>,
}

impl Model {
    fn size(&self) -> (usize, usize) {
        let names = self.evidence.iter().chain(&self.caveats);
        (
            self.evidence.len() + self.caveats.len(),
            names.map(String::len).sum(),
        )
    }
}

/// The refusal joining `theirs` into `ours` gives, if any: each lineage is
/// checked whole, ours first, and then each name `theirs` adds is counted in
/// turn, identifiers before bytes.
fn refusal(ours: &Model, theirs: &Model) -> Option<&'static str> {
    for model in [ours, theirs] {
        let (count, bytes) = model.size();
        if count > IDENTIFIERS {
            return Some(OVER_IDENTIFIERS);
        }
        if bytes > BYTES {
            return Some(OVER_BYTES);
        }
    }
    let (mut count, mut bytes) = ours.size();
    let added = theirs
        .evidence
        .difference(&ours.evidence)
        .chain(theirs.caveats.difference(&ours.caveats));
    for name in added {
        if count + 1 > IDENTIFIERS {
            return Some(OVER_IDENTIFIERS);
        }
        if bytes + name.len() > BYTES {
            return Some(OVER_BYTES);
        }
        count += 1;
        bytes += name.len();
    }
    None
}

/// A set of `names`, each a copy of the pool's name or, at random, a name
/// of its own with the same text, added in a random order.
fn set_of(names: &BTreeSet<String>, pool: &BTreeMap<String, Name>, random: &mut Random) -> NameSet {
    let mut order: Vec<&String> = names.iter().collect();
    for at in (1..order.len()).rev() {
        order.swap(at, random.below(at + 1));
    }
    let mut set = NameSet::default();
    for name in order {
        if random.below(3) == 0 {
            set.insert(name.clone());
        } else {
            set.insert(pool[name].clone());
        }
    }
    set
}

/// Join `theirs` into `ours` and compare with the model.
fn join_and_compare(
    ours: &Provenance,
    theirs: &Provenance,
    ours_model: &Model,
    theirs_model: &Model,
    context: &str,
) {
    let mut joined = ours.clone();
    match (joined.merge(theirs), refusal(ours_model, theirs_model)) {
        (Ok(()), None) => {
            let mut evidence = ours_model.evidence.clone();
            evidence.extend(theirs_model.evidence.iter().cloned());
            let mut caveats = ours_model.caveats.clone();
            caveats.extend(theirs_model.caveats.iter().cloned());
            assert!(joined.evidence == evidence, "{context}: evidence");
            assert!(joined.caveats == caveats, "{context}: caveats");
            let listed: Vec<&str> = joined.evidence.iter().map(|name| name.as_str()).collect();
            let expected: Vec<&str> = evidence.iter().map(String::as_str).collect();
            assert_eq!(listed, expected, "{context}: evidence order");
            joined.validate().unwrap();
        }
        (Err(error), Some(text)) => {
            assert!(
                error.to_string().contains(text),
                "{context}: refused with {error}, not {text}"
            );
            assert!(
                joined == *ours,
                "{context}: a refused join changed the lineage"
            );
        }
        (result, expected) => {
            panic!("{context}: joined with {result:?}, the model refuses with {expected:?}")
        }
    }
}

#[test]
fn small_joins_hold_what_a_union_of_name_trees_holds() {
    let mut random = Random(0x6a01_2b3c_4d5e_6f70);
    let pool: BTreeMap<String, Name> = (0..64)
        .map(|index| {
            let text = text(index, 1 + index % 12);
            (text.clone(), Name::from(text))
        })
        .collect();
    let texts: Vec<&String> = pool.keys().collect();
    for step in 0..20_000 {
        let mut models = [Model::default(), Model::default()];
        // Ours from none to forty names, theirs from none to twelve: both
        // ways of looking a name up are taken.
        for (model, most) in models.iter_mut().zip([41, 13]) {
            for _ in 0..random.below(most) {
                let name = texts[random.below(texts.len())].clone();
                if random.below(4) == 0 {
                    model.caveats.insert(name);
                } else {
                    model.evidence.insert(name);
                }
            }
        }
        let [ours_model, mut theirs_model] = models;
        let ours = Provenance {
            evidence: set_of(&ours_model.evidence, &pool, &mut random),
            caveats: set_of(&ours_model.caveats, &pool, &mut random),
            ..Provenance::default()
        };
        let mut theirs = Provenance {
            evidence: set_of(&theirs_model.evidence, &pool, &mut random),
            caveats: set_of(&theirs_model.caveats, &pool, &mut random),
            ..Provenance::default()
        };
        match random.below(8) {
            // Theirs shares our evidence, or takes part of it.
            0 => {
                theirs.evidence = ours.evidence.clone();
                theirs_model.evidence = ours_model.evidence.clone();
            }
            1 => {
                let keep = random.below(3);
                theirs.evidence = ours.evidence.clone();
                theirs.evidence.retain(|name| name.len() % 3 != keep);
                theirs_model.evidence = ours_model.evidence.clone();
                theirs_model.evidence.retain(|name| name.len() % 3 != keep);
            }
            _ => {}
        }
        join_and_compare(
            &ours,
            &theirs,
            &ours_model,
            &theirs_model,
            &format!("step {step}"),
        );
    }
}

#[test]
fn joins_near_the_limits_refuse_at_the_same_name() {
    let mut random = Random(0x0b1c_2d3e_4f50_6172);
    // Names of 4 to 120 bytes, 62 on average.
    let pool: BTreeMap<String, Name> = (0..2_500)
        .map(|index| {
            let text = text(index, 4 + (index * 37) % 117);
            (text.clone(), Name::from(text))
        })
        .collect();
    // Joins refused for a name the other side adds: for identifiers, and
    // for bytes.
    let mut refused = [0; 2];
    for step in 0..300 {
        // Some joins draw only short names, so identifiers run out first,
        // some only long ones, so bytes do, and some draw any.
        let bias = random.below(3);
        let drawn: Vec<&String> = pool
            .keys()
            .filter(|text| match bias {
                0 => text.len() <= 60,
                1 => text.len() > 60,
                _ => true,
            })
            .collect();
        let ours_size = match bias {
            1 => 680 + random.below(50),
            _ => 960 + random.below(70),
        };
        let mut models = [Model::default(), Model::default()];
        for (model, size) in models.iter_mut().zip([ours_size, random.below(60)]) {
            while model.evidence.len() + model.caveats.len() < size {
                let name = drawn[random.below(drawn.len())].clone();
                if random.below(5) == 0 {
                    model.caveats.insert(name);
                } else {
                    model.evidence.insert(name);
                }
            }
        }
        let [ours_model, theirs_model] = models;
        let lineage = |model: &Model| {
            let set = |names: &BTreeSet<String>| {
                names
                    .iter()
                    .map(|name| pool[name].clone())
                    .collect::<NameSet>()
            };
            Provenance {
                evidence: set(&model.evidence),
                caveats: set(&model.caveats),
                ..Provenance::default()
            }
        };
        let (ours, theirs) = (lineage(&ours_model), lineage(&theirs_model));
        let whole = |model: &Model| {
            let (count, bytes) = model.size();
            count <= IDENTIFIERS && bytes <= BYTES
        };
        if whole(&ours_model) && whole(&theirs_model) {
            match refusal(&ours_model, &theirs_model) {
                Some(OVER_IDENTIFIERS) => refused[0] += 1,
                Some(_) => refused[1] += 1,
                None => {}
            }
        }
        join_and_compare(
            &ours,
            &theirs,
            &ours_model,
            &theirs_model,
            &format!("step {step}"),
        );
    }
    // Both refusals are met for added names many times.
    assert!(refused.iter().all(|&count| count >= 20), "{refused:?}");
}

/// A lineage holding `evidence` and `caveats`, and its model.
fn shaped(evidence: &[String], caveats: &[String]) -> (Provenance, Model) {
    let model = Model {
        evidence: evidence.iter().cloned().collect(),
        caveats: caveats.iter().cloned().collect(),
    };
    let lineage = Provenance {
        evidence: evidence.iter().cloned().collect(),
        caveats: caveats.iter().cloned().collect(),
        ..Provenance::default()
    };
    (lineage, model)
}

/// Our evidence and caveats, theirs, and the refusal joining them gives.
type Case = (
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Option<&'static str>,
);

#[test]
fn joins_of_one_name_each_and_into_an_empty_set_count_what_they_add() {
    let names = |from: usize, count: usize, bytes: usize| -> Vec<String> {
        (from..from + count)
            .map(|index| text(index, bytes))
            .collect()
    };
    let none = Vec::new();
    let long = |index: usize, bytes: usize| vec![text(index, bytes)];
    // (ours evidence, ours caveats, theirs evidence, theirs caveats, refusal)
    let cases: Vec<Case> = vec![
        // One name each, under and past the byte limit, in either order.
        // Names of one width sort by number; a wider name sorts first, so
        // long(2, 40_000) sorts before long(1, 30_000).
        (
            long(1, 30_000),
            none.clone(),
            long(2, 30_000),
            none.clone(),
            None,
        ),
        (
            long(2, 30_000),
            none.clone(),
            long(1, 30_000),
            none.clone(),
            None,
        ),
        (
            long(1, 40_000),
            none.clone(),
            long(2, 30_000),
            none.clone(),
            Some(OVER_BYTES),
        ),
        (
            long(2, 33_000),
            none.clone(),
            long(1, 33_000),
            none.clone(),
            Some(OVER_BYTES),
        ),
        (
            none.clone(),
            long(2, 33_000),
            none.clone(),
            long(1, 33_000),
            Some(OVER_BYTES),
        ),
        (
            none.clone(),
            long(1, 40_000),
            none.clone(),
            long(2, 30_000),
            Some(OVER_BYTES),
        ),
        // The same name, held as a name of its own on each side: nothing added.
        (
            long(1, 40_000),
            none.clone(),
            long(1, 40_000),
            none.clone(),
            None,
        ),
        // Into empty evidence while caveats hold most of the count or bytes.
        (
            none.clone(),
            names(0, 1_000, 4),
            names(5_000, 30, 4),
            none.clone(),
            Some(OVER_IDENTIFIERS),
        ),
        (
            none.clone(),
            names(0, 1_000, 4),
            names(5_000, 24, 4),
            none.clone(),
            None,
        ),
        (
            none.clone(),
            names(0, 60, 1_000),
            names(100, 10, 1_000),
            none.clone(),
            Some(OVER_BYTES),
        ),
        (
            none.clone(),
            names(0, 60, 1_000),
            names(100, 5, 1_000),
            none.clone(),
            None,
        ),
        // Into empty caveats while evidence holds them.
        (
            names(0, 1_000, 4),
            none.clone(),
            none.clone(),
            names(5_000, 30, 4),
            Some(OVER_IDENTIFIERS),
        ),
        (
            names(0, 60, 1_000),
            none.clone(),
            none.clone(),
            names(100, 10, 1_000),
            Some(OVER_BYTES),
        ),
        // Evidence fits; the caveats added after it do not.
        (
            names(0, 1_000, 4),
            none.clone(),
            names(5_000, 20, 4),
            names(6_000, 10, 4),
            Some(OVER_IDENTIFIERS),
        ),
    ];
    for (index, (ours_evidence, ours_caveats, theirs_evidence, theirs_caveats, expected)) in
        cases.iter().enumerate()
    {
        let (ours, ours_model) = shaped(ours_evidence, ours_caveats);
        let (theirs, theirs_model) = shaped(theirs_evidence, theirs_caveats);
        assert_eq!(
            refusal(&ours_model, &theirs_model),
            *expected,
            "case {index}: the model"
        );
        join_and_compare(
            &ours,
            &theirs,
            &ours_model,
            &theirs_model,
            &format!("case {index}"),
        );
    }
}

/// One departed record of a history, as a provenance holds it in place of
/// the record's name.
fn marker() -> Marker {
    Marker {
        read: 1,
        from: 1,
        through: 1,
        departed_at: 1,
        archive_ref: None,
    }
}

/// A departed history counts its name's bytes, so a lineage it takes past
/// the byte limit is refused, whether checked whole or joined.
#[test]
fn departed_histories_count_their_names_toward_the_byte_limit() {
    // 65,000 bytes of evidence and a history named in 537 bytes: 65,537.
    let mut lineage = Provenance::default();
    lineage.evidence.insert(text(1, 65_000));
    lineage.departed.insert(text(2, 537), marker());
    let error = lineage.validate().unwrap_err().to_string();
    assert!(error.contains(OVER_BYTES), "{error}");
    // Named in 536 bytes, the lineage is at the limit.
    lineage.departed.clear();
    lineage.departed.insert(text(2, 536), marker());
    lineage.validate().unwrap();
    // 100 bytes more for 500 bytes of evidence and a history named in
    // 65,000 bytes: refused, and nothing changes.
    let mut ours = Provenance::default();
    ours.evidence.insert(text(3, 500));
    ours.departed.insert(text(4, 65_000), marker());
    let mut theirs = Provenance::default();
    theirs.evidence.insert(text(5, 100));
    let before = ours.clone();
    let error = ours.merge(&theirs).unwrap_err().to_string();
    assert!(error.contains(OVER_BYTES), "{error}");
    assert!(ours == before, "a refused join changed the lineage");
}

/// 1,023 names in 64,088 bytes: one more name fits the identifier limit and
/// a second does not, and a name of 2,000 bytes passes the byte limit.
fn nearly_full() -> Vec<String> {
    let mut names: Vec<String> = (0..1_022).map(|index| text(index, 4)).collect();
    names.push(text(1_022, 60_000));
    names
}

/// A join into an empty set counts the names it adds in name order, as the
/// walk does: a long name first passes the byte limit, and a long name last
/// passes the identifier limit once the short one is added.
#[test]
fn a_join_into_an_empty_set_counts_what_it_adds_in_name_order() {
    // (the names added, the refusal): "0" < "000...02" < "5".
    let cases = [
        (vec![text(2, 2_000), text(5, 1)], OVER_BYTES),
        (vec![text(0, 1), text(2, 2_000)], OVER_IDENTIFIERS),
    ];
    for (index, (added, expected)) in cases.into_iter().enumerate() {
        // Into empty evidence while caveats hold the names, and into empty
        // caveats while evidence holds them.
        for into_evidence in [true, false] {
            let full: NameSet = nearly_full().into_iter().collect();
            let added: NameSet = added.iter().cloned().collect();
            let (ours, theirs) = if into_evidence {
                (
                    Provenance {
                        caveats: full,
                        ..Provenance::default()
                    },
                    Provenance {
                        evidence: added,
                        ..Provenance::default()
                    },
                )
            } else {
                (
                    Provenance {
                        evidence: full,
                        ..Provenance::default()
                    },
                    Provenance {
                        caveats: added,
                        ..Provenance::default()
                    },
                )
            };
            let mut joined = ours.clone();
            let error = joined.merge(&theirs).unwrap_err().to_string();
            assert!(
                error.contains(expected),
                "case {index}, into evidence {into_evidence}: refused with {error}, not {expected}"
            );
            assert!(
                joined == ours,
                "case {index}: a refused join changed the lineage"
            );
        }
    }
}

/// A join counts the evidence it adds, then the caveats, then the departed
/// histories: with a name of a byte and one of 2,000 bytes added to 1,023
/// names in 64,088 bytes, the order decides which limit the refusal names.
#[test]
fn a_join_counts_evidence_then_caveats_then_departed_histories() {
    let mut evidence = nearly_full();
    let caveat = evidence.pop().unwrap();
    let ours = Provenance {
        evidence: evidence.into_iter().collect(),
        caveats: [caveat].into_iter().collect(),
        ..Provenance::default()
    };
    let short = || text(9, 1);
    let long = || text(8, 2_000);
    // (theirs evidence, caveats and departed history, the refusal)
    let cases = [
        (Some(short()), Some(long()), None, OVER_IDENTIFIERS),
        (Some(long()), Some(short()), None, OVER_BYTES),
        (Some(short()), None, Some(long()), OVER_IDENTIFIERS),
        (None, Some(short()), Some(long()), OVER_IDENTIFIERS),
    ];
    for (index, (evidence, caveat, history, expected)) in cases.into_iter().enumerate() {
        let mut theirs = Provenance {
            evidence: evidence.into_iter().collect(),
            caveats: caveat.into_iter().collect(),
            ..Provenance::default()
        };
        if let Some(history) = history {
            theirs.departed.insert(history, marker());
        }
        let mut joined = ours.clone();
        let error = joined.merge(&theirs).unwrap_err().to_string();
        assert!(
            error.contains(expected),
            "case {index}: refused with {error}, not {expected}"
        );
        assert!(
            joined == ours,
            "case {index}: a refused join changed the lineage"
        );
    }
}
