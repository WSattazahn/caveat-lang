//! Repetition 0.1: `for KIND as $NAME { ... };`

use caveat_runtime::reactive::{BindingValue, ParameterDomain, ReactiveSession};
use caveat_runtime::{link, repeat};

fn statements(source: &str) -> Vec<String> {
    source
        .split(';')
        .map(|statement| statement.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|statement| !statement.is_empty())
        .collect()
}

const ENTITIES: &str = "entity reef_one kind reef at harbor;\n\
     entity reef_two kind reef at harbor;\n\
     entity reef_three kind reef at harbor;\n";

#[test]
fn a_block_expands_to_exactly_what_a_hand_unrolled_source_says() {
    // The shapes are the ones game/light_the_way.cav actually repeats.
    let block = format!(
        "{ENTITIES}\
         for reef as $r {{\n\
           evidence $r_reading from \"$r\";\n\
           state $r_distance = 100;\n\
           cue $r_ring ring $r \"#ffe3a0\" 0.75;\n\
           on tick when contact == $index set hit_x = $r.x;\n\
           bind $r.reveal.opacity = 0;\n\
           bind $r.reveal.opacity = 0.25 when observed($r_reading);\n\
         }};\n"
    );
    let mut hand = String::from(ENTITIES);
    for (index, reef) in ["reef_one", "reef_two", "reef_three"].iter().enumerate() {
        hand.push_str(&format!(
            "evidence {reef}_reading from \"{reef}\";\n\
             state {reef}_distance = 100;\n\
             cue {reef}_ring ring {reef} \"#ffe3a0\" 0.75;\n\
             on tick when contact == {} set hit_x = {reef}.x;\n\
             bind {reef}.reveal.opacity = 0;\n\
             bind {reef}.reveal.opacity = 0.25 when observed({reef}_reading);\n",
            index + 1
        ));
    }

    let expanded = repeat::expand(&block).expect("block expands");
    assert_eq!(
        statements(&expanded),
        statements(&hand),
        "expansion must produce the statements a hand-unrolled source has"
    );
}

#[test]
fn expansion_is_member_major() {
    // Order is meaning: `on` rules fire in source order and the last matching
    // `bind` wins, so the whole body is emitted per member.
    let expanded = repeat::expand(&format!(
        "{ENTITIES}for reef as $r {{ claim $r_seen; claim $r_charted; }};"
    ))
    .expect("expands");
    assert_eq!(
        statements(&expanded)[3..],
        statements(
            "claim reef_one_seen; claim reef_one_charted;\
             claim reef_two_seen; claim reef_two_charted;\
             claim reef_three_seen; claim reef_three_charted;"
        )
    );
}

#[test]
fn a_source_without_a_block_is_returned_unchanged() {
    let source = "claim mist; entity reef_one kind reef at harbor; budget 2;";
    assert_eq!(repeat::expand(source).expect("no-op"), source);
}

#[test]
fn every_existing_game_expands_to_itself() {
    // Repetition must be invisible to sources that do not use it, including
    // the policy whose bytes Vessel pins.
    for name in [
        "../game/light_the_way.cav",
        "../game/the_last_beacon.cav",
        "../game/moon_garden.cav",
        "../game/missing_dumpling.cav",
        "../examples/slime_glow_ability.cav",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
        let source = std::fs::read_to_string(&path).expect("game source reads");
        assert_eq!(
            repeat::expand(&source).expect("expands"),
            source,
            "{name} must expand to itself byte for byte"
        );
    }
}

#[test]
fn a_block_is_rejected_rather_than_guessed_at() {
    for (source, expected) in [
        (
            format!("{ENTITIES}for buoy as $b {{ claim $b_seen; }};"),
            "no entity is declared `kind buoy`",
        ),
        (
            format!("{ENTITIES}for reef as $r {{ claim $typo_seen; }};"),
            "$typo_seen is not bound",
        ),
        (
            format!("{ENTITIES}for reef as r {{ claim r_seen; }};"),
            "r is not a binding; write $name",
        ),
        (
            format!("{ENTITIES}for reef as $index {{ claim $index; }};"),
            "$index is provided by the block",
        ),
        (
            format!("{ENTITIES}for reef as $r {{ for reef as $q {{ claim $q; }}; }};"),
            "for blocks do not nest",
        ),
        (
            format!("{ENTITIES}for reef {{ claim x; }};"),
            "expected `for KIND as $NAME",
        ),
    ] {
        let message = repeat::expand(&source).expect_err("must be rejected");
        assert!(
            message.contains(expected),
            "expected {expected}, got: {message}"
        );
    }
}

