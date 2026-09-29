//! Routed Repetition 0.1: `for KIND as $NAME routed by P { ... };`. See
//! spec/caveat-routed-repetition-0.1.md.

use caveat_runtime::reactive::{
    BindingValue, DispatchResult, ParameterDomain, ReactiveSession, ReactiveSnapshot,
};
use caveat_runtime::{link, repeat};
use serde_json::Value;

/// Two plots and a probe. `read` and `ping` name a plot by `target`, `swap`
/// names a probe, and `tick` names nothing.
const PLOTS: &str = "place field kind field;
entity north kind plot at field;
entity south kind plot at field;
entity handheld kind probe at field;
state total = 0;
event read target kind plot, value min 0 max 9;
event ping target kind plot, loud min 0 max 1;
event swap device kind probe;
event tick dt min 0 max 0.1;
";

const ROUTED: &str = "for plot as $p routed by target";
const PLAIN: &str = "for plot as $p";

/// The plots with one block, `header { body };`, after them.
fn program(header: &str, body: &str) -> String {
    format!("{PLOTS}{header} {{{body}}};\n")
}

/// What the block expands to: the text that replaces it, and the line after.
fn expanded(header: &str, body: &str) -> Result<String, String> {
    repeat::expand(&program(header, body)).map(|text| text[PLOTS.len()..].to_string())
}

fn routed(body: &str) -> String {
    expanded(ROUTED, body).unwrap_or_else(|error| panic!("{error}\n{body}"))
}

fn plain(body: &str) -> String {
    expanded(PLAIN, body).unwrap_or_else(|error| panic!("{error}\n{body}"))
}

fn refused(header: &str, body: &str) -> String {
    match expanded(header, body) {
        Ok(text) => panic!("expected a refusal, expanded to:\n{text}"),
        Err(error) => error,
    }
}

fn read(file: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{file}: {error}"))
}

fn hits(session: &ReactiveSession, plot: &str) -> BindingValue {
    session.snapshot().bindings[plot]["hits"].clone()
}

// ── Section 3: where the route goes ────────────────────────────────────────

#[test]
fn a_routed_rule_expands_as_if_its_route_were_written_first_in_its_guard() {
    // The table in section 3: as written in the block, and as if written.
    for (written, as_if) in [
        (
            "on read set $p_n = value;",
            "on read when target == $index set $p_n = value;",
        ),
        (
            "on read when value > 2 set $p_n = value;",
            "on read when target == $index and value > 2 set $p_n = value;",
        ),
        (
            "on read when value > 2 or value == 0 set $p_n = value;",
            "on read when target == $index and (value > 2 or value == 0) set $p_n = value;",
        ),
    ] {
        assert_eq!(
            routed(&format!("\n    {written}\n")),
            plain(&format!("\n    {as_if}\n")),
            "{written}"
        );
    }
    // In full: each member's copy carries its own position.
    assert_eq!(
        routed("\n    on read set $p_n = value;\n    on read when value > 2 or value == 0 set $p_n = 0;\n"),
        "\n    on read when target == 1 set north_n = value;\
         \n    on read when target == 1 and (value > 2 or value == 0) set north_n = 0;\n\
         \n    on read when target == 2 set south_n = value;\
         \n    on read when target == 2 and (value > 2 or value == 0) set south_n = 0;\n\n"
    );
}

#[test]
fn the_route_is_placed_in_each_members_copy_after_substitution() {
    // As written, neither effect is complete: `$index` is not yet a number
    // and `$p_readings` not yet a name. The guard's end, and its `or`, are
    // found in the copy.
    assert_eq!(
        routed(
            "\n    on read when value > 2 or value == 0 examine $p_fog cost $index;\
             \n    on read when value > 2 or value == 0 sample $p_readings = value supports $p_wet;\n"
        ),
        "\n    on read when target == 1 and (value > 2 or value == 0) examine north_fog cost 1;\
         \n    on read when target == 1 and (value > 2 or value == 0) sample north_readings = value supports north_wet;\n\
         \n    on read when target == 2 and (value > 2 or value == 0) examine south_fog cost 2;\
         \n    on read when target == 2 and (value > 2 or value == 0) sample south_readings = value supports south_wet;\n\n"
    );
}

#[test]
fn routing_keeps_spacing_line_breaks_comments_and_quoted_text() {
    let body = "
    on read # which plot
        when value > 2 // or a flood
            or value == 0
        set $p_n = value;
    on read // no guard
        reject \"or $p\";
";
    let expanded = routed(body);
    assert_eq!(
        expanded,
        "
    on read # which plot
        when target == 1 and (value > 2 // or a flood
            or value == 0)
        set north_n = value;
    on read when target == 1 // no guard
        reject \"or north\";

    on read # which plot
        when target == 2 and (value > 2 // or a flood
            or value == 0)
        set south_n = value;
    on read when target == 2 // no guard
        reject \"or south\";

"
    );
    // Routing adds no line, so every line keeps its number.
    assert_eq!(expanded.lines().count(), plain(body).lines().count());
}

#[test]
fn a_copy_with_no_complete_effect_gets_its_route_at_the_front_and_does_not_load() {
    let body = "\n    state $p_n = 0;\n    on read when value > 2 or value == 0 soak $p_n;\n";
    assert_eq!(
        routed(body),
        plain(&body.replace("when ", "when target == $index and "))
    );
    assert!(ReactiveSession::from_source(&program(ROUTED, body)).is_err());
}

#[test]
fn an_or_is_the_operator_as_the_expression_reader_reads_it() {
    // One `or` each, however the parentheses touch it.
    for guard in [
        "value > 2 or value == 0",
        "value > 2 or(value == 0)",
        "(value > 2)or value == 0",
        "(value > 2)or(value == 0)",
        "value > 2 # déjà vu\n        or value == 0",
    ] {
        assert_eq!(
            routed(&format!("\n    on read when {guard} set $p_n = value;\n")),
            plain(&format!(
                "\n    on read when target == $index and ({guard}) set $p_n = value;\n"
            )),
            "{guard}"
        );
    }
    // None: inside parentheses (a call's included), quoted text or a
    // comment, inside a longer name, or inside a define read as one operand.
    for guard in [
        "(value > 2 or value == 0)",
        "max(value, value == 0 or value == 1) > 2",
        "$p_label == \"wet or dry\"",
        "value > 2 # or value == 0\n        and value < 9",
        "value > 2 # déjà or\n        and value < 9",
        "value > 2 // or value == 0\n        and value < 9",
        "order > 2",
        "floor(value) > 2 and $p_orbit == 0",
        "$p_either",
    ] {
        let body = format!("\n    define $p_either = value > 2 or value == 0;\n    on read when {guard} set $p_n = value;\n");
        assert_eq!(
            routed(&body),
            plain(&body.replace("when ", "when target == $index and ")),
            "{guard}"
        );
    }
}

