//! Random clones, merges and edits over a pool of provenances whose name
//! sets may be shared between values: every value keeps exactly the names
//! its own history gave it, whatever happened to the values it was cloned
//! from or merged into, and equality and serialization see the names alone.
use caveat_runtime::reactive::Provenance;
use std::collections::BTreeSet;

const NAMES: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];
const POOL: usize = 6;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
struct Model {
    evidence: BTreeSet<String>,
    caveats: BTreeSet<String>,
    inherited: BTreeSet<String>,
    kept: BTreeSet<String>,
}

impl Model {
    fn merge(&mut self, other: &Model) {
        if !self.inherited.is_empty() || !other.inherited.is_empty() {
            let inherited: BTreeSet<String> = self
                .inherited
                .iter()
                .filter(|name| !other.evidence.contains(*name) || other.inherited.contains(*name))
                .chain(other.inherited.iter().filter(|name| {
                    !self.evidence.contains(*name) || self.inherited.contains(*name)
                }))
                .cloned()
                .collect();
            self.inherited = inherited;
        }
        self.evidence.extend(other.evidence.iter().cloned());
        self.caveats.extend(other.caveats.iter().cloned());
        self.kept.extend(other.kept.iter().cloned());
    }

    fn matches(&self, actual: &Provenance) -> bool {
        actual.evidence == self.evidence
            && actual.caveats == self.caveats
            && actual.inherited == self.inherited
            && actual.kept == self.kept
            && actual.departed.is_empty()
    }
}

#[test]
fn shared_name_sets_never_leak_a_change_between_values() {
    let mut seed: u64 = 0x7a1e_5e75_0bad_cafe;
    let mut next = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };
    let (
        mut clones,
        mut merges,
        mut inherited_merges,
        mut edits,
        mut round_trips,
        mut shared_edits,
    ) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    for run in 0..300 {
        let mut pool: Vec<Provenance> = (0..POOL).map(|_| Provenance::default()).collect();
        let mut model: Vec<Model> = vec![Model::default(); POOL];
        // Which pool members were last made by cloning another, so an edit
        // to one of them exercises a set the other may still share.
        let mut cloned_from: Vec<Option<usize>> = vec![None; POOL];
        for step in 0..80 {
            let at = format!("run {run} step {step}");
            let i = next() % POOL;
            let j = next() % POOL;
            let name = NAMES[next() % NAMES.len()].to_string();
            match next() % 9 {
                0 => {
                    pool[i] = pool[j].clone();
                    model[i] = model[j].clone();
                    cloned_from[i] = Some(j);
                    clones += 1;
                }
                1 | 2 => {
                    let other = pool[j].clone();
                    pool[i]
                        .merge(&other)
                        .unwrap_or_else(|error| panic!("{at}: merge: {error}"));
                    let other_model = model[j].clone();
                    model[i].merge(&other_model);
                    merges += 1;
                }
                3 => {
                    let other = pool[j].clone();
                    pool[i]
                        .merge_inherited(&other)
                        .unwrap_or_else(|error| panic!("{at}: merge_inherited: {error}"));
                    let mut other_model = model[j].clone();
                    other_model.inherited = other_model.evidence.clone();
                    model[i].merge(&other_model);
                    inherited_merges += 1;
                }
                4 => {
                    pool[i].evidence.insert(name.clone());
                    model[i].evidence.insert(name);
                    if cloned_from[i].is_some() {
                        shared_edits += 1;
                    }
                    edits += 1;
                }
                5 => {
                    pool[i].caveats.insert(name.clone());
                    model[i].caveats.insert(name);
                    edits += 1;
                }
                6 => {
                    pool[i].kept.insert(name.clone());
                    model[i].kept.insert(name);
                    edits += 1;
                }
                7 => {
                    pool[i].evidence.remove(&name);
                    pool[i].caveats.remove(&name);
                    model[i].evidence.remove(&name);
                    model[i].caveats.remove(&name);
                    edits += 1;
                }
                _ => {
                    let text = serde_json::to_string(&pool[i]).unwrap();
                    let back: Provenance = serde_json::from_str(&text).unwrap();
                    assert_eq!(back, pool[i], "{at}: serde round trip");
                    pool[i] = back;
                    round_trips += 1;
                }
            }
            for (k, actual) in pool.iter().enumerate() {
                assert!(
                    model[k].matches(actual),
                    "{at}: value {k} holds {actual:?}, expected {:?}",
                    model[k]
                );
            }
            for (k, left) in pool.iter().enumerate() {
                for (l, right) in pool.iter().enumerate() {
                    assert_eq!(
                        left == right,
                        model[k] == model[l],
                        "{at}: equality of {k} and {l}: {left:?} / {right:?}"
                    );
                }
            }
            let names = |set: &BTreeSet<String>| set.iter().cloned().collect::<Vec<_>>();
            let rebuilt =
                Provenance::from_names(names(&model[i].evidence), names(&model[i].caveats))
                    .unwrap();
            assert_eq!(rebuilt.evidence, model[i].evidence, "{at}: from_names");
            assert_eq!(rebuilt.caveats, model[i].caveats, "{at}: from_names");
        }
    }
    eprintln!("{clones} clones, {merges} merges, {inherited_merges} inherited merges, {edits} edits ({shared_edits} on a cloned value), {round_trips} round trips");
    assert!(
        clones > 1500
            && merges > 3000
            && inherited_merges > 1500
            && edits > 6000
            && shared_edits > 500
            && round_trips > 1500,
        "coverage"
    );
}