#[test]
fn the_expanded_program_and_the_hand_written_one_run_identically() {
    // The point of a compile-time expansion is that the runtime cannot tell.
    let shared = "scene \"Chart the reefs.\";\n\
         claim clear_course;\n\
         state charted = 0 min 0 max 9;\n\
         event sweep;\n\
         bind hud.charted = charted;\n";
    let entities = "place harbor kind harbor;\n\
         entity reef_one kind reef at harbor;\n\
         entity reef_two kind reef at harbor;\n";
    let body = |reef: &str, index: usize| {
        format!(
            "evidence {reef}_reading from \"{reef}\";\n\
             on sweep when charted == {} reveal {reef}_reading opposes clear_course;\n\
             on sweep when charted == {} set charted = charted + 1;\n",
            index - 1,
            index - 1
        )
    };

    let hand = format!(
        "{shared}{entities}{}{}",
        body("reef_one", 1),
        body("reef_two", 2)
    );
    let looped = format!(
        "{shared}{entities}\
         for reef as $r {{\n\
           evidence $r_reading from \"$r\";\n\
           on sweep when charted == $index - 1 reveal $r_reading opposes clear_course;\n\
           on sweep when charted == $index - 1 set charted = charted + 1;\n\
         }};\n"
    );

    let mut one = ReactiveSession::from_source(&hand).expect("hand-written runs");
    let mut two = ReactiveSession::from_source(&looped).expect("looped runs");
    assert_eq!(one.snapshot().bindings, two.snapshot().bindings);
    for _ in 0..3 {
        let left = one.dispatch_json("sweep", "{}");
        let right = two.dispatch_json("sweep", "{}");
        assert_eq!(left.is_ok(), right.is_ok());
        assert_eq!(
            one.snapshot().bindings,
            two.snapshot().bindings,
            "the two layouts diverged"
        );
    }
    assert_eq!(
        one.snapshot().bindings["hud"]["charted"],
        BindingValue::Number(2.0),
        "both reefs should have been charted"
    );
    assert_eq!(
        one.snapshot().relations.len(),
        two.snapshot().relations.len()
    );
}

#[test]
fn a_block_reproduces_the_real_reef_rules_in_light_the_way() {
    // Checked against the shipped game rather than a fixture: every statement
    // the block generates must already appear in the hand-unrolled source.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../game/light_the_way.cav");
    let game = std::fs::read_to_string(&path).expect("light_the_way reads");
    let entities: String = game
        .lines()
        .filter(|line| line.starts_with("entity reef_"))
        .map(|line| format!("{line}\n"))
        .collect();
    assert_eq!(entities.lines().count(), 6, "six reefs are declared");

    let block = format!(
        "{entities}\
         for reef as $r {{\n\
           evidence $r_reading from \"$r\";\n\
           on tick when phase == 1 and paused == 0 and light_on == 1 and distance(aim_x, aim_z, $r.x, $r.z) <= beam_radius and not observed($r_reading) reveal $r_reading opposes clear_course;\n\
           on tick when phase == 1 and paused == 0 set $r_distance = distance(boat_x, boat_z, $r.x, $r.z);\n\
           on tick when phase == 1 and paused == 0 and contact == $index set hit_x = $r.x;\n\
           bind $r.reveal.opacity = 0;\n\
         }};\n"
    );
    let expanded = repeat::expand(&block).expect("block expands");
    let shipped = statements(&game);
    let generated = statements(&expanded);
    assert_eq!(
        generated.len(),
        6 + 6 * 5,
        "six members times five statements"
    );

    let mut checked = 0;
    for statement in generated.iter().filter(|s| !s.starts_with("entity ")) {
        assert!(
            shipped.contains(statement),
            "generated a statement the shipped game does not have:\n  {statement}"
        );
        checked += 1;
    }
    assert_eq!(
        checked, 30,
        "every generated rule was matched against the game"
    );
}

#[test]
fn a_block_survives_linking_into_a_module() {
    // Expansion runs per part before names are rewritten, so the generated
    // names are scoped to the module that wrote the block.
    let module = format!(
        "module reefs;\n{ENTITIES}for reef as $r {{ claim $r_seen; evidence $r_reading from \"$r\"; $r_reading supports $r_seen; }};\n"
    );
    let text = link::bundle(&[
        link::BundlePart {
            name: "reefs".into(),
            source: module,
        },
        link::BundlePart {
            name: "main".into(),
            source: "use reefs;\nbudget 1;\nclaim here;\n".into(),
        },
    ]);
    let linked = link::link(&text).expect("bundle links");
    assert!(
        linked.contains("claim reefs__reef_one_seen;"),
        "generated names belong to the module that wrote the block: {linked}"
    );
    assert!(
        linked.contains("reefs__reef_two_reading supports reefs__reef_two_seen;"),
        "{linked}"
    );
    let evaluation =
        caveat_runtime::eval::evaluate(&caveat_runtime::parser::parse(&linked).expect("parses"))
            .expect("evaluates");
    assert!(evaluation.symbols.contains_key("reefs__reef_three_seen"));
}