#[test]
fn a_guard_that_names_a_parameter_like_an_effect_word_ends_at_the_complete_effect() {
    // `commit` is an effect word and here a parameter too. The guard ends at
    // `set`, the first effect word that begins a complete effect, so the
    // whole guard, `or` and all, goes in the parentheses.
    let source = |guard: &str| {
        format!(
            "{PLOTS}event mark target kind plot, commit min 0 max 9, value min 0 max 9;
{ROUTED} {{
    state $p_hits = 0;
    bind $p.hits = $p_hits;
    on mark when {guard} set $p_hits = $p_hits + 1;
}};
"
        )
    };
    for guard in ["commit == 1 or value == 0", "value == 0 or commit == 1"] {
        let expanded = repeat::expand(&source(guard)).unwrap();
        for (index, plot) in [(1, "north"), (2, "south")] {
            let rule = format!(
                "on mark when target == {index} and ({guard}) set {plot}_hits = {plot}_hits + 1;"
            );
            assert!(expanded.contains(&rule), "{rule}\n{expanded}");
        }
        let mut session = ReactiveSession::from_source(&source(guard)).unwrap();
        session
            .dispatch_json("mark", r#"{"target":"north","commit":0,"value":0}"#)
            .unwrap();
        assert_eq!(
            hits(&session, "north"),
            BindingValue::Number(1.0),
            "{guard}"
        );
        assert_eq!(
            hits(&session, "south"),
            BindingValue::Number(0.0),
            "{guard}"
        );
    }
}

/// A bundle of the module `glow` and the plots with one block, `header {
/// body };`, as its program.
fn glow_bundle(header: &str, body: &str) -> String {
    link::bundle(&[
        link::BundlePart {
            name: "glow".into(),
            source: "module glow;\nstate level = 0;\nevent charge v min 0 max 9;\non charge set level = v;\n"
                .into(),
        },
        link::BundlePart {
            name: "main".into(),
            source: format!("use glow;\n{}", program(header, body)),
        },
    ])
}

#[test]
fn a_qualified_name_is_read_as_linking_writes_it() {
    // The pass runs before `glow::level` is written `glow__level`, and reads
    // the copy as the loader will: the `or` is found in the guard, and the
    // effect is complete.
    for (guard, effect, north, south) in [
        ("value > 5 or glow::level > 5", "$p_hits + 1", 1.0, 0.0),
        ("value > 5 or value == 0", "glow::level", 9.0, 0.0),
    ] {
        let body = |guard: &str| {
            format!(
                "\n    state $p_hits = 0;\n    bind $p.hits = $p_hits;\n    on read when {guard} set $p_hits = {effect};\n"
            )
        };
        let routed = link::link(&glow_bundle(ROUTED, &body(guard))).expect("links");
        let hand = glow_bundle(PLAIN, &body(&format!("target == $index and ({guard})")));
        assert_eq!(routed, link::link(&hand).unwrap(), "{guard}");
        let routed_rule = format!(
            "on read when target == 2 and ({}) set south_hits = {};",
            guard.replace("::", "__"),
            effect.replace("$p", "south").replace("::", "__")
        );
        assert!(routed.contains(&routed_rule), "{routed_rule}\n{routed}");

        let mut session = ReactiveSession::from_source(&glow_bundle(ROUTED, &body(guard))).unwrap();
        session.dispatch_json("glow__charge", r#"{"v":9}"#).unwrap();
        session
            .dispatch_json("read", r#"{"target":"north","value":0}"#)
            .unwrap();
        assert_eq!(
            hits(&session, "north"),
            BindingValue::Number(north),
            "{guard}"
        );
        assert_eq!(
            hits(&session, "south"),
            BindingValue::Number(south),
            "{guard}"
        );
    }
}

/// A block that counts each plot's pings, with `guard` on the count.
fn pings(header: &str, guard: &str) -> ReactiveSession {
    let source = program(
        header,
        &format!(
            "
    state $p_hits = 0;
    define $p_either = loud == 1 or loud == 0;
    bind $p.hits = $p_hits;
    on ping when {guard} set $p_hits = $p_hits + 1;
"
        ),
    );
    ReactiveSession::from_source(&source).unwrap_or_else(|error| panic!("{error}\n{source}"))
}

#[test]
fn a_routed_guard_with_an_or_runs_only_for_the_member_the_event_names() {
    for guard in ["loud == 1 or loud == 0", "$p_either"] {
        let mut session = pings(ROUTED, guard);
        session
            .dispatch_json("ping", r#"{"target":"north","loud":0}"#)
            .unwrap();
        assert_eq!(
            hits(&session, "north"),
            BindingValue::Number(1.0),
            "{guard}"
        );
        assert_eq!(
            hits(&session, "south"),
            BindingValue::Number(0.0),
            "{guard}"
        );
    }
    // Without the parentheses, `and` binds first and `loud == 0` alone runs
    // every plot's copy.
    let mut session = pings(PLAIN, "target == $index and loud == 1 or loud == 0");
    session
        .dispatch_json("ping", r#"{"target":"north","loud":0}"#)
        .unwrap();
    assert_eq!(hits(&session, "south"), BindingValue::Number(1.0));
}

#[test]
fn the_route_comes_first_so_another_members_copy_reads_no_further() {
    // `latest` of a stream with no reading is a fatal error when evaluated.
    let body = |guard: &str| {
        format!(
            "
    claim $p_wet;
    evidence $p_probe from \"a probe in $p\";
    readings $p_readings from $p_probe limit 4;
    state $p_hits = 0;
    bind $p.hits = $p_hits;
    on read when target == $index sample $p_readings = value supports $p_wet;
    on ping when {guard} set $p_hits = $p_hits + 1;
"
        )
    };
    let run = |header: &str, guard: &str| {
        let mut session = ReactiveSession::from_source(&program(header, &body(guard))).unwrap();
        session
            .dispatch_json("read", r#"{"target":"north","value":3}"#)
            .unwrap();
        let result = session.dispatch_json("ping", r#"{"target":"north","loud":0}"#);
        (session, result)
    };

    let (session, result) = run(ROUTED, "latest($p_readings) > 0");
    result.expect("south's copy stops at its route");
    assert_eq!(hits(&session, "north"), BindingValue::Number(1.0));
    assert_eq!(hits(&session, "south"), BindingValue::Number(0.0));

    // Evaluated, the same term is fatal in south's copy: with no route, or
    // with the selection after it.
    for guard in [
        "latest($p_readings) > 0",
        "latest($p_readings) > 0 and target == $index",
    ] {
        let error = run(PLAIN, guard).1.expect_err("south's copy reads latest");
        assert!(error.contains("south_readings"), "{guard}: {error}");
    }
}

// ── Section 2: which rules are routed ──────────────────────────────────────

#[test]
fn every_rule_on_an_event_that_names_the_member_by_p_is_routed() {
    for (written, as_if) in [
        // A reject, and a call. The procedure's step is not a rule.
        (
            "on read when value > 8 reject \"Too wet for $p.\";",
            "on read when target == $index and value > 8 reject \"Too wet for $p.\";",
        ),
        (
            "proc dry_$p() { set $p_n = 0; };\n    on read when value == 0 call dry_$p();",
            "proc dry_$p() { set $p_n = 0; };\n    on read when target == $index and value == 0 call dry_$p();",
        ),
        (
            "proc dry_$p() { set $p_n = 0; };\n    on read call dry_$p();",
            "proc dry_$p() { set $p_n = 0; };\n    on read when target == $index call dry_$p();",
        ),
        // A selection already written is kept, and routed as well.
        (
            "on read when target == $index and value > 2 set $p_n = value;",
            "on read when target == $index and target == $index and value > 2 set $p_n = value;",
        ),
        // Any other comparison with P stays, read with the route.
        (
            "on read when target == target.north set $p_n = value;",
            "on read when target == $index and target == target.north set $p_n = value;",
        ),
        (
            "on read when target != $index set $p_n = value;",
            "on read when target == $index and target != $index set $p_n = value;",
        ),
    ] {
        assert_eq!(
            routed(&format!("\n    {written}\n")),
            plain(&format!("\n    {as_if}\n")),
            "{written}"
        );
    }
}

#[test]
fn a_routed_reject_refuses_only_the_event_about_its_member() {
    let body = "
    state $p_locked = 0 min 0 max 1;
    on ping when loud == 1 set $p_locked = 1;
    on read when $p_locked == 1 reject \"$p is locked.\";
";
    let mut session = ReactiveSession::from_source(&program(ROUTED, body)).unwrap();
    session
        .dispatch_json("ping", r#"{"target":"north","loud":1}"#)
        .unwrap();
    session
        .dispatch_json("read", r#"{"target":"south","value":1}"#)
        .expect("north's reject does not see south's reading");
    let error = session
        .dispatch_json("read", r#"{"target":"north","value":1}"#)
        .expect_err("north's reading is refused");
    assert!(error.contains("north is locked."), "{error}");

    // Unrouted, north's copy refuses south's reading too.
    let mut session = ReactiveSession::from_source(&program(PLAIN, body)).unwrap();
    session
        .dispatch_json("ping", r#"{"target":"north","loud":1}"#)
        .unwrap();
    assert!(session
        .dispatch_json("read", r#"{"target":"south","value":1}"#)
        .is_err());
}

#[test]
fn rules_on_events_that_name_no_member_by_p_expand_as_written() {
    // `tick` names nothing, `swap` names a probe, `nudge_$p` is the member's
    // own event, and the other statements are not rules on an event.
    let body = "
    state $p_n = 0;
    event nudge_$p;
    define $p_wet = $p_n > 2;
    bind $p.n = $p_n;
    cue $p_ring ring $p \"#ffe3a0\" 0.75;
    proc dry_$p() { set $p_n = 0; };
    on tick set $p_n = 0;
    on swap set $p_n = device;
    on nudge_$p set $p_n = 1;
    on read set $p_n = value;
";
    assert_eq!(
        routed(body),
        plain(&body.replace("on read set", "on read when target == $index set"))
    );
}

#[test]
fn a_rule_that_mentions_no_binding_on_an_event_that_names_no_member_expands_as_written() {
    let body = "\n    state $p_n = 0;\n    on tick set total = total + 1;\n    on read set $p_n = value;\n";
    assert_eq!(
        routed(body),
        plain(&body.replace("on read set", "on read when target == $index set"))
    );
    ReactiveSession::from_source(&program(ROUTED, body)).expect("loads");
    // Such a rule on an event the pass cannot read, or on one whose P has
    // another form, is not routed, so its own error comes first.
    assert_eq!(
        refused(
            ROUTED,
            "\n    state $p_n = 0;\n    on landed set total = 1;\n    on read set $p_n = value;\n"
        ),
        "`on landed` in `for plot as $p routed by target`: no event `landed` is declared in this part, so the block cannot tell whether it names a plot"
    );
    let source = format!(
        "{PLOTS}event tally target id;\n{ROUTED} {{ on tally set total = 1; on read set $p_n = value; }};"
    );
    assert_eq!(
        repeat::expand(&source).unwrap_err(),
        "`on tally` in `for plot as $p routed by target`: `tally` declares `target id`, not `target kind plot`; keep the rules on `tally` in a plain for block"
    );
}

/// The refusal of `written`, the last statement of a body with `header`,
/// which has no `;`, at the `line` and `column` where it is written
/// (spec/caveat-repetition-0.1.md section 2).
fn unended(header: &str, (line, column): (usize, usize), written: &str) -> String {
    format!(
        "line {line}, column {column}: the last statement in the body of `{header}`, `{written}`, has no `;` before the `}}` that ends the body; each member's copy of it would run into what follows the copy, so end it with `;`"
    )
}

#[test]
fn a_last_rule_without_its_semicolon_is_refused_before_it_is_routed() {
    // A body's last statement must end with `;`, a routed rule too
    // (spec/caveat-repetition-0.1.md section 2). The block is refused before
    // any rule is routed, plain or routed alike. With two plots, each copy's
    // last rule ran into the next copy, and neither program loaded: "expected
    // '=' at byte 21".
    let body = "\n    state $p_n = 0;\n    on ping set $p_n = 1;\n    on read set $p_m = value\n";
    assert_eq!(
        refused(ROUTED, body),
        unended(ROUTED, (13, 5), "on read set $p_m = value")
    );
    let by_hand = body.replace(" set $p_", " when target == $index set $p_");
    assert_eq!(
        refused(PLAIN, &by_hand),
        unended(
            PLAIN,
            (13, 5),
            "on read when target == $index set $p_m = value"
        )
    );
    assert_eq!(
        refused(ROUTED, " state $p_n = 0; on read set $p_n = value\n"),
        unended(ROUTED, (10, 51), "on read set $p_n = value")
    );
    // Before routing's own checks too: this rule mentions neither `$p` nor
    // `$index`.
    assert_eq!(
        refused(
            ROUTED,
            "\n    state $p_n = 0;\n    on ping set $p_n = 1;\n    on read set total = value\n"
        ),
        unended(ROUTED, (13, 5), "on read set total = value")
    );
    // With `;`, each is routed like any other rule.
    let ended = body.replace("value\n", "value;\n");
    assert_eq!(
        routed(&ended),
        plain(&by_hand.replace("value\n", "value;\n"))
    );
    // With one member, the copy was a last statement that loaded, as the
    // hand-routed one does. It is refused too, and loads with its `;`.
    let source = |header: &str, rule: &str| {
        format!("{PLOTS}{header} {{ state $d_n = 0; bind $d.n = $d_n; {rule} }};\n")
    };
    let header = "for probe as $d routed by device";
    assert_eq!(
        repeat::expand(&source(header, "on swap set $d_n = 1")).unwrap_err(),
        unended(header, (10, 70), "on swap set $d_n = 1")
    );
    let routed = source(header, "on swap set $d_n = 1;");
    assert_eq!(
        repeat::expand(&routed).unwrap(),
        repeat::expand(&source(
            "for probe as $d",
            "on swap when device == $index set $d_n = 1;"
        ))
        .unwrap()
    );
    let mut session = ReactiveSession::from_source(&routed).expect("loads");
    session
        .dispatch_json("swap", r#"{"device":"handheld"}"#)
        .unwrap();
    assert_eq!(
        session.snapshot().bindings["handheld"]["n"],
        BindingValue::Number(1.0)
    );
}

#[test]
fn an_event_a_for_block_declares_is_read_once_expanded() {
    // In a plain block, and in a routed one.
    for (header, rules) in [
        ("for probe as $d", ""),
        ("for probe as $d routed by device", " on swap set $d_x = 1;"),
    ] {
        let source = format!(
            "{PLOTS}{header} {{
    state $d_x = 0;
    event poke_$d target kind plot;{rules}
}};
{ROUTED} {{
    state $p_n = 0;
    on poke_handheld set $p_n = 1;
}};
"
        );
        let expanded = repeat::expand(&source).unwrap();
        assert!(
            expanded.contains("on poke_handheld when target == 1 set north_n = 1;")
                && expanded.contains("on poke_handheld when target == 2 set south_n = 1;"),
            "{expanded}"
        );
        ReactiveSession::from_source(&source).expect("loads");
    }
}

#[test]
fn an_event_declared_in_a_block_that_does_not_expand_is_not_read() {
    // The block's `event` statement expands, and another statement does not,
    // so the block does not: the routed block before it cannot read the
    // event, and loading stops there.
    let unread = "`on poke_handheld` in `for plot as $p routed by target`: no event `poke_handheld` is declared in this part, so the block cannot tell whether it names a plot";
    for block in [
        "for probe as $d { event poke_$d target kind plot; claim $typo; };",
        "for probe as $d { event poke_$d target kind plot; for plot as $q { claim $q; }; };",
        "for probe is $d { event poke_$d target kind plot; };",
    ] {
        let source = format!(
            "{PLOTS}{ROUTED} {{ state $p_n = 0; on poke_handheld set $p_n = 1; }};\n{block}\n"
        );
        assert_eq!(repeat::expand(&source).unwrap_err(), unread, "{block}");
    }
}

#[test]
fn a_last_event_declaration_without_its_semicolon_is_read() {
    // The parser reads it, so the part declares the event.
    let source = |header: &str, rule: &str| {
        format!(
            "place field kind field;
entity north kind plot at field;
entity south kind plot at field;
{header} {{
    state $p_hits = 0;
    {rule}
}};
event read target kind plot"
        )
    };
    let routed = source(ROUTED, "on read set $p_hits = 1;");
    assert_eq!(
        repeat::expand(&routed).unwrap(),
        repeat::expand(&source(
            PLAIN,
            "on read when target == $index set $p_hits = 1;"
        ))
        .unwrap()
    );
    ReactiveSession::from_source(&routed).expect("loads");
    // So is a `for` block's last `event` statement, once expanded, ended
    // with its `;`.
    let source = |end: &str| {
        format!(
            "{PLOTS}{ROUTED} {{ state $p_n = 0; on poke_handheld set $p_n = 1; }};
for probe as $d {{ event poke_$d target kind plot{end} }};
"
        )
    };
    assert!(repeat::expand(&source(";"))
        .unwrap()
        .contains("on poke_handheld when target == 2 set south_n = 1;"));
    ReactiveSession::from_source(&source(";")).expect("loads");
    // Without its `;`, that block is refused. The routed block before it
    // still reads the event its copy declares, so the refusal is the one that
    // says what is wrong, not that no `poke_handheld` is declared.
    assert_eq!(
        repeat::expand(&source("")).unwrap_err(),
        unended(
            "for probe as $d",
            (11, 19),
            "event poke_$d target kind plot"
        )
    );
}

// ── Section 7: errors, and their order ─────────────────────────────────────

#[test]
fn a_malformed_routed_header_is_refused() {
    for header in [
        "for plot as $p routed target",
        "for plot as $p routed by",
        "for plot as $p routed by $target",
        "for plot as $p routed by target.north",
        "for plot as $p routed by target extra",
        "for plot   as $p routed\n    by 1st",
        "for plot is $p routed by target",
        // A binding that is not `$NAME` is not the routed header either.
        "for plot as p routed by target",
        "for plot as $1p routed by target",
        "for plot as $p.x routed by target",
    ] {
        let normalized = header.split_whitespace().collect::<Vec<_>>().join(" ");
        assert_eq!(
            refused(header, "\n    on read set $p_n = value;\n"),
            format!("expected `for KIND as $NAME routed by PARAM {{ ... }}`, found: {normalized}"),
            "{header}"
        );
    }
    // `$index` is a `$NAME`, which Repetition 0.1 refuses to rebind.
    assert_eq!(
        refused(
            "for plot as $index routed by target",
            "\n    on read set total = $index;\n"
        ),
        "$index is provided by the block and cannot be rebound"
    );
    // A fifth word other than `routed` is Repetition 0.1's malformed header.
    assert!(
        refused("for plot as $p routes by target", "on read set $p_n = 1;")
            .starts_with("expected `for KIND as $NAME { ... }`, found: ")
    );
}

#[test]
fn a_block_that_routes_no_rule_is_refused() {
    let expected = "`for plot as $p routed by target` routes no rule: no rule in it is on an event that declares `target kind plot`";
    assert_eq!(
        refused(
            ROUTED,
            "\n    state $p_n = 0;\n    on tick set $p_n = 0;\n    on swap set $p_n = 1;\n"
        ),
        expected
    );
    // A rule on the member's own event is not routed.
    assert_eq!(
        refused(
            ROUTED,
            "\n    state $p_n = 0;\n    event nudge_$p;\n    on nudge_$p set $p_n = 1;\n"
        ),
        expected
    );
    // HEADER is written with each run of whitespace as one space.
    assert_eq!(
        refused(
            "for  plot\n    as $p   routed by\ttarget",
            "\n    on tick set $p_n = 0;\n"
        ),
        expected
    );
}

#[test]
fn p_declared_in_another_form_is_refused() {
    for (declaration, declared) in [
        ("event tally target min 0 max 3;", "target min 0 max 3"),
        (
            "event tally target in north south;",
            "target in north south",
        ),
        ("event tally target id;", "target id"),
        ("event tally target kind probe;", "target kind probe"),
    ] {
        let source = format!("{PLOTS}{declaration}\n{ROUTED} {{ on tally set $p_n = 1; }};");
        assert_eq!(
            repeat::expand(&source).unwrap_err(),
            format!("`on tally` in `for plot as $p routed by target`: `tally` declares `{declared}`, not `target kind plot`; keep the rules on `tally` in a plain for block")
        );
    }
    // A parameter of the kind besides P does not make it error 4.
    let source = format!(
        "{PLOTS}event tally target id, base kind plot;\n{ROUTED} {{ on tally set $p_n = 1; }};"
    );
    assert_eq!(
        repeat::expand(&source).unwrap_err(),
        "`on tally` in `for plot as $p routed by target`: `tally` declares `target id`, not `target kind plot`; keep the rules on `tally` in a plain for block"
    );
    // The example in section 7.
    let (_, converted) = ledger();
    let broken = converted.replace(
        "event approved target kind pr, commit id;",
        "event approved target id;",
    );
    assert_eq!(
        repeat::expand(&broken).unwrap_err(),
        "`on approved` in `for pr as $p routed by target`: `approved` declares `target id`, not `target kind pr`; keep the rules on `approved` in a plain for block"
    );
}

#[test]
fn a_member_named_by_another_parameter_is_refused() {
    let source = format!(
        "{PLOTS}event probe_read sensor kind plot, value min 0 max 9;\n{ROUTED} {{ on probe_read set $p_n = value; }};"
    );
    assert_eq!(
        repeat::expand(&source).unwrap_err(),
        "`on probe_read` in `for plot as $p routed by target`: `probe_read` names a plot by `sensor`, not by `target`; route the block by `sensor`, or keep the rules on `probe_read` in a plain for block"
    );
    let source = format!(
        "{PLOTS}event stacked base kind plot, level min 0 max 3, head kind plot;\n{ROUTED} {{ on stacked set $p_n = level; }};"
    );
    assert_eq!(
        repeat::expand(&source).unwrap_err(),
        "`on stacked` in `for plot as $p routed by target`: `stacked` names a plot by `base`, `head`, not by `target`; route the block by one of them, or keep the rules on `stacked` in a plain for block"
    );
    // In the order the event declares them.
    let source = format!(
        "{PLOTS}event stacked head kind plot, base kind plot;\n{ROUTED} {{ on stacked set $p_n = 1; }};"
    );
    assert_eq!(
        repeat::expand(&source).unwrap_err(),
        "`on stacked` in `for plot as $p routed by target`: `stacked` names a plot by `head`, `base`, not by `target`; route the block by one of them, or keep the rules on `stacked` in a plain for block"
    );
}

#[test]
fn a_mistyped_p_on_the_ledger_is_refused_at_its_first_rule() {
    let (_, converted) = ledger();
    let mistyped = converted.replace("routed by target {", "routed by targt {");
    assert_eq!(
        repeat::expand(&mistyped).unwrap_err(),
        "`on pushed` in `for pr as $p routed by targt`: `pushed` names a pr by `target`, not by `targt`; route the block by `target`, or keep the rules on `pushed` in a plain for block"
    );
}

#[test]
fn an_event_not_declared_in_the_part_is_refused() {
    assert_eq!(
        refused(ROUTED, "\n    on landed set $p_n = 1;\n"),
        "`on landed` in `for plot as $p routed by target`: no event `landed` is declared in this part, so the block cannot tell whether it names a plot"
    );
    // A module's event is not read from the program's part.
    let text = link::bundle(&[
        link::BundlePart {
            name: "sensors".into(),
            source: "module sensors;\nevent landed target kind plot;\n".into(),
        },
        link::BundlePart {
            name: "main".into(),
            source: format!(
                "use sensors;\n{PLOTS}{ROUTED} {{ state $p_n = 0; on landed set $p_n = 1; }};\n"
            ),
        },
    ]);
    assert_eq!(
        link::link(&text).unwrap_err(),
        "main: `on landed` in `for plot as $p routed by target`: no event `landed` is declared in this part, so the block cannot tell whether it names a plot"
    );
}

#[test]
fn a_routed_rule_that_mentions_no_binding_is_refused() {
    let expected = "`on read` in `for plot as $p routed by target` mentions neither `$p` nor `$index`; its copies are all the same rule, so write it once, outside the block";
    assert_eq!(
        refused(ROUTED, "\n    on read set total = total + 1;\n"),
        expected
    );
    // A comment is not a mention.
    assert_eq!(
        refused(ROUTED, "\n    on read # for $p\n        set total = 1;\n"),
        expected
    );
    // Quoted text is.
    assert!(expanded(
        ROUTED,
        "\n    on read when value > 8 reject \"Too wet for $p.\";\n"
    )
    .is_ok());
    assert!(expanded(ROUTED, "\n    on read when value > $index set total = 1;\n").is_ok());
}

#[test]
fn an_entity_of_the_kind_in_a_for_block_is_refused() {
    let source = format!(
        "{PLOTS}for probe as $d {{ entity $d_plot kind plot at field; }};\n{ROUTED} {{ on read set $p_n = value; }};\n"
    );
    assert_eq!(
        repeat::expand(&source).unwrap_err(),
        "`for plot as $p routed by target`: `$index` does not count entity `$d_plot` of kind plot, declared in a for block, but `target` does; declare it at the top level of this part"
    );
    // An entity of another kind in a for block is not counted by `target`.
    let source = format!(
        "{PLOTS}for probe as $d {{ entity $d_tag kind tag at field; }};\n{ROUTED} {{ state $p_n = 0; on read set $p_n = value; }};\n"
    );
    assert!(repeat::expand(&source)
        .unwrap()
        .contains("on read when target == 2 set south_n = value;"));
    ReactiveSession::from_source(&source).expect("loads");
}

/// Plots north and south, with `north` written as given, and a routed block.
fn with_north(north: &str, after: &str) -> String {
    format!(
        "place field kind field;
{north}
entity south kind plot at field;
event read target kind plot, value min 0 max 9;
{ROUTED} {{
    state $p_hits = 0;
    on read set $p_hits = $p_hits + 1;
}};
{after}"
    )
}

/// Each plot's hits after one `read` of `target` on a fresh session, in the
/// order `target` numbers the plots.
fn hits_after_read(source: &str, target: &str) -> Vec<(String, f64)> {
    let mut session =
        ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("{error}\n{source}"));
    let snapshot = session
        .dispatch_json("read", &format!(r#"{{"target":"{target}","value":1}}"#))
        .expect("read is accepted");
    let read = snapshot
        .events
        .iter()
        .find(|event| event.name == "read")
        .expect("read is declared");
    let ParameterDomain::Entity { members, .. } = &read.parameters[0].domain else {
        panic!("target names a plot");
    };
    members
        .iter()
        .map(|plot| (plot.clone(), snapshot.values[&format!("{plot}_hits")]))
        .collect()
}

#[test]
fn a_top_level_entity_with_a_comment_in_it_or_without_its_semicolon_is_routed_to() {
    // Repetition 0.1 reads an entity statement as the loader does (#47), so
    // `$index` counts the plots `target` counts, and a read runs only the
    // copy of the plot it names.
    for (north, after, plots) in [
        (
            "entity north kind plot # the first plot\n    at field;",
            "",
            &["north", "south"][..],
        ),
        (
            "entity north kind plot // the first plot\n    at field;",
            "",
            &["north", "south"],
        ),
        (
            "entity north kind plot at field;",
            "entity east kind plot at field",
            &["north", "south", "east"],
        ),
    ] {
        let source = with_north(north, after);
        for (position, target) in plots.iter().enumerate() {
            let expected = plots
                .iter()
                .map(|plot| (plot.to_string(), f64::from(u8::from(plot == target))))
                .collect::<Vec<_>>();
            assert_eq!(hits_after_read(&source, target), expected, "{source}");
            let copy = format!(
                "on read when target == {} set {target}_hits = {target}_hits + 1;",
                position + 1
            );
            assert!(repeat::expand(&source).unwrap().contains(&copy), "{copy}");
        }
    }
    // A comment elsewhere, a statement over several lines and an entity of
    // another kind are read as before.
    let source = with_north(
        "# the first plot\nentity north\n    kind plot at field; # north\nentity tag kind tag # a tag\n    at field;",
        "",
    );
    assert!(repeat::expand(&source)
        .unwrap()
        .contains("on read when target == 2 set south_hits = south_hits + 1;"));
    assert_eq!(
        hits_after_read(&source, "south"),
        [("north".to_string(), 0.0), ("south".to_string(), 1.0)]
    );
    // The routed block expands as the plain block with the route written by
    // hand. Before #47, Repetition 0.1 gave that plain block one copy, south's,
    // selected by north's position.
    let commented = with_north("entity north kind plot # the first plot\n    at field;", "");
    let by_hand = commented
        .replace(ROUTED, PLAIN)
        .replace("on read set", "on read when target == $index set");
    assert_eq!(repeat::expand(&commented), repeat::expand(&by_hand));
    let before = repetition_0_1::expand(&by_hand).unwrap();
    assert!(before.contains("on read when target == 1 set south_hits = south_hits + 1;"));
    assert!(!before.contains("north_hits"), "{before}");
}

#[test]
fn errors_come_in_the_order_section_7_gives() {
    let entity = "for probe as $d { entity $d_plot kind plot at field; };\n";
    let first = |source: String| repeat::expand(&source).unwrap_err();
    // The header, before an entity of the kind in a for block.
    assert!(first(format!(
        "{PLOTS}{entity}for plot as $p routed target {{ on landed set total = 1; }};"
    ))
    .starts_with("expected `for KIND as $NAME routed by PARAM"));
    // The entity, before any rule.
    assert!(first(format!(
        "{PLOTS}{entity}{ROUTED} {{ on landed set total = 1; }};"
    ))
    .contains("does not count entity `$d_plot`"));
    // An entity in a for block, then the rules. A top-level entity with a
    // comment in its statement is counted, so it is no error.
    let commented = "entity north kind plot # the first plot\n    at field;";
    assert!(first(with_north(commented, entity)).contains("declared in a for block"));
    assert!(
        first(with_north(commented, "").replace("on read", "on landed"))
            .contains("no event `landed` is declared")
    );
    // The rules in the order written, then whether any rule is routed.
    let rules = |body: &str| first(format!("{PLOTS}{ROUTED} {{ {body} }};"));
    assert!(rules("on read set total = 1; on landed set $p_n = 1;")
        .contains("mentions neither `$p` nor `$index`"));
    assert!(rules("on landed set $p_n = 1; on read set total = 1;")
        .contains("no event `landed` is declared"));
    assert!(rules("on tick set $p_n = 0; on landed set $p_n = 1;")
        .contains("no event `landed` is declared"));
    // Blocks in source order.
    assert!(first(format!(
        "{PLOTS}{ROUTED} {{ on tick set $p_n = 0; }};\n{ROUTED} {{ on landed set $p_n = 1; }};"
    ))
    .contains("routes no rule"));
}

#[test]
fn a_routed_block_in_a_module_expands_in_its_own_part() {
    let module = format!(
        "module plots;\n{PLOTS}{ROUTED} {{ state $p_n = 0; on read set $p_n = value; }};\n"
    );
    let text = link::bundle(&[
        link::BundlePart {
            name: "plots".into(),
            source: module.clone(),
        },
        link::BundlePart {
            name: "main".into(),
            source: "use plots;\nbudget 1;\n".into(),
        },
    ]);
    let linked = link::link(&text).expect("links");
    assert!(
        linked.contains("on plots__read when target == 1 set plots__north_n = value;"),
        "{linked}"
    );
    // An error names the part, as Repetition 0.1's do.
    let text = link::bundle(&[
        link::BundlePart {
            name: "plots".into(),
            source: module.replace("on read", "on tick"),
        },
        link::BundlePart {
            name: "main".into(),
            source: "use plots;\nbudget 1;\n".into(),
        },
    ]);
    assert_eq!(
        link::link(&text).unwrap_err(),
        "plots: `for plot as $p routed by target` routes no rule: no rule in it is on an event that declares `target kind plot`"
    );
}

#[test]
fn entities_of_the_kind_in_another_part_meet_the_same_limit_as_a_hand_written_route() {
    // Section 9: the pass reads only its own part, so it cannot see a
    // module's plots, which `target` counts and `$index` does not. The
    // routed block expands exactly as the hand-routed one does.
    let bundle = |header: &str, guard: &str| {
        link::bundle(&[
            link::BundlePart {
                name: "annex".into(),
                source: "module annex;\nplace shed kind shed;\nentity east kind plot at shed;\n"
                    .into(),
            },
            link::BundlePart {
                name: "main".into(),
                source: program(
                    header,
                    &format!("\n    state $p_n = 0;\n    on read {guard}set $p_n = value;\n"),
                )
                .replacen("place field", "use annex;\nplace field", 1),
            },
        ])
    };
    let routed = link::link(&bundle(ROUTED, "")).expect("links");
    assert_eq!(
        routed,
        link::link(&bundle(PLAIN, "when target == $index ")).unwrap()
    );
    assert!(
        routed.contains("on read when target == 1 set north_n = value;"),
        "{routed}"
    );
}

// ── Section 1: a plain block is unchanged ──────────────────────────────────

/// Repetition 0.1's expansion as it was at 386c5cf, before routing, copied
/// from runtime/src/repeat.rs and the two helpers it used from link.rs. It
/// reads entity statements as Repetition 0.1 did before #47, with their
/// comments left in, and does not read a last one without its `;`. No
/// program it is compared with below has either.
mod repetition_0_1 {
    fn is_identifier_start(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    fn is_identifier_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }

    fn statement_spans(source: &str) -> Vec<(usize, usize)> {
        let mut spans = Vec::new();
        let mut start = None;
        let mut braces = 0usize;
        let mut chars = source.char_indices().peekable();
        while let Some((index, ch)) = chars.next() {
            if ch == '"' {
                if start.is_none() {
                    start = Some(index);
                }
                let mut escaped = false;
                for (_, ch) in chars.by_ref() {
                    if escaped {
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == '"' {
                        break;
                    }
                }
                continue;
            }
            if ch == '#' || (ch == '/' && chars.peek().map(|(_, next)| *next) == Some('/')) {
                for (_, ch) in chars.by_ref() {
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }
            if ch == '{' {
                braces += 1;
            } else if ch == '}' {
                braces = braces.saturating_sub(1);
            }
            if ch == ';' && braces == 0 {
                if let Some(begin) = start.take() {
                    spans.push((begin, index + 1));
                }
                continue;
            }
            if start.is_none() && !ch.is_whitespace() {
                start = Some(index);
            }
        }
        spans
    }

    fn statement_words(text: &str) -> Vec<&str> {
        text.trim_end_matches(';').split_whitespace().collect()
    }

    pub fn expand(source: &str) -> Result<String, String> {
        let spans = statement_spans(source);
        let blocks: Vec<(usize, usize)> = spans
            .iter()
            .copied()
            .filter(|(start, end)| statement_words(&source[*start..*end]).first() == Some(&"for"))
            .collect();
        if blocks.is_empty() {
            return Ok(source.to_string());
        }
        let members = entity_kinds(source, &spans);

        let mut out = String::with_capacity(source.len());
        let mut cursor = 0;
        for (start, end) in blocks {
            out.push_str(&source[cursor..start]);
            out.push_str(&expand_block(&source[start..end], &members)?);
            cursor = end;
        }
        out.push_str(&source[cursor..]);
        Ok(out)
    }

    fn entity_kinds(source: &str, spans: &[(usize, usize)]) -> Vec<(String, Vec<String>)> {
        let mut kinds: Vec<(String, Vec<String>)> = Vec::new();
        for (start, end) in spans {
            if let ["entity", name, "kind", kind, "at", _] =
                statement_words(&source[*start..*end]).as_slice()
            {
                match kinds.iter_mut().find(|(declared, _)| declared == kind) {
                    Some((_, members)) => members.push((*name).to_string()),
                    None => kinds.push(((*kind).to_string(), vec![(*name).to_string()])),
                }
            }
        }
        kinds
    }

    fn expand_block(statement: &str, kinds: &[(String, Vec<String>)]) -> Result<String, String> {
        let open = statement
            .find('{')
            .ok_or_else(|| format!("for block has no body: {}", head(statement)))?;
        let close = statement
            .rfind('}')
            .ok_or_else(|| format!("for block is not closed: {}", head(statement)))?;
        if close < open {
            return Err(format!("for block is not closed: {}", head(statement)));
        }
        let header = statement_words(&statement[..open]);
        let ["for", kind, "as", binding] = header.as_slice() else {
            return Err(format!(
                "expected `for KIND as $NAME {{ ... }}`, found: {}",
                head(statement)
            ));
        };
        let Some(name) = binding.strip_prefix('$') else {
            return Err(format!("{binding} is not a binding; write $name"));
        };
        check_binding(name)?;
        if name == "index" {
            return Err("$index is provided by the block and cannot be rebound".into());
        }

        let body = &statement[open + 1..close];
        if statement_spans(body)
            .into_iter()
            .any(|(start, end)| statement_words(&body[start..end]).first() == Some(&"for"))
        {
            return Err(format!(
                "for blocks do not nest; see spec/caveat-repetition-0.1.md section 4: {}",
                head(statement)
            ));
        }
        let Some((_, members)) = kinds.iter().find(|(declared, _)| declared == kind) else {
            return Err(format!(
                "no entity is declared `kind {kind}`, so `for {kind}` has nothing to expand"
            ));
        };

        let mut out = String::new();
        for (position, member) in members.iter().enumerate() {
            let bindings = [
                (name, member.as_str()),
                ("index", &(position + 1).to_string()),
            ];
            out.push_str(&substitute(body, &bindings)?);
            if !out.ends_with('\n') {
                out.push('\n');
            }
        }
        Ok(out)
    }

    fn substitute(body: &str, bindings: &[(&str, &str); 2]) -> Result<String, String> {
        let mut out = String::with_capacity(body.len());
        let mut chars = body.char_indices().peekable();
        while let Some((index, ch)) = chars.next() {
            if ch != '$' {
                out.push(ch);
                continue;
            }
            let start = index + ch.len_utf8();
            let mut end = start;
            while let Some(next) = body[end..].chars().next() {
                let acceptable = if end == start {
                    is_identifier_start(next)
                } else {
                    is_identifier_char(next)
                };
                if !acceptable {
                    break;
                }
                end += next.len_utf8();
            }
            let word = &body[start..end];
            let matched = bindings
                .iter()
                .filter(|(name, _)| word.starts_with(*name))
                .max_by_key(|(name, _)| name.len());
            let Some((name, value)) = matched else {
                return Err(format!(
                    "${word} is not bound in this for block; $ is not a literal"
                ));
            };
            out.push_str(value);
            out.push_str(&word[name.len()..]);
            while chars.peek().map(|(next, _)| *next < end) == Some(true) {
                chars.next();
            }
        }
        Ok(out)
    }

    fn check_binding(name: &str) -> Result<(), String> {
        let mut chars = name.chars();
        let valid = chars.next().is_some_and(is_identifier_start) && chars.all(is_identifier_char);
        if valid {
            Ok(())
        } else {
            Err(format!("${name} is not a usable binding name"))
        }
    }

    fn head(statement: &str) -> String {
        statement
            .split_whitespace()
            .take(5)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[test]
fn every_program_with_a_for_block_expands_as_before() {
    // The editor's highlighting fixture has one routed block among its plain
    // ones. Without the clause it expands as before, and with it, as if the
    // route were written by hand. One of its rules names a second reef by
    // `$near` (section 10), and by hand it names the fixture's one reef.
    let fixture = read("../editors/vscode/test/fixtures/scopes.cav");
    assert_eq!(fixture.matches(" routed by target {").count(), 1);
    let near = "on dive when $near_seen == 1 set $r_seen = 3;";
    assert_eq!(fixture.matches(near).count(), 1);
    let by_hand = fixture.replace(
        near,
        "on dive when near == 1 and reef_one_seen == 1 set $r_seen = 3;",
    );
    let unrouted = by_hand.replace(" routed by target {", " {");
    assert_eq!(repeat::expand(&unrouted), repetition_0_1::expand(&unrouted));
    assert_eq!(
        repeat::expand(&fixture).unwrap(),
        repeat::expand(&unrouted.replace("on dive when ", "on dive when target == $index and "))
            .unwrap()
    );
    // Every other tracked .cav file with a `for` block, all of them plain.
    for file in [
        "../experiments/agent-ledger/ledger-approved-head.cav",
        "../experiments/agent-ledger/ledger-identifiers.cav",
        "../experiments/agent-ledger/ledger.cav",
        "../experiments/glowcap/caveat/glowcap.cav",
        "../experiments/glowcap/caveat2/glowcap.cav",
        "../experiments/glowcap/caveat3/glowcap.cav",
        "../experiments/glowcap/caveat4/glowcap.cav",
        "../experiments/glowcap/caveat5/glowcap.cav",
        "../game/glowcap.cav",
        "../game/trail_rescue.cav",
    ] {
        let source = read(file);
        assert!(source.contains("for "), "{file} has a for block");
        assert_eq!(
            repeat::expand(&source),
            repetition_0_1::expand(&source),
            "{file}"
        );
    }
    // And the shapes Repetition 0.1's own tests use, errors included.
    let entities = "entity reef_one kind reef at harbor;\nentity reef_two kind reef at harbor;\n";
    for block in [
        "for reef as $r { evidence $r_reading from \"$r\"; on tick when contact == $index set hit_x = $r.x; };",
        "for reef as $r {\n    # a note on routed by target\n    on read set $r_n = target;\n};",
        "for reef as $r { on read when target == $index or x > 1 set $r_n = 1; };",
        "for buoy as $b { claim $b_seen; };",
        "for reef as $r { claim $typo_seen; };",
        "for reef as r { claim r_seen; };",
        "for reef as $index { claim $index; };",
        "for reef as $r { for reef as $q { claim $q; }; };",
        "for reef { claim x; };",
        "for reef as $r routes by target { claim $r_seen; };",
        // A word the binding begins is the binding's, though an event
        // parameter begins it too (section 10).
        "event show target kind reef, shown kind reef;\nfor reef as $s { state $shown_n = 0; on show when target == $index set $shown_n = 1; };",
    ] {
        let source = format!("{entities}{block}\n");
        assert_eq!(
            repeat::expand(&source),
            repetition_0_1::expand(&source),
            "{block}"
        );
    }
}

// ── Section 5: the agent ledger ────────────────────────────────────────────

/// Today's ledger, and the same ledger converted as section 5 says: `routed
/// by target` added to the header, `target == $index and ` deleted from 20
/// rules and ` when target == $index` from the other 11.
fn ledger() -> (String, String) {
    let today = read("../experiments/agent-ledger/ledger-identifiers.cav");
    assert_eq!(today.matches("target == $index and ").count(), 20);
    let converted = today.replace("target == $index and ", "");
    assert_eq!(converted.matches(" when target == $index").count(), 11);
    let converted = converted.replace(" when target == $index", "");
    assert_eq!(converted.matches("for pr as $p {").count(), 1);
    let converted = converted.replace("for pr as $p {", "for pr as $p routed by target {");
    assert!(!converted.contains("target == $index"));
    (today, converted)
}

#[test]
fn the_converted_ledger_expands_to_the_hand_routed_ledger_byte_for_byte() {
    let (today, converted) = ledger();
    let expected = repeat::expand(&today).unwrap();
    assert_eq!(repeat::expand(&converted).unwrap(), expected);
    // 31 rules for each of the four pull requests.
    assert_eq!(expected.matches("when target == ").count(), 124);
}

/// A dispatch outcome as JSON, with the snapshot's `source_id` taken out:
/// the two ledgers are different sources.
fn outcome(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let mut value = match session.dispatch_outcome_json(event, payload) {
        Ok(outcome) => serde_json::to_value(outcome),
        Err(fatal) => serde_json::to_value(fatal),
    }
    .unwrap();
    if let Some(snapshot) = value.get_mut("snapshot").and_then(Value::as_object_mut) {
        assert!(snapshot.remove("source_id").is_some());
    }
    value
}

#[test]
fn the_converted_ledger_runs_the_ledgers_scenarios_as_the_hand_routed_one() {
    let (today, converted) = ledger();
    let file: Value = serde_json::from_str(&read(
        "../experiments/agent-ledger/ledger-identifiers.scenarios.json",
    ))
    .unwrap();
    let scenarios = file["scenarios"].as_array().unwrap();
    assert_eq!(scenarios.len(), 11);
    let (mut accepted, mut rejected) = (0, 0);
    for scenario in scenarios {
        let id = &scenario["id"];
        let mut hand = ReactiveSession::from_source(&today).unwrap();
        let mut routed = ReactiveSession::from_source(&converted).unwrap();
        for step in scenario["steps"].as_array().unwrap() {
            if step.get("resume").is_some() {
                hand = ReactiveSession::restore_json(&today, &hand.save_json().unwrap()).unwrap();
                routed = ReactiveSession::restore_json(&converted, &routed.save_json().unwrap())
                    .unwrap();
            }
            let Some(event) = step["send"].as_str() else {
                continue;
            };
            let payload = step.get("payload").map_or("{}".into(), Value::to_string);
            for _ in 0..step["repeat"].as_u64().unwrap_or(1) {
                let left = outcome(&mut hand, event, &payload);
                assert_eq!(left, outcome(&mut routed, event, &payload), "{id}: {event}");
                // As the scenario expects: refused exactly when it says so.
                assert_eq!(
                    left["outcome"] == "rejected",
                    step.get("rejected").is_some(),
                    "{id}: {event} {left}"
                );
                if left["outcome"] == "rejected" {
                    rejected += 1;
                } else {
                    accepted += 1;
                }
            }
        }
        let (mut left, mut right) = (hand.snapshot(), routed.snapshot());
        assert_ne!(left.source_id, right.source_id);
        left.source_id.clear();
        right.source_id.clear();
        assert_eq!(left, right, "{id}");
    }
    assert!(
        accepted > 50 && rejected > 10,
        "{accepted} accepted, {rejected} rejected"
    );
}

#[test]
fn a_save_from_the_hand_routed_ledger_does_not_restore_into_the_routed_one() {
    let (today, converted) = ledger();
    let mut hand = ReactiveSession::from_source(&today).unwrap();
    let mut routed = ReactiveSession::from_source(&converted).unwrap();
    let head = "602bdbec0a047a5319f53e83f336b9f7aec0e5ed";
    for (event, payload) in [
        (
            "pushed",
            format!(r#"{{"target":"pr26","commit":"{head}"}}"#),
        ),
        (
            "checks",
            format!(r#"{{"target":"pr26","commit":"{head}","result":"passed"}}"#),
        ),
        (
            "approved",
            format!(r#"{{"target":"pr26","commit":"{head}"}}"#),
        ),
        ("merge", r#"{"target":"pr26"}"#.to_string()),
    ] {
        hand.dispatch_json(event, &payload).unwrap();
        routed.dispatch_json(event, &payload).unwrap();
    }
    // The same state, apart from which program holds it.
    let (mut left, mut right) = (hand.snapshot(), routed.snapshot());
    assert_ne!(left.source_id, right.source_id);
    left.source_id.clear();
    right.source_id.clear();
    assert_eq!(left, right);

    let save = hand.save_json().unwrap();
    match ReactiveSession::restore_json(&converted, &save) {
        Ok(_) => panic!("a save restored into another program"),
        Err(error) => assert_eq!(
            error,
            "cannot restore save: it belongs to a different program"
        ),
    }
    let restored =
        ReactiveSession::restore_json(&today, &save).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(restored.snapshot(), hand.snapshot());
}

// ── Section 3: the route counts toward the expression limits ───────────────

#[test]
fn a_guard_at_the_nesting_limit_as_written_fails_to_load_once_routed() {
    // A chain of `and` nests one level per operand. `$p_n >= 0` and 62 more
    // operands is 64 levels: the limit.
    let chain = |operands: usize| {
        std::iter::once("$p_n >= 0")
            .chain(std::iter::repeat_n("true", operands - 1))
            .collect::<Vec<_>>()
            .join(" and ")
    };
    let load = |header: &str, guard: String| {
        ReactiveSession::from_source(&program(
            header,
            &format!("\n    state $p_n = 0;\n    on read when {guard} set $p_n = value;\n"),
        ))
        .map(|_| ())
    };
    let too_deep = "expression exceeds nesting limit 64";

    // At the limit as written, the guard loads, and routed it does not.
    load(PLAIN, chain(63)).expect("63 operands load");
    assert!(load(ROUTED, chain(63)).unwrap_err().contains(too_deep));
    // The same holds for a guard in parentheses.
    let either = format!("{} or true", chain(62));
    load(PLAIN, either.clone()).expect("an or at the limit loads");
    assert!(load(ROUTED, either).unwrap_err().contains(too_deep));

    // A hand-routed guard at the limit loads, and so does its conversion.
    // With the selection left in place, the route is one level more.
    load(PLAIN, format!("target == $index and {}", chain(62))).expect("hand-routed loads");
    load(ROUTED, chain(62)).expect("converted loads");
    assert!(load(ROUTED, format!("target == $index and {}", chain(62)))
        .unwrap_err()
        .contains(too_deep));
}

// ── Section 10: a member named by its parameter ────────────────────────────

/// Two witnesses and two traces, as in docs/AI_AUTHORING.md: `confront`
/// names a witness by `witness` and a trace by `trace`, and `pick` names the
/// witness a verdict rests on. The trace block comes first, so a trace that
/// shows nothing is refused before a witness's copy can withdraw anything
/// because of it.
const CASE: &str = "place yard kind farm;
entity ann kind witness at yard;
entity bob kind witness at yard;
entity prints kind trace at yard;
entity can kind trace at yard;
claim guilty;
state basis = 0;
decisions verdict limit 4;
event ask witness kind witness, says min 0 max 3;
event examine trace kind trace, says min 0 max 3;
event confront witness kind witness, trace kind trace, recants in no yes;
event pick witness kind witness;
for trace as $t routed by trace {
    evidence ev_$t from \"the $t\";
    state $t_says = 0 min 0 max 3;
    on examine reveal ev_$t opposes guilty;
    on examine set $t_says = qualified(says, ev_$t);
    on confront when $t_says == 0 reject \"That trace shows nothing yet.\";
};
";

const WITNESSES: &str = "for witness as $w routed by witness";

/// The witness block's declarations, and its rules before the two below.
const ACCOUNTS: &str = "
    evidence ev_$w from \"$w's account\";
    state $w_says = 0 min 0 max 3;
    on ask reveal ev_$w supports guilty;
    on ask set $w_says = qualified(says, ev_$w);
    on pick set basis = $w_says;
    on confront when not observed(ev_$w) reject \"They have not given an account yet.\";
    on confront when withdrawn(ev_$w) reject \"They have already taken it back.\";
    bind $w.status = \"on record\" when observed(ev_$w);
    bind $w.status = \"taken back\" when withdrawn(ev_$w);
";

/// The two rules that name the trace the event names, by `$trace`.
const RECANT: &str =
    "    on confront when $w_says == $trace_says reject \"That trace agrees with them.\";
    on confront when recants == recants.yes withdraw ev_$w because ev_$trace;
";

/// The same two rules, each written by hand once for each trace, as a
/// program without section 10 writes them.
const RECANT_BY_HAND: &str = "    on confront when trace == 1 and $w_says == prints_says reject \"That trace agrees with them.\"; on confront when trace == 2 and $w_says == can_says reject \"That trace agrees with them.\";
    on confront when trace == 1 and recants == recants.yes withdraw ev_$w because ev_prints; on confront when trace == 2 and recants == recants.yes withdraw ev_$w because ev_can;
";

/// After the witness block: the verdict rests on the account picked, and
/// comes loose when that account is taken back.
const VERDICT: &str = "on pick commit verdict because enough using basis;
on confront when committed(verdict) and not reopened(verdict) and rests_on_withdrawn(verdict) reopen verdict because caveated(basis, withdrawn);
bind case.verdict = \"none\";
bind case.verdict = \"held\" when committed(verdict) and not reopened(verdict);
bind case.verdict = \"loose\" when reopened(verdict);
";

/// The case with a witness block, `header { body };`, and the verdict.
fn case(header: &str, body: &str) -> String {
    format!("{CASE}{header} {{{body}}};\n{VERDICT}")
}

/// What the witness block expands to, and the line after it.
fn witnesses(header: &str, body: &str) -> Result<String, String> {
    let before = repeat::expand(CASE).unwrap().len();
    repeat::expand(&case(header, body))
        .map(|text| text[before..text.len() - VERDICT.len()].to_string())
}

/// Each outcome, as accepted or as its refusal's message, and the snapshot
/// after the last, its `source_id` cleared: the programs compared are
/// different sources.
fn run_case(source: &str, events: &[(&str, &str)]) -> (Vec<String>, ReactiveSnapshot) {
    let mut session =
        ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("{error}\n{source}"));
    let outcomes = events
        .iter()
        .map(
            |(event, payload)| match session.dispatch_outcome_json(event, payload) {
                Ok(outcome) => match outcome.result {
                    DispatchResult::Accepted { .. } => "accepted".to_string(),
                    DispatchResult::Rejected { message, .. } => message,
                },
                Err(fatal) => panic!("{event} {payload}: {}", fatal.message),
            },
        )
        .collect();
    let mut snapshot = session.snapshot();
    snapshot.source_id.clear();
    (outcomes, snapshot)
}

/// Bob gives an account, a verdict rests on it, and he takes it back when
/// shown the can, which says otherwise.
const RECANTING: [(&str, &str); 9] = [
    ("ask", r#"{"witness":"bob","says":2}"#),
    ("examine", r#"{"trace":"can","says":1}"#),
    ("pick", r#"{"witness":"bob"}"#),
    (
        "confront",
        r#"{"witness":"bob","trace":"prints","recants":"yes"}"#,
    ),
    (
        "confront",
        r#"{"witness":"ann","trace":"can","recants":"yes"}"#,
    ),
    ("examine", r#"{"trace":"prints","says":2}"#),
    (
        "confront",
        r#"{"witness":"bob","trace":"prints","recants":"yes"}"#,
    ),
    (
        "confront",
        r#"{"witness":"bob","trace":"can","recants":"yes"}"#,
    ),
    (
        "confront",
        r#"{"witness":"bob","trace":"can","recants":"yes"}"#,
    ),
];

const RECANTING_OUTCOMES: [&str; 9] = [
    "accepted",
    "accepted",
    "accepted",
    "That trace shows nothing yet.",
    "They have not given an account yet.",
    "accepted",
    "That trace agrees with them.",
    "accepted",
    "They have already taken it back.",
];

#[test]
fn a_rule_that_names_a_member_by_its_parameter_is_the_rule_written_for_each_by_hand() {
    // Each witness's copy of a rule, once for each trace, is the rule that
    // was written by hand for that trace.
    let named = witnesses(WITNESSES, &format!("{ACCOUNTS}{RECANT}")).unwrap();
    assert_eq!(
        named,
        witnesses(WITNESSES, &format!("{ACCOUNTS}{RECANT_BY_HAND}")).unwrap()
    );
    // Every pair, in full.
    assert_eq!(
        witnesses(
            WITNESSES,
            "\n    on confront when recants == recants.yes withdraw ev_$w because ev_$trace;\n"
        )
        .unwrap(),
        "\n    on confront when witness == 1 and trace == 1 and recants == recants.yes withdraw ev_ann because ev_prints; \
         on confront when witness == 1 and trace == 2 and recants == recants.yes withdraw ev_ann because ev_can;\n\
         \n    on confront when witness == 2 and trace == 1 and recants == recants.yes withdraw ev_bob because ev_prints; \
         on confront when witness == 2 and trace == 2 and recants == recants.yes withdraw ev_bob because ev_can;\n\n"
    );
}

#[test]
fn the_copies_go_in_the_kinds_order_where_the_rule_is_adding_no_line() {
    let body = format!("{ACCOUNTS}{RECANT}");
    let named = witnesses(WITNESSES, &body).unwrap();
    // As many lines as the same block names only its own witness in.
    let own = witnesses(WITNESSES, &body.replace("$trace", "$w")).unwrap();
    assert_eq!(named.lines().count(), own.lines().count());
    // Member-major: ann's copies, in the order the traces are declared, then
    // bob's, each on the line the rule is written on, right after the
    // member's copies of the rule before it.
    let lines = named.lines().collect::<Vec<_>>();
    let at = |text: &str| lines.iter().position(|line| line.contains(text)).unwrap();
    assert_eq!(
        [lines[at("withdraw ev_ann")], lines[at("withdraw ev_bob")]],
        [
            "    on confront when witness == 1 and trace == 1 and recants == recants.yes withdraw ev_ann because ev_prints; \
             on confront when witness == 1 and trace == 2 and recants == recants.yes withdraw ev_ann because ev_can;",
            "    on confront when witness == 2 and trace == 1 and recants == recants.yes withdraw ev_bob because ev_prints; \
             on confront when witness == 2 and trace == 2 and recants == recants.yes withdraw ev_bob because ev_can;",
        ]
    );
    assert_eq!(at("withdraw ev_ann"), at("ann_says == prints_says") + 1);
    assert!(at("withdraw ev_ann") < at("bob_says == prints_says"));
}

#[test]
fn each_copy_gets_its_route_where_section_3_puts_it_with_q_after_p() {
    for (written, first) in [
        (
            "on confront withdraw ev_$w because ev_$trace;",
            "on confront when witness == 1 and trace == 1 withdraw ev_ann because ev_prints;",
        ),
        (
            "on confront when recants == recants.yes withdraw ev_$w because ev_$trace;",
            "on confront when witness == 1 and trace == 1 and recants == recants.yes withdraw ev_ann because ev_prints;",
        ),
        (
            "on confront when recants == recants.yes or $trace_says == 3 withdraw ev_$w because ev_$trace;",
            "on confront when witness == 1 and trace == 1 and (recants == recants.yes or prints_says == 3) withdraw ev_ann because ev_prints;",
        ),
    ] {
        let expanded = witnesses(WITNESSES, &format!("\n    {written}\n")).unwrap();
        let copy = expanded.lines().nth(1).unwrap().trim_start();
        assert_eq!(&copy[..first.len()], first, "{written}");
    }
}

#[test]
fn q_composes_into_names_dotted_names_and_quoted_text_as_name_does() {
    let expanded = witnesses(
        WITNESSES,
        "\n    on confront when $trace_says > $w_says and $trace.x >= 0 reject \"$w: $trace, ev_$trace\";\n",
    )
    .unwrap();
    assert!(
        expanded.contains(
            "on confront when witness == 1 and trace == 2 and can_says > ann_says and can.x >= 0 reject \"ann: can, ev_can\";"
        ),
        "{expanded}"
    );
}

/// Two exhibits, a witness, and events that name exhibits and witnesses.
/// No entity is declared `kind suspect`.
const EXHIBITS: &str = "place yard kind farm;
entity x_a kind exhibit at yard;
entity x_b kind exhibit at yard;
entity ann kind witness at yard;
event confront about kind exhibit, shown kind exhibit, recants in no yes;
event compare about kind exhibit, first kind exhibit, second kind exhibit;
event rank about kind exhibit, index_of kind exhibit;
event testify about kind exhibit, by kind witness;
event accuse about kind exhibit, by kind suspect;
event point by kind witness;
";

const EXHIBIT_BLOCK: &str = "for exhibit as $w routed by about";

/// What `header { body };` after the exhibits expands to.
fn exhibits(header: &str, body: &str) -> Result<String, String> {
    repeat::expand(&format!("{EXHIBITS}{header} {{{body}}};\n"))
        .map(|text| text[EXHIBITS.len()..].to_string())
}

fn exhibits_refused(header: &str, body: &str) -> String {
    match exhibits(header, body) {
        Ok(text) => panic!("expected a refusal, expanded to:\n{text}"),
        Err(error) => error,
    }
}

#[test]
fn over_one_kind_every_ordered_pair_is_copied_the_same_member_twice_too() {
    assert_eq!(
        exhibits(
            EXHIBIT_BLOCK,
            "\n    on confront when $w_n == $shown_n reject \"$shown agrees with $w\";\n"
        )
        .unwrap(),
        "\n    on confront when about == 1 and shown == 1 and x_a_n == x_a_n reject \"x_a agrees with x_a\"; \
         on confront when about == 1 and shown == 2 and x_a_n == x_b_n reject \"x_b agrees with x_a\";\n\
         \n    on confront when about == 2 and shown == 1 and x_b_n == x_a_n reject \"x_a agrees with x_b\"; \
         on confront when about == 2 and shown == 2 and x_b_n == x_b_n reject \"x_b agrees with x_b\";\n\n"
    );
}

#[test]
fn a_withdrawal_rests_on_the_trace_the_event_names_and_a_verdict_comes_loose() {
    let (outcomes, snapshot) =
        run_case(&case(WITNESSES, &format!("{ACCOUNTS}{RECANT}")), &RECANTING);
    assert_eq!(outcomes, RECANTING_OUTCOMES);
    // The rejected events change nothing, so the recant is the fifth event
    // that was accepted.
    assert_eq!(
        serde_json::to_value(&snapshot.withdrawals).unwrap(),
        serde_json::json!([{"evidence": "ev_bob", "because": "ev_can", "sequence": 5, "event": "confront"}])
    );
    assert_eq!(
        snapshot.bindings["bob"]["status"],
        BindingValue::Text("taken back".into())
    );
    // The verdict rested on bob's account, so it came loose.
    assert_eq!(
        snapshot.bindings["case"]["verdict"],
        BindingValue::Text("loose".into())
    );
    // Written by hand, one rule for each trace: the same records.
    assert_eq!(
        run_case(
            &case(WITNESSES, &format!("{ACCOUNTS}{RECANT_BY_HAND}")),
            &RECANTING
        ),
        (outcomes, snapshot)
    );
}

#[test]
fn a_procedure_takes_the_members_symbols_and_is_specialized_for_each_pair() {
    let body = format!(
        "{ACCOUNTS}    on confront when $w_says == $trace_says reject \"That trace agrees with them.\";
    on confront when recants == recants.yes call recant(ev_$w, ev_$trace);
"
    );
    let expanded = witnesses(WITNESSES, &body).unwrap();
    assert!(expanded.contains(
        "on confront when witness == 2 and trace == 2 and recants == recants.yes call recant(ev_bob, ev_can);"
    ));
    let source = format!(
        "{}proc recant(account evidence, trace evidence) {{\n    withdraw account because trace;\n}};\n",
        case(WITNESSES, &body)
    );
    // The same records as the rule that withdraws the account itself.
    assert_eq!(
        run_case(&source, &RECANTING),
        run_case(&case(WITNESSES, &format!("{ACCOUNTS}{RECANT}")), &RECANTING)
    );
}

#[test]
fn a_copy_that_its_route_stops_adds_nothing_to_lineage() {
    // reactive.rs builds an event's parameters as plain values
    // (`Tracked::plain` in `run_event`). A route compares only parameters,
    // so a route that does not hold has no provenance, and a copy it stops
    // retains nothing (`retain_skipped_effect` returns on an empty guard).
    // The copy that runs is then the rule written by hand, alone. Section 10
    // rests on this: if parameters ever carry provenance, this test fails.
    let doubt = "    state $w_doubt = 0 min 0 max 3;
    bind $w.doubt = $w_doubt;
";
    // The withdrawal reads both accounts before the recant. A copy that
    // does not withdraw, with a guard that read them, qualifies `withdrawn`
    // with them; a copy its route stops reads neither.
    let rules = "    on confront when $w_says == $trace_says reject \"That trace agrees with them.\";
    on confront when $w_says != $trace_says and recants == recants.yes withdraw ev_$w because ev_$trace;
    on confront when recants == recants.no set $w_doubt = $trace_says;
";
    let named = case(WITNESSES, &format!("{ACCOUNTS}{doubt}{rules}"));
    // Only the copies for bob and the can, written by hand after the block.
    // Bob's copies are the last rules in the block, so they run in the same
    // place.
    let running = case(WITNESSES, &format!("{ACCOUNTS}{doubt}")).replace(
        VERDICT,
        &format!(
            "on confront when witness == 2 and trace == 2 and bob_says == can_says reject \"That trace agrees with them.\";
on confront when witness == 2 and trace == 2 and bob_says != can_says and recants == recants.yes withdraw ev_bob because ev_can;
on confront when witness == 2 and trace == 2 and recants == recants.no set bob_doubt = can_says;
{VERDICT}"
        ),
    );
    let events = [
        ("ask", r#"{"witness":"bob","says":2}"#),
        ("ask", r#"{"witness":"ann","says":3}"#),
        ("examine", r#"{"trace":"can","says":1}"#),
        ("examine", r#"{"trace":"prints","says":3}"#),
        ("pick", r#"{"witness":"bob"}"#),
        (
            "confront",
            r#"{"witness":"bob","trace":"can","recants":"no"}"#,
        ),
        (
            "confront",
            r#"{"witness":"bob","trace":"can","recants":"yes"}"#,
        ),
    ];
    let (outcomes, snapshot) = run_case(&named, &events);
    assert_eq!(outcomes, ["accepted"; 7]);
    assert_eq!(run_case(&running, &events), (outcomes, snapshot.clone()));
    // The records compared are not empty.
    assert!(!snapshot.binding_explanations.is_empty());
    assert!(!snapshot.predicate_qualifications.is_empty());
    assert!(snapshot.qualified_values.contains_key("ann_doubt"));
    assert_eq!(snapshot.values["bob_doubt"], 1.0);
}

/// Before the Rain's confront rules as Vessel first wrote them (its
/// FINDINGS F3): one line for each witness to compare, and one for each
/// trace to recant because of.
const BEFORE_THE_RAIN_BY_HAND: &str = r#"// What was shown must be an examined trace that says otherwise.
for exhibit as $s {
    on confront when shown == $index and $index <= 3 reject "Show them a trace.";
    on confront when shown == $index and $s_points == 0 reject "That trace shows nothing yet.";
    on confront when shown == $index and about == 1 and x_farmer_points == $s_points reject "That trace agrees with them.";
    on confront when shown == $index and about == 2 and x_farmhand_points == $s_points reject "That trace agrees with them.";
    on confront when shown == $index and about == 3 and x_neighbour_points == $s_points reject "That trace agrees with them.";
};

// A recant takes the account back because of the trace that was shown.
for exhibit as $w routed by about {
    on confront when recants == recants.yes and shown == 4 withdraw ev_$w because ev_x_prints;
    on confront when recants == recants.yes and shown == 5 withdraw ev_$w because ev_x_can;
    on confront when recants == recants.yes and shown == 6 withdraw ev_$w because ev_x_lamp;
    on confront when recants == recants.yes and shown == 7 withdraw ev_$w because ev_x_hay;
};
"#;

/// The same rules in game/before_the_rain.cav, the trace named by `$shown`.
const BEFORE_THE_RAIN_NAMED: &str = r#"// What was shown must be an examined trace.
for exhibit as $s routed by shown {
    on confront when $index <= 3 reject "Show them a trace.";
    on confront when $s_points == 0 reject "That trace shows nothing yet.";
};

// A recant takes the account back because of the trace that was shown, and
// only a trace that says otherwise can bring one.
for exhibit as $w routed by about {
    on confront when $w_points == $shown_points reject "That trace agrees with them.";
    on confront when recants == recants.yes withdraw ev_$w because ev_$shown;
};
"#;

/// Before the Rain written by hand, and as the game directory holds it.
fn before_the_rain() -> (String, String) {
    let named = read("../game/before_the_rain.cav");
    assert_eq!(named.matches(BEFORE_THE_RAIN_NAMED).count(), 1);
    (
        named.replace(BEFORE_THE_RAIN_NAMED, BEFORE_THE_RAIN_BY_HAND),
        named,
    )
}

/// A fixed-seed xorshift generator, so every run sends the same histories.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }

    fn pick<'a>(&mut self, from: &[&'a str]) -> &'a str {
        from[(self.next() * from.len() as f64) as usize]
    }
}

/// One of Before the Rain's histories: mostly accounts and examinations
/// first, then confrontations, holds, rulings and the rest, in any order a
/// player might send them. About one event in six shows a witness a trace
/// and has them recant, so that recants are not rare.
fn history(rng: &mut Rng, length: usize) -> Vec<(&'static str, String)> {
    const EXHIBITS: [&str; 7] = [
        "x_farmer",
        "x_farmhand",
        "x_neighbour",
        "x_prints",
        "x_can",
        "x_lamp",
        "x_hay",
    ];
    const SUSPECTS: [&str; 3] = ["farmhand", "neighbour", "accident"];
    let mut events = Vec::new();
    for step in 0..length {
        let roll = rng.next();
        let early = step < length / 3;
        let event = if roll < if early { 0.3 } else { 0.1 } {
            let (about, accuses) = (rng.pick(&EXHIBITS), rng.pick(&SUSPECTS));
            (
                "ask",
                format!(r#"{{"about":"{about}","accuses":"{accuses}"}}"#),
            )
        } else if roll < if early { 0.6 } else { 0.2 } {
            let (about, points) = (rng.pick(&EXHIBITS), (rng.next() * 4.0) as u32);
            (
                "examine",
                format!(r#"{{"about":"{about}","points":{points}}}"#),
            )
        } else if roll < 0.45 {
            let (about, shown) = (rng.pick(&EXHIBITS), rng.pick(&EXHIBITS));
            let recants = rng.pick(&["no", "yes", "yes"]);
            (
                "confront",
                format!(r#"{{"about":"{about}","shown":"{shown}","recants":"{recants}"}}"#),
            )
        } else if roll < 0.58 {
            let (suspect, first, second) = (
                rng.pick(&SUSPECTS),
                rng.pick(&EXHIBITS),
                rng.pick(&EXHIBITS),
            );
            (
                "hold",
                format!(r#"{{"suspect":"{suspect}","first":"{first}","second":"{second}"}}"#),
            )
        } else if roll < 0.63 {
            ("warrant", "{}".into())
        } else if roll < 0.67 {
            let found = rng.pick(&["no", "yes"]);
            ("search_cot", format!(r#"{{"found":"{found}"}}"#))
        } else if roll < 0.7 {
            ("rain", "{}".into())
        } else if roll < 0.76 {
            let dt = (rng.next() * 3.0) as u32;
            ("advance", format!(r#"{{"dt":{dt}}}"#))
        } else if roll < 0.78 {
            ("rule", "{}".into())
        } else if roll < 0.79 {
            ("dusk", "{}".into())
        } else if roll < 0.82 {
            let about = rng.pick(&EXHIBITS);
            ("expose", format!(r#"{{"about":"{about}"}}"#))
        } else if roll < 0.84 {
            let culprit = rng.pick(&SUSPECTS);
            ("close", format!(r#"{{"culprit":"{culprit}"}}"#))
        } else {
            let (about, shown) = (rng.pick(&EXHIBITS[..3]), rng.pick(&EXHIBITS[3..]));
            (
                "confront",
                format!(r#"{{"about":"{about}","shown":"{shown}","recants":"yes"}}"#),
            )
        };
        events.push(event);
    }
    events
}

/// What an outcome shows: the snapshot, its `source_id` cleared, or the
/// refusal, with the rule numbers in its message taken out, since the two
/// programs number their rules differently.
#[derive(Debug, PartialEq)]
enum Seen {
    Accepted(Box<ReactiveSnapshot>),
    Rejected(String, String, String),
    Fatal(String, String),
}

fn seen(session: &mut ReactiveSession, event: &str, payload: &str) -> Seen {
    match session.dispatch_outcome_json(event, payload) {
        Ok(outcome) => match outcome.result {
            DispatchResult::Accepted { mut snapshot } => {
                snapshot.source_id.clear();
                Seen::Accepted(snapshot)
            }
            DispatchResult::Rejected {
                origin,
                code,
                message,
            } => Seen::Rejected(
                format!("{origin:?}"),
                format!("{code:?}"),
                without_rule_numbers(&message),
            ),
        },
        Err(fatal) => Seen::Fatal(fatal.code.into(), without_rule_numbers(&fatal.message)),
    }
}

/// A message with each `rule N` written `rule _`.
fn without_rule_numbers(message: &str) -> String {
    let mut out = String::new();
    let mut rest = message;
    while let Some(at) = rest.find("rule ") {
        out.push_str(&rest[..at + "rule ".len()]);
        rest = &rest[at + "rule ".len()..];
        let number = rest.len()
            - rest
                .trim_start_matches(|ch: char| ch.is_ascii_digit())
                .len();
        if number > 0 {
            out.push('_');
        }
        rest = &rest[number..];
    }
    out.push_str(rest);
    out
}

#[test]
fn before_the_rain_runs_every_history_as_the_program_written_by_hand() {
    let (by_hand, named) = before_the_rain();
    // F3's seven rules are two, and confront has 49 more rules to try.
    let count = |source: &str| {
        repeat::expand(source)
            .unwrap()
            .matches("on confront when")
            .count()
    };
    assert_eq!((count(&by_hand), count(&named)), (86, 135));

    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let (mut events, mut accepted, mut withdrawals) = (0, 0, 0);
    let mut refusals = std::collections::BTreeSet::new();
    for _ in 0..70 {
        let mut hand = ReactiveSession::from_source(&by_hand).unwrap();
        let mut session = ReactiveSession::from_source(&named).unwrap();
        for (event, payload) in history(&mut rng, 30) {
            let outcome = seen(&mut session, event, &payload);
            assert_eq!(
                outcome,
                seen(&mut hand, event, &payload),
                "{event} {payload}"
            );
            events += 1;
            match outcome {
                Seen::Accepted(_) => accepted += 1,
                Seen::Rejected(_, _, message) => {
                    refusals.insert(message);
                }
                // The rejects come first, so no recant withdraws because of
                // a trace that was never read.
                Seen::Fatal(code, message) => panic!("{event} {payload}: {code} {message}"),
            }
        }
        withdrawals += session.snapshot().withdrawals.len();
        // The final save restores, into the program it was made by.
        for (source, session) in [(&by_hand, &hand), (&named, &session)] {
            let restored =
                ReactiveSession::restore_json(source, &session.save_json().unwrap()).unwrap();
            assert_eq!(restored.snapshot(), session.snapshot());
        }
    }
    assert!(events >= 2000, "{events} events");
    assert!(
        accepted > 500 && withdrawals > 10,
        "{accepted} accepted, {withdrawals} withdrawn"
    );
    for message in [
        "Show them a trace.",
        "That trace shows nothing yet.",
        "That trace agrees with them.",
        "They have already taken it back.",
    ] {
        assert!(refusals.contains(message), "{message}: {refusals:?}");
    }
}

// Section 10's errors, in the order section 7 lists them.

#[test]
fn a_word_both_the_binding_and_a_parameter_begin_is_refused() {
    // A longer parameter, and one as long as the binding.
    assert_eq!(
        exhibits_refused(
            "for exhibit as $s routed by about",
            "\n    state $s_n = 0;\n    on confront when $s_n == $shown_n reject \"x\";\n"
        ),
        "`$shown_n` in `on confront` in `for exhibit as $s routed by about` could be `$s` or the exhibit `confront` names by `shown`; rename `$s` so that only one of them begins it, or write the rule in a block routed by `shown`"
    );
    assert_eq!(
        exhibits_refused(
            "for exhibit as $shown routed by about",
            "\n    on confront when $shown_n == 0 reject \"x\";\n"
        ),
        "`$shown_n` in `on confront` in `for exhibit as $shown routed by about` could be `$shown` or the exhibit `confront` names by `shown`; rename `$shown` so that only one of them begins it, or write the rule in a block routed by `shown`"
    );
    // P longer than the binding, and a parameter longer than `$index`.
    assert_eq!(
        exhibits_refused(
            "for exhibit as $a routed by about",
            "\n    on confront when $about_n == 0 reject \"x\";\n"
        ),
        "`$about_n` in `on confront` in `for exhibit as $a routed by about` could be `$a` or the exhibit `confront` names by `about`; rename `$a` so that only one of them begins it"
    );
    assert_eq!(
        exhibits_refused(
            EXHIBIT_BLOCK,
            "\n    on rank when $index_of == 1 and $w_n == 0 reject \"x\";\n"
        ),
        "`$index_of` in `on rank` in `for exhibit as $w routed by about` could be `$index` or the exhibit `rank` names by `index_of`; write the rule in a block routed by `index_of`"
    );
    // A binding named P keeps its word: it names the same member.
    assert_eq!(
        exhibits(
            "for exhibit as $about routed by about",
            "\n    on confront when $about_n == $shown_n reject \"x\";\n"
        )
        .unwrap()
        .lines()
        .nth(1)
        .unwrap(),
        "    on confront when about == 1 and shown == 1 and x_a_n == x_a_n reject \"x\"; on confront when about == 1 and shown == 2 and x_a_n == x_b_n reject \"x\";"
    );
}

#[test]
fn p_named_by_its_parameter_is_refused() {
    assert_eq!(
        exhibits_refused(
            EXHIBIT_BLOCK,
            "\n    on confront when $about_n == 0 and $w_n == 0 reject \"x\";\n"
        ),
        "`$about_n` in `on confront` in `for exhibit as $w routed by about`: the block is routed by `about`, so the exhibit it names is `$w`; write `$w` for `$about`"
    );
}

#[test]
fn a_parameter_of_another_form_names_no_member() {
    assert_eq!(
        exhibits_refused(
            EXHIBIT_BLOCK,
            "\n    on confront when $recants_x == 0 and $w_n == 0 reject \"x\";\n"
        ),
        "`$recants_x` in `on confront` in `for exhibit as $w routed by about`: `confront` declares `recants in no yes`, and a `$` names a member only by a `kind` parameter"
    );
}

#[test]
fn a_rule_that_names_members_by_two_parameters_is_refused() {
    assert_eq!(
        exhibits_refused(
            EXHIBIT_BLOCK,
            "\n    on compare when $first_n == $second_n and $w_n == 0 reject \"x\";\n"
        ),
        "`on compare` in `for exhibit as $w routed by about` names members by `$first` and `$second`; a rule names a member by one parameter at most, so write it in a block routed by one of them"
    );
}

#[test]
fn a_rule_that_names_members_only_by_a_parameter_is_refused() {
    assert_eq!(
        exhibits_refused(
            EXHIBIT_BLOCK,
            "\n    on confront when $shown_n == 0 reject \"x\";\n"
        ),
        "`on confront` in `for exhibit as $w routed by about` names members only by `$shown`, and neither `$w` nor `$index`; each exhibit's copy of it would be the same rule, so write it in a block routed by `shown`"
    );
}

#[test]
fn a_parameter_whose_kind_has_no_top_level_member_is_refused() {
    assert_eq!(
        exhibits_refused(
            EXHIBIT_BLOCK,
            "\n    on accuse when $by_n == 0 and $w_n == 0 reject \"x\";\n"
        ),
        "`$by` in `on accuse` in `for exhibit as $w routed by about`: no entity is declared `kind suspect` at the top level of this part, so `$by` names no member"
    );
    // Of another kind, with a member, the rule is copied for it.
    let body = "\n    on testify when $by_n == 0 and $w_n == 0 reject \"x\";\n";
    assert_eq!(
        exhibits(EXHIBIT_BLOCK, body)
            .unwrap()
            .lines()
            .nth(1)
            .unwrap(),
        "    on testify when about == 1 and by == 1 and ann_n == 0 and x_a_n == 0 reject \"x\";"
    );
    // An entity of that kind in a for block is counted by the parameter.
    let zones =
        "entity z1 kind zone at yard;\nfor zone as $z { entity $z_w kind witness at yard; };\n";
    assert_eq!(
        repeat::expand(&format!("{EXHIBITS}{zones}{EXHIBIT_BLOCK} {{{body}}};\n")).unwrap_err(),
        "`$by` in `on testify` in `for exhibit as $w routed by about`: `by` counts entity `$z_w` of kind witness, declared in a for block, and `$by` does not; declare it at the top level of this part"
    );
}

#[test]
fn a_parameter_names_a_member_only_in_a_routed_rule() {
    let unrouted = |header: &str, event: &str, word: &str| {
        format!("`{word}` in `on {event}` in `{header}`: a `$` names the member a parameter names only in a routed rule (spec/caveat-routed-repetition-0.1.md section 10), and this rule is not routed")
    };
    // In a plain block, and in a rule of a routed block on an event that
    // names no exhibit.
    assert_eq!(
        exhibits_refused(
            "for exhibit as $w",
            "\n    on confront when about == $index and $shown_n == 0 reject \"x\";\n"
        ),
        unrouted("for exhibit as $w", "confront", "$shown_n")
    );
    assert_eq!(
        exhibits_refused(
            EXHIBIT_BLOCK,
            "\n    on confront when $w_n == 0 reject \"x\";\n    on point when $by_n == 0 and $w_n == 0 reject \"y\";\n"
        ),
        unrouted(EXHIBIT_BLOCK, "point", "$by_n")
    );
    // A word the block's own binding begins is its own, as before.
    assert_eq!(
        exhibits(
            "for exhibit as $s",
            "\n    on confront when shown == $index and $shown_n == 0 reject \"x\";\n"
        )
        .unwrap()
        .lines()
        .nth(1)
        .unwrap(),
        "    on confront when shown == 1 and x_ahown_n == 0 reject \"x\";"
    );
}

#[test]
fn q_is_not_bound_outside_a_routed_rule() {
    for statement in [
        "state $shown_n = 0;",
        "bind $shown.x = 1;",
        "define $w_d = $shown_n;",
        "proc p_$w() { set $w_n = $shown_n; };",
        "on $w_ev when $shown_n == 0 reject \"x\";",
    ] {
        assert_eq!(
            exhibits_refused(
                EXHIBIT_BLOCK,
                &format!("\n    state $w_n = 0;\n    on confront when $w_n == 0 reject \"x\";\n    {statement}\n")
            ),
            format!(
                "${} is not bound in this for block; $ is not a literal",
                if statement.starts_with("bind") { "shown" } else { "shown_n" }
            ),
            "{statement}"
        );
    }
}

/// Members of kind `member`, and a block routed by `target` whose rule on
/// `meet` names the other member, with `extra` rules after it.
fn crowd(members: usize, extra: &str) -> String {
    let mut source = String::from("place yard kind farm;\n");
    for member in 1..=members {
        source.push_str(&format!("entity m{member} kind member at yard;\n"));
    }
    source.push_str(&format!(
        "event meet target kind member, other kind member;
for member as $m routed by target {{
    state $m_n = 0;
    on meet when $other_n == 0 set $m_n = 1;
{extra}}};
"
    ));
    source
}

#[test]
fn an_event_that_a_block_gives_more_work_than_one_event_can_do_is_refused() {
    let too_much = |steps: usize| {
        format!("`for member as $m routed by target` writes {steps} rules on `meet`, and one event does at most 4096 steps of work; each copy spends a step whether its route holds or not")
    };
    // 64 members, each with 64 copies: every step one event can take.
    let mut session = ReactiveSession::from_source(&crowd(64, "")).unwrap();
    session
        .dispatch_json("meet", r#"{"target":"m64","other":"m1"}"#)
        .unwrap();
    assert_eq!(session.snapshot().values["m64_n"], 1.0);
    assert_eq!(repeat::expand(&crowd(65, "")).unwrap_err(), too_much(4225));
    // Every rule of the block on the event counts, once per member.
    let more = "    on meet set $m_n = 2;\n";
    let mut session = ReactiveSession::from_source(&crowd(63, more)).unwrap();
    session
        .dispatch_json("meet", r#"{"target":"m63","other":"m63"}"#)
        .unwrap();
    assert_eq!(session.snapshot().values["m63_n"], 2.0);
    assert_eq!(
        repeat::expand(&crowd(64, more)).unwrap_err(),
        too_much(4160)
    );
    // It is refused before a `$` name that is not bound.
    assert_eq!(
        repeat::expand(&crowd(65, "    state $typo = 0;\n")).unwrap_err(),
        too_much(4225)
    );
}

#[test]
fn section_10_errors_come_after_routing_and_word_by_word_in_a_rule() {
    // Routing reads every rule first: an event it cannot read is refused
    // before a rule above it that names members only by `$shown`.
    assert_eq!(
        exhibits_refused(
            EXHIBIT_BLOCK,
            "\n    on confront when $shown_n == 0 reject \"x\";\n    on nowhere set $w_n = 1;\n"
        ),
        "`on nowhere` in `for exhibit as $w routed by about`: no event `nowhere` is declared in this part, so the block cannot tell whether it names a exhibit"
    );
    // In a rule, each word first, then the rule: `$about` is refused before
    // the two parameters, and before the missing `$w`.
    assert!(exhibits_refused(
        EXHIBIT_BLOCK,
        "\n    on compare when $first_n == $second_n and $about_n == 0 reject \"x\";\n"
    )
    .starts_with("`$about_n` in `on compare`"));
    // And rule by rule: the first rule's error is the one reported.
    assert!(exhibits_refused(
        EXHIBIT_BLOCK,
        "\n    on confront when $recants_x == $w_n reject \"x\";\n    on confront when $about_n == $w_n reject \"x\";\n"
    )
    .starts_with("`$recants_x`"));
    // A word refused as not routed comes before a `$` name that is not bound
    // in an earlier statement.
    assert!(exhibits_refused(
        "for exhibit as $w",
        "\n    state $typo = 0;\n    on confront when about == $index and $shown_n == 0 reject \"x\";\n"
    )
    .starts_with("`$shown_n` in `on confront`"));
}

#[test]
fn a_rule_that_names_a_member_by_its_parameter_hides_no_event_its_block_declares() {
    // Each exhibit's block declares its own `nudge` event, and has a rule
    // that names the exhibit shown. A routed block reads those events: they
    // are read without the block's rules.
    let source = format!(
        "{EXHIBITS}for exhibit as $w routed by about {{
    event nudge_$w target kind exhibit;
    state $w_n = 0;
    on confront when $w_n == $shown_n reject \"x\";
}};
for exhibit as $e routed by target {{
    state $e_m = 0;
    on nudge_x_a set $e_m = 1;
}};
"
    );
    let expanded = repeat::expand(&source).unwrap();
    assert!(expanded.contains("on nudge_x_a when target == 2 set x_b_m = 1;"));
}