#[test]
fn a_block_cannot_iterate_a_kind_another_part_declared() {
    // spec/caveat-repetition-0.1.md section 4. Expansion runs per part, so a
    // module's entities are not in scope for the program's block. Refusing is
    // what keeps one part's generated names from depending on another's
    // declarations.
    let text = link::bundle(&[
        link::BundlePart {
            name: "harbour".into(),
            source: format!("module harbour;\n{ENTITIES}"),
        },
        link::BundlePart {
            name: "main".into(),
            source: "use harbour;\nfor reef as $r { claim $r_seen; };\n".into(),
        },
    ]);
    let message = link::link(&text).expect_err("iterating another part's kind must be rejected");
    assert!(
        message.contains("no entity is declared `kind reef`"),
        "{message}"
    );

    // The module may iterate its own, and the generated names are its own.
    let text = link::bundle(&[
        link::BundlePart {
            name: "harbour".into(),
            source: format!("module harbour;\n{ENTITIES}for reef as $r {{ claim $r_seen; }};\n"),
        },
        link::BundlePart {
            name: "main".into(),
            source: "use harbour;\nbudget 1;\n".into(),
        },
    ]);
    let linked = link::link(&text).expect("a module iterates its own kind");
    assert!(linked.contains("claim harbour__reef_one_seen;"), "{linked}");
}

#[test]
fn the_word_for_in_a_comment_or_text_is_not_a_nested_block() {
    // The glowcap experiment's first failed run: a comment explaining what a
    // rule was *for* was rejected as a nested `for` block.
    let source = format!(
        "{ENTITIES}for reef as $r {{\n\
         // Plain copies for the fade rules.\n\
         # Kept for later.\n\
         evidence $r_sighting from \"good for $r\";\n\
         }};"
    );
    let expanded = repeat::expand(&source).expect("comments and text may say for");
    assert!(
        expanded.contains("evidence reef_two_sighting from \"good for reef_two\";"),
        "{expanded}"
    );
    // A real nested block is still refused.
    let nested =
        format!("{ENTITIES}for reef as $r {{ // note\n for reef as $q {{ claim $q; }}; }};");
    assert!(repeat::expand(&nested)
        .unwrap_err()
        .contains("for blocks do not nest"));
}

/// Issue #47's program. The loader declares both plots, and north's
/// statement has a comment inside it.
const COMMENTED_NORTH: &str = "place field kind field;
entity north # the first plot
    kind plot at field;
entity south kind plot at field;
state north_n = 0;
state south_n = 0;
event read target kind plot;
for plot as $p {
    on read when target == $index set $p_n = $p_n + 1;
};
";

/// `read` sent for `target` on a fresh session: north's count, then south's.
fn after_read(source: &str, target: &str) -> (f64, f64) {
    let mut session = ReactiveSession::from_source(source).expect("loads");
    let values = session
        .dispatch_json("read", &format!(r#"{{"target":"{target}"}}"#))
        .expect("read is accepted")
        .values;
    (values["north_n"], values["south_n"])
}

#[test]
fn a_comment_in_an_entity_statement_does_not_change_which_member_an_event_affects() {
    // Issue #47. `target` counts north as 1 and south as 2. Repetition used
    // to skip north's statement and give south `$index` 1, so `read north`
    // counted for south and `read south` for nobody.
    assert_eq!(after_read(COMMENTED_NORTH, "north"), (1.0, 0.0));
    assert_eq!(after_read(COMMENTED_NORTH, "south"), (0.0, 1.0));
}

/// Plots declared by `entities`, then a plain block that keeps each member's
/// `$index` and counts the reads that select it by hand.
fn indexed(entities: &str) -> String {
    format!(
        "place field kind field;
event read target kind plot;
for plot as $p {{
    state $p_index = $index;
    state $p_n = 0;
    on read when target == $index set $p_n = $p_n + 1;
}};
{entities}"
    )
}

/// Each member's `$index` as its state holds it, by member, and the members
/// of `read`'s `target`, in the order that numbers them.
fn counts(source: &str) -> (Vec<(String, f64)>, Vec<String>) {
    let snapshot = ReactiveSession::from_source(source)
        .unwrap_or_else(|error| panic!("{error}\n{source}"))
        .snapshot();
    let read = snapshot
        .events
        .iter()
        .find(|event| event.name == "read")
        .expect("read is declared");
    let ParameterDomain::Entity { members, .. } = &read.parameters[0].domain else {
        panic!("target names a plot");
    };
    let indices = members
        .iter()
        .map(|member| (member.clone(), snapshot.values[&format!("{member}_index")]))
        .collect();
    let states = snapshot
        .values
        .keys()
        .filter(|name| name.ends_with("_index"))
        .count();
    assert_eq!(states, members.len(), "one copy per member:\n{source}");
    (indices, members.clone())
}

#[test]
fn index_counts_every_entity_the_loader_declares_as_its_typed_parameter_does() {
    let north_first = vec![("north".to_string(), 1.0), ("south".to_string(), 2.0)];
    for entities in [
        // Issue #47's statement, and the same with the other comment form.
        "entity north # the first plot\n    kind plot at field;\nentity south kind plot at field;\n",
        "entity north // the first plot\n    kind plot at field;\nentity south kind plot at field;\n",
        // A comment ends the word it touches.
        "entity north#first\n    kind plot at field;\nentity south kind plot at field;\n",
        // A `;` or a quote in a comment is part of the comment.
        "entity north # the first; plot\n    kind plot at field;\nentity south kind plot at field;\n",
        "entity north // the \"first plot\n    kind plot at field;\nentity south kind plot at field;\n",
        // `#`, `//` and `;` in quoted text are text, not a comment and not
        // the end of a statement.
        "evidence note from \"plot #1; // north\"; entity north kind plot at field;\nentity south kind plot at field;\n",
        "entity north kind plot at field; evidence note from \"# south //\";\nentity south kind plot at field;\n",
        // A last statement without its `;`, which the loader reads.
        "entity north kind plot at field;\nentity south kind plot at field\n",
        "entity north kind plot at field;\nentity south # the last; no `;` ends it\n    kind plot at field\n",
    ] {
        let source = indexed(entities);
        let (indices, members) = counts(&source);
        assert_eq!(indices, north_first, "{source}");
        assert_eq!(members, ["north", "south"], "{source}");
        // Both directions: a read selects its own plot's copy and no other.
        for (target, expected) in [("north", (1.0, 0.0)), ("south", (0.0, 1.0))] {
            assert_eq!(after_read(&source, target), expected, "read {target}\n{source}");
        }
    }
}

#[test]
fn a_commented_entity_gets_its_own_copy_and_index() {
    // The issue's second observation: the block expanded to one state,
    // `south_index = 1`, and none for north.
    let source = COMMENTED_NORTH.replace(
        "    on read when target == $index set $p_n = $p_n + 1;\n",
        "    state $p_index = $index;\n",
    );
    let expanded = repeat::expand(&source).expect("expands");
    assert!(
        expanded.ends_with("\n    state north_index = 1;\n\n    state south_index = 2;\n\n"),
        "{expanded}"
    );
    let (indices, members) = counts(&source);
    assert_eq!(
        indices,
        [("north".to_string(), 1.0), ("south".to_string(), 2.0)]
    );
    assert_eq!(members, ["north", "south"]);
}

#[test]
fn a_comment_touching_an_entity_name_is_not_part_of_the_name() {
    // The loader ends a word at a comment, so this plot is `north`, as
    // `target` names it. Repetition used to read `north#first`: `"$p"`
    // became `"north#first"`, silently, and a copy that used `$p` outside
    // quoted text did not load.
    let entities =
        "entity north#first\n    kind plot at field;\nentity south kind plot at field;\n";
    let source = format!(
        "place field kind field;
{entities}event read target kind plot;
for plot as $p {{
    on read when target == $index reject \"$p is locked.\";
}};
"
    );
    for (target, rule) in [("north", 1), ("south", 2)] {
        let error = ReactiveSession::from_source(&source)
            .expect("loads")
            .dispatch_json("read", &format!(r#"{{"target":"{target}"}}"#))
            .expect_err("the member's own copy refuses the read");
        assert_eq!(
            error,
            format!("event read, rule {rule}: rejected: {target} is locked.")
        );
    }
    let source = indexed(entities);
    let expanded = repeat::expand(&source).expect("expands");
    assert!(
        expanded.contains("\n    state north_index = 1;\n"),
        "{expanded}"
    );
    let (indices, members) = counts(&source);
    assert_eq!(
        indices,
        [("north".to_string(), 1.0), ("south".to_string(), 2.0)]
    );
    assert_eq!(members, ["north", "south"]);
}

/// Plots and statements written around a block: `before` above it, `after`
/// below. Each member keeps its `$index` and counts the reads that select it,
/// by hand in a plain block and by the route in a routed one.
fn around(before: &str, after: &str, routed: bool) -> String {
    let (header, guard) = match routed {
        false => ("for plot as $p {", " when target == $index"),
        true => ("for plot as $p routed by target {", ""),
    };
    format!(
        "place field kind field;
{before}event read target kind plot;
{header}
    state $p_index = $index;
    state $p_n = 0;
    on read{guard} set $p_n = $p_n + 1;
}};
{after}"
    )
}

/// Each member's count after `read` names `target` on a fresh session, in
/// the order `target` numbers them.
fn after_read_by_member(source: &str, members: &[String], target: &str) -> Vec<f64> {
    let mut session = ReactiveSession::from_source(source).expect("loads");
    let values = session
        .dispatch_json("read", &format!(r#"{{"target":"{target}"}}"#))
        .expect("read is accepted")
        .values;
    members
        .iter()
        .map(|member| values[&format!("{member}_n")])
        .collect()
}

#[test]
fn a_brace_outside_a_procedure_or_block_does_not_hide_the_members_after_it() {
    // The loader counts `{` and `}` only in a `proc` statement, so this
    // evidence, with unquoted provenance the loader accepts, ends at its
    // `;`. Repetition counted its brace and read everything after it as one
    // unterminated statement: below the block, north and south got no copy
    // and `read south` changed nothing; above it, or in its body, the block
    // was not expanded and the program did not load. In a body it is still
    // refused, now as a `{` its statement does not close.
    let evidence = "evidence manual from see{appendix;\n";
    let east = "entity east kind plot at field;\n";
    let rest = "entity north kind plot at field;\nentity south kind plot at field;\n";
    let east_first = [("east", 1.0), ("north", 2.0), ("south", 3.0)]
        .map(|(member, index)| (member.to_string(), index))
        .to_vec();
    for routed in [false, true] {
        let in_body = around(east, rest, routed).replace(
            "    state $p_index",
            "    evidence $p_manual from see{appendix;\n    state $p_index",
        );
        assert_eq!(
            repeat::expand(&in_body).unwrap_err(),
            unclosed(5, 32, routed, "evidence $p_manual from see{appendix"),
            "{in_body}"
        );
        for source in [
            around(east, &format!("{evidence}{rest}"), routed),
            around(&format!("{evidence}{east}"), rest, routed),
        ] {
            let (indices, members) = counts(&source);
            assert_eq!(indices, east_first, "{source}");
            // Both directions: a read selects its own plot's copy and no other.
            for (position, target) in members.iter().enumerate() {
                let expected = (0..members.len())
                    .map(|member| f64::from(u8::from(member == position)))
                    .collect::<Vec<_>>();
                assert_eq!(
                    after_read_by_member(&source, &members, target),
                    expected,
                    "read {target}\n{source}"
                );
            }
        }
    }
}

#[test]
fn a_first_word_that_only_begins_with_for_is_no_block() {
    // A block is a statement whose first word is `for`, and the loader
    // accepts an evidence named `for{`, so `for{ supports c;` is a relation,
    // as Repetition's own block test read it. Its statement reader took that
    // `{` for a block's body and read everything after it as the body: below
    // the block, south got no copy while `target` counted it, and `read
    // south` changed nothing, silently. Above the block, the block was not
    // expanded and the program did not load. In its body, the block was
    // refused as having text after its body; it is refused now as a `{` its
    // statement does not close, as `for{ supports $p_claim;` leaves it.
    let evidence = "claim c;\nevidence for{ from log;\n";
    let relation = "for{ supports c;\n";
    let east = "entity east kind plot at field;\n";
    let south = "entity south kind plot at field;\n";
    let east_first = [("east", 1.0), ("south", 2.0)]
        .map(|(member, index)| (member.to_string(), index))
        .to_vec();
    for routed in [false, true] {
        let in_body = around(&format!("{evidence}{east}"), south, routed).replace(
            "    state $p_index",
            "    claim $p_claim;\n    for{ supports $p_claim;\n    state $p_index",
        );
        assert_eq!(
            repeat::expand(&in_body).unwrap_err(),
            unclosed(8, 8, routed, "for{ supports $p_claim"),
            "{in_body}"
        );
        for source in [
            around(east, &format!("{evidence}{relation}{south}"), routed),
            around(&format!("{evidence}{relation}{east}"), south, routed),
        ] {
            let expanded =
                repeat::expand(&source).unwrap_or_else(|error| panic!("{error}\n{source}"));
            assert_eq!(
                expanded.matches("_index = ").count(),
                2,
                "a copy for east and one for south:\n{expanded}"
            );
            let (indices, members) = counts(&source);
            assert_eq!(indices, east_first, "{source}");
            // Both directions: a read selects its own plot's copy and no other.
            for (position, target) in members.iter().enumerate() {
                let expected = (0..members.len())
                    .map(|member| f64::from(u8::from(member == position)))
                    .collect::<Vec<_>>();
                assert_eq!(
                    after_read_by_member(&source, &members, target),
                    expected,
                    "read {target}\n{source}"
                );
            }
        }
    }
}

/// Plots and statements around a block whose body names its states by
/// `$index`, and writes its member's name only where the loader reads any
/// word: in unquoted provenance and in quoted text. Each copy counts the
/// reads that select it, by hand in a plain block and by the route in a
/// routed one, and refuses a `name` that selects it with its member's name
/// and `$index`.
fn by_index(before: &str, after: &str, routed: bool) -> String {
    let (header, guard) = match routed {
        false => ("for plot as $p {", " when target == $index"),
        true => ("for plot as $p routed by target {", ""),
    };
    format!(
        "place field kind field;
{before}event read target kind plot;
event name target kind plot;
{header}
    state n_$index = 0;
    evidence e_$index from $p;
    on read{guard} set n_$index = n_$index + 1;
    on name{guard} reject \"$p is $index.\";
}};
{after}"
    )
}

/// `north{` above a block and below it, among plots east and south.
fn around_north_brace() -> [(String, String); 2] {
    let east = "entity east kind plot at field;\n";
    let north = "entity north{ kind plot at field;\n";
    let south = "entity south kind plot at field;\n";
    [
        (east.to_string(), format!("{north}{south}")),
        (format!("{east}{north}"), south.to_string()),
    ]
}

#[test]
fn a_member_whose_name_is_not_an_identifier_gets_its_copy_and_index() {
    // The loader declares an entity by the word after `entity`, so `north{`
    // is a plot and a member of `target`, and Repetition counts it as well.
    // Before a statement ended at its `;` where the loader ends it, its
    // brace hid it: below the block, north{ and south got no copy and
    // `read south` changed nothing; above it, the block was not expanded and
    // the program did not load.
    let plots = ["east", "north{", "south"];
    for routed in [false, true] {
        for (before, after) in around_north_brace() {
            let source = by_index(&before, &after, routed);
            let snapshot = ReactiveSession::from_source(&source)
                .unwrap_or_else(|error| panic!("{error}\n{source}"))
                .snapshot();
            let read = snapshot
                .events
                .iter()
                .find(|event| event.name == "read")
                .expect("read is declared");
            let ParameterDomain::Entity { members, .. } = &read.parameters[0].domain else {
                panic!("target names a plot");
            };
            assert_eq!(members, &plots, "{source}");
            // Repetition's members: a copy for each, in the same order, with
            // the name as the loader declares it.
            let expanded = repeat::expand(&source).expect("expands");
            assert_eq!(expanded.matches("evidence e_").count(), plots.len());
            for (position, member) in plots.iter().enumerate() {
                let index = position + 1;
                assert!(
                    expanded.contains(&format!("evidence e_{index} from {member};\n")),
                    "{expanded}"
                );
                // Both directions: a read selects its own plot's copy and no
                // other, and a `name` reaches that copy's text.
                let mut session = ReactiveSession::from_source(&source).expect("loads");
                let payload = serde_json::json!({ "target": member }).to_string();
                let values = session
                    .dispatch_json("read", &payload)
                    .expect("read is accepted")
                    .values;
                let counts = (1..=plots.len())
                    .map(|copy| values[&format!("n_{copy}")])
                    .collect::<Vec<_>>();
                let expected = (1..=plots.len())
                    .map(|copy| f64::from(u8::from(copy == index)))
                    .collect::<Vec<_>>();
                assert_eq!(counts, expected, "read {member}\n{source}");
                assert_eq!(
                    session
                        .dispatch_json("name", &payload)
                        .expect_err("the member's own copy refuses"),
                    format!(
                        "event name, rule {}: rejected: {member} is {index}.",
                        2 * index
                    ),
                    "{source}"
                );
            }
        }
    }
}

#[test]
fn a_member_name_that_is_not_an_identifier_does_not_load_where_the_body_needs_one() {
    // `state $p_index` is `state north{_index` in north{'s copy, which no
    // reactive identifier is, so the program does not load, and the error
    // gives the copy's line in the expanded text. Before, below the block,
    // the program loaded without copies for north{ and south, and `read
    // south` changed nothing.
    for routed in [false, true] {
        for ((before, after), line) in around_north_brace().into_iter().zip([9, 10]) {
            let source = around(&before, &after, routed);
            let error = match ReactiveSession::from_source(&source) {
                Ok(_) => panic!("loaded:\n{source}"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                format!("line {line}, column 5: invalid reactive identifier north{{_index"),
                "{source}"
            );
        }
    }
}

/// Plots north and south, then a block with `header` over them. Each copy
/// keeps its `$index` and counts the reads that select it, by hand unless
/// the block is routed.
fn headed(header: &str, routed: bool) -> String {
    let guard = if routed { "" } else { " when target == $index" };
    format!(
        "place field kind field;
entity north kind plot at field;
entity south kind plot at field;
event read target kind plot;
{header}
    state $p_index = $index;
    state $p_n = 0;
    on read{guard} set $p_n = $p_n + 1;
}};
"
    )
}

#[test]
fn a_comment_in_a_for_header_is_whitespace() {
    // Comments count as whitespace (spec/caveat-text-0.1.md). Repetition read
    // a header's words, and its braces, with its comments left in, and
    // refused the first block: "expected `for KIND as $NAME { ... }`, found:
    // for plot # each plot".
    let mut sources = [
        ("for plot # each plot\n    as $p {", false),
        ("for plot // each plot\n    as $p {", false),
        ("for# each plot\n    plot as $p {", false),
        ("for plot as $p # a { in a comment\n{", false),
        ("for plot # each plot\n    as $p routed by target {", true),
        (
            "for plot as $p routed # by nothing yet\n    by target {",
            true,
        ),
    ]
    .map(|(header, routed)| headed(header, routed))
    .to_vec();
    // A `}` in a comment after the body does not close it.
    sources.push(headed("for plot as $p {", false).replace("\n};\n", "\n} # }\n;\n"));
    for source in sources {
        let (indices, members) = counts(&source);
        assert_eq!(
            indices,
            [("north".to_string(), 1.0), ("south".to_string(), 2.0)],
            "{source}"
        );
        assert_eq!(members, ["north", "south"]);
        for (target, expected) in [("north", (1.0, 0.0)), ("south", (0.0, 1.0))] {
            assert_eq!(
                after_read(&source, target),
                expected,
                "read {target}\n{source}"
            );
        }
    }
    // A malformed header is shown as the loader reads it, without comments.
    assert_eq!(
        repeat::expand(&headed("for plot # as $p\n{", false)).unwrap_err(),
        "expected `for KIND as $NAME { ... }`, found: for plot { state $p_index"
    );
    assert_eq!(
        repeat::expand(&headed("for plot as $p routed # by target\n{", true)).unwrap_err(),
        "expected `for KIND as $NAME routed by PARAM { ... }`, found: for plot as $p routed"
    );
}

#[test]
fn a_last_block_without_its_semicolon_is_expanded() {
    // The loader reads a last statement without its `;`
    // (spec/caveat-text-0.1.md). Repetition did not read a block there, so
    // the program did not load: "cannot parse statement: for plot as $p {".
    for (header, routed) in [
        ("for plot as $p {", false),
        ("for plot as $p routed by target {", true),
    ] {
        for end in ["}", "}\n", "} # the last block; no `;`\n", "}\n// the end"] {
            let source = headed(header, routed).replace("\n};\n", &format!("\n{end}"));
            let (indices, members) = counts(&source);
            assert_eq!(
                indices,
                [("north".to_string(), 1.0), ("south".to_string(), 2.0)],
                "{source}"
            );
            assert_eq!(members, ["north", "south"]);
            for (target, expected) in [("north", (1.0, 0.0)), ("south", (0.0, 1.0))] {
                assert_eq!(
                    after_read(&source, target),
                    expected,
                    "read {target}\n{source}"
                );
            }
        }
    }
    // A block that is the last statement of a body is a nested block. It was
    // not read, and its `$q` was refused as unbound.
    let nested =
        format!("{ENTITIES}for reef as $r {{ claim $r_seen; for reef as $q {{ claim $q; }} }};");
    assert_eq!(
        repeat::expand(&nested).unwrap_err(),
        "for blocks do not nest; see spec/caveat-repetition-0.1.md section 4: for reef as $r {"
    );
}

#[test]
fn text_between_a_block_and_its_end_is_refused_not_dropped() {
    // Repetition replaced a block with its body's copies, and dropped what
    // followed the body. With a `;` after it, `entity east` was dropped
    // silently: `target` named north and south only. As the last statement,
    // which the loader reads, the block was not expanded.
    for end in [
        "} entity east kind plot at field;\n",
        "}\nentity east kind plot at field\n",
    ] {
        let source = headed("for plot as $p {", false).replace("\n};\n", &format!("\n{end}"));
        assert_eq!(
            repeat::expand(&source).unwrap_err(),
            "for block has text after its body: entity east kind plot at; end the block with `};`",
            "{source}"
        );
    }
}

#[test]
fn a_brace_pair_in_a_body_statement_stays_in_the_statement() {
    // The loader reads a brace outside a `proc` statement as text, so this
    // provenance is the statement's own. A `}` in a body statement that
    // closes a `{` earlier in it is text here too. Once a brace outside a
    // procedure stopped hiding the members after it, every such `}` closed
    // the body, and a block that loaded before was refused: "for block has
    // text after its body: b; end the block with `};`".
    let plots = "entity north kind plot at field;\nentity south kind plot at field;\n";
    for routed in [false, true] {
        for provenance in ["see{appendix}b", "{a}{b}", "x{{y}}z"] {
            let source = around(plots, "", routed).replace(
                "    state $p_index",
                &format!("    evidence $p_manual from {provenance};\n    state $p_index"),
            );
            let expanded = repeat::expand(&source).expect("expands");
            for member in ["north", "south"] {
                assert!(
                    expanded.contains(&format!("evidence {member}_manual from {provenance};\n")),
                    "{expanded}"
                );
            }
            let (indices, members) = counts(&source);
            assert_eq!(
                indices,
                [("north".to_string(), 1.0), ("south".to_string(), 2.0)],
                "{source}"
            );
            assert_eq!(members, ["north", "south"]);
            for (target, expected) in [("north", (1.0, 0.0)), ("south", (0.0, 1.0))] {
                assert_eq!(
                    after_read(&source, target),
                    expected,
                    "read {target}\n{source}"
                );
            }
        }
    }
    // A `}` that closes no `{` of its statement closes the body, so what
    // follows it is refused, not read as the statement's text.
    let source = around(plots, "", false).replace(
        "    state $p_index",
        "    evidence $p_manual from a}b;\n    state $p_index",
    );
    assert_eq!(
        repeat::expand(&source).unwrap_err(),
        "for block has text after its body: b; end the block with `};`"
    );
    // So is such a `}` in the body's last statement without its `;`.
    // Repetition took the block statement's last `}` for the body's end, so
    // this body read on past the `}` that ended it, and the block expanded.
    let source = around(plots, "", false).replace(
        "$p_n + 1;\n};\n",
        "$p_n + 1;\n    evidence $p_manual from a}b\n};\n",
    );
    assert_eq!(
        repeat::expand(&source).unwrap_err(),
        "for block has text after its body: b }; end the block with `};`"
    );
    // A `{` there is closed by the `}` after it, so the body is not closed.
    // Repetition took that `}` for the body's end too, and the block
    // expanded.
    let source = around(plots, "", false).replace(
        "$p_n + 1;\n};\n",
        "$p_n + 1;\n    evidence $p_manual from see{appendix\n};\n",
    );
    assert_eq!(
        repeat::expand(&source).unwrap_err(),
        "for block is not closed: for plot as $p {"
    );
}

/// The refusal of a `{` at `line` and `column` that the body statement
/// `written` of a block over plots, plain or routed by `target`, leaves open.
fn unclosed(line: usize, column: usize, routed: bool, written: &str) -> String {
    let header = if routed {
        "for plot as $p routed by target"
    } else {
        "for plot as $p"
    };
    format!(
        "line {line}, column {column}: for block `{header}`: `{written}` has a `{{` that its statement does not close; a brace in unquoted text pairs only within its statement, so quote the text"
    )
}

/// Repetition 0.1's example of a split pair: `x{` in one body statement and
/// `y}` in the next, over the first `members` of north, south and east, on
/// one line or a statement to a line, with `after` below the block.
fn split_pair(members: usize, one_line: bool, after: &str) -> String {
    let plots = ["north", "south", "east"][..members]
        .iter()
        .map(|plot| format!("entity {plot} kind plot at field;\n"))
        .collect::<String>();
    let block = if one_line {
        "for plot as $p { evidence $p_a from x{; evidence $p_b from y}; };\n"
    } else {
        "for plot as $p {\n    evidence $p_a from x{;\n    evidence $p_b from y};\n};\n"
    };
    format!("place field kind field;\n{plots}event go;\n{block}{after}")
}

#[test]
fn a_brace_in_unquoted_body_text_pairs_only_within_its_statement() {
    // A `}` in a body that closes no `{` of its statement ends the body, so
    // `x{` and `y}` in two statements are no pair. The body ended at `y}`,
    // its last statement had no `;`, and each copy but the last read on into
    // the next: with north and south, the program loaded without a word and
    // declared north_a, north_b and south_b, where main paired the braces and
    // declared all four. A `{` that its statement leaves open is refused, for
    // any number of members.
    let after = "claim done;\nstate n = 0;\non go set n = n + 1;\n";
    for members in 1..=3 {
        for one_line in [true, false] {
            for after in ["", after] {
                let source = split_pair(members, one_line, after);
                let expected = if one_line {
                    unclosed(members + 3, 38, false, "evidence $p_a from x{")
                } else {
                    unclosed(members + 4, 25, false, "evidence $p_a from x{")
                };
                assert_eq!(repeat::expand(&source).unwrap_err(), expected, "{source}");
                match ReactiveSession::from_source(&source) {
                    Ok(_) => panic!("loaded:\n{source}"),
                    Err(error) => assert_eq!(error, expected, "{source}"),
                }
                // Quoted, each brace is its statement's text, and every copy
                // is declared with it.
                let quoted = source
                    .replace("from x{", "from \"x{\"")
                    .replace("from y}", "from \"y}\"");
                let symbols = ReactiveSession::from_source(&quoted)
                    .unwrap_or_else(|error| panic!("{error}\n{quoted}"))
                    .snapshot()
                    .symbols
                    .into_iter()
                    .filter(|symbol| symbol.kind == "evidence")
                    .map(|symbol| (symbol.name, symbol.source.unwrap_or_default()))
                    .collect::<Vec<_>>();
                // The snapshot lists symbols by name.
                let mut expected = ["north", "south", "east"][..members]
                    .iter()
                    .flat_map(|plot| {
                        [
                            (format!("{plot}_a"), "x{".to_string()),
                            (format!("{plot}_b"), "y}".to_string()),
                        ]
                    })
                    .collect::<Vec<_>>();
                expected.sort();
                assert_eq!(symbols, expected, "{quoted}");
            }
        }
    }
    // The same in a routed block. Its routed rule, after the pair, was read
    // below the body, and the block was refused as routing no rule.
    let source = "place field kind field;
entity north kind plot at field;
entity south kind plot at field;
event read target kind plot;
for plot as $p routed by target {
    evidence $p_a from x{;
    evidence $p_b from y};
    state $p_n = 0;
    on read set $p_n = $p_n + 1;
};
";
    assert_eq!(
        repeat::expand(source).unwrap_err(),
        unclosed(6, 25, true, "evidence $p_a from x{")
    );
}

#[test]
fn a_last_statement_after_an_empty_statement_is_read() {
    // The loader accepts empty statements (spec/caveat-text-0.1.md), and a
    // last statement without its `;` begins at its first word. Repetition
    // began it at the empty statement's `;`, so it read no entity or block
    // there. Below a block, south got no copy while `target` named it, and
    // `read south` changed nothing, silently. A last block was not
    // expanded: "cannot parse statement: for plot as $p {".
    let north = "entity north kind plot at field;\n";
    let south = "entity south kind plot at field";
    let mut sources = Vec::new();
    for routed in [false, true] {
        for empty in [";", " ;\n", "\n# nothing\n;\n", ";;\n"] {
            sources.push(around(north, &format!("{empty}{south}"), routed));
        }
        let header = if routed {
            "for plot as $p routed by target {"
        } else {
            "for plot as $p {"
        };
        for empty in [";", "; # nothing\n;"] {
            sources.push(
                headed(header, routed)
                    .replace(
                        "event read target kind plot;",
                        &format!("event read target kind plot;{empty}"),
                    )
                    .replace("\n};\n", "\n}\n"),
            );
        }
    }
    for source in sources {
        let (indices, members) = counts(&source);
        assert_eq!(
            indices,
            [("north".to_string(), 1.0), ("south".to_string(), 2.0)],
            "{source}"
        );
        assert_eq!(members, ["north", "south"]);
        for (target, expected) in [("north", (1.0, 0.0)), ("south", (0.0, 1.0))] {
            assert_eq!(
                after_read(&source, target),
                expected,
                "read {target}\n{source}"
            );
        }
    }
}
