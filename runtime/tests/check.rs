//! `caveat check`: when each warning appears, and when it must stay quiet.
//! See spec/caveat-check-0.1.md.
use caveat_runtime::reactive::{check_source, CheckReport, ReactiveSession};

const DECLARATIONS: &str = r#"claim frost_risk;
evidence probe from "a soil probe";
evidence drone from "a survey drone";
evidence kite from "a kite camera";
evidence thaw from "a thaw report";
caveat uncalibrated consequence material;
uncalibrated qualifies drone;
readings soil from probe limit 12;
readings aerial from drone limit 12;
readings overhead from kite limit 12;
decisions uncover limit 4;
state last_probe = 0 min -40 max 60;
state last_drone = 0 min -40 max 60;
event probe_read celsius min -40 max 60;
event drone_read celsius min -40 max 60;
event kite_read reading min -40 max 60;
event decide;
event rethink;
"#;

/// The same two rules for each instrument, written out by hand.
const BY_HAND: &str = r#"on probe_read when celsius <= 2 sample soil = celsius supports frost_risk;
on probe_read when celsius > 2 sample soil = celsius opposes frost_risk;
on drone_read when celsius <= 2 sample aerial = celsius supports frost_risk;
on drone_read when celsius > 2 sample aerial = celsius opposes frost_risk;
"#;

fn check(source: &str) -> CheckReport {
    check_source(source).unwrap_or_else(|error| panic!("check failed: {error}\n{source}"))
}

fn codes(report: &CheckReport) -> Vec<(&str, usize)> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.line))
        .collect()
}

/// The line of the first line of `source` that starts with `prefix`.
fn line_of(source: &str, prefix: &str) -> usize {
    source
        .lines()
        .position(|line| line.trim_start().starts_with(prefix))
        .unwrap_or_else(|| panic!("no line starts with {prefix}"))
        + 1
}

#[test]
fn rules_repeated_with_other_streams_are_reported_where_the_repeat_starts() {
    let source = format!("{DECLARATIONS}{BY_HAND}");
    let report = check(&source);
    assert_eq!(
        codes(&report),
        [("C001", line_of(&source, "on drone_read"))]
    );
    let warning = &report.diagnostics[0];
    assert_eq!(
        (warning.name, warning.severity),
        ("repeated-rules", "warning")
    );
    assert_eq!(warning.column, 1);
    assert_eq!(
        warning.message,
        format!(
            "the 2 rules on `drone_read` repeat the rules on `probe_read` (line {}), with `aerial` for `soil`",
            line_of(&source, "on probe_read")
        )
    );
    assert!(warning.suggestion.contains("candidates for one `proc`"));
    assert_eq!(warning.related[0].line, line_of(&source, "on probe_read"));
    assert!(report.suppressed.is_empty());
}

#[test]
fn numbers_and_event_parameters_may_differ_too() {
    let source = format!(
        "{DECLARATIONS}{BY_HAND}on kite_read when reading <= 3 sample overhead = reading supports frost_risk;\non kite_read when reading > 3 sample overhead = reading opposes frost_risk;\n"
    );
    let report = check(&source);
    // Each repeat points at the first block it repeats.
    assert_eq!(
        codes(&report),
        [
            ("C001", line_of(&source, "on drone_read")),
            ("C001", line_of(&source, "on kite_read"))
        ]
    );
    assert!(report.diagnostics[1]
        .message
        .ends_with("with `reading` for `celsius`, `overhead` for `soil`, other numbers"));
}

#[test]
fn nothing_is_reported_where_one_procedure_could_not_take_the_difference() {
    for rules in [
        // A state is not a procedure parameter.
        "on probe_read set last_probe = celsius;\non probe_read sample soil = celsius supports frost_risk;\non drone_read set last_drone = celsius;\non drone_read sample aerial = celsius supports frost_risk;\n",
        // The relation differs.
        "on probe_read sample soil = celsius supports frost_risk;\non probe_read set last_probe = 1;\non drone_read sample aerial = celsius opposes frost_risk;\non drone_read set last_probe = 1;\n",
        // A reading stream in one place, a decision series in the other.
        "on probe_read when history_count(soil) > 1 set last_probe = 1;\non probe_read when history_count(soil) > 2 set last_probe = 2;\non drone_read when history_count(uncover) > 1 set last_probe = 1;\non drone_read when history_count(uncover) > 2 set last_probe = 2;\n",
        // One rule each: a call is no shorter.
        "on probe_read sample soil = celsius supports frost_risk;\non drone_read sample aerial = celsius supports frost_risk;\n",
        // A different number of rules.
        "on probe_read sample soil = celsius supports frost_risk;\non probe_read set last_probe = 1;\non drone_read sample aerial = celsius supports frost_risk;\n",
        // The same name must stand for the same name throughout.
        "on probe_read sample soil = celsius supports frost_risk;\non probe_read sample soil = celsius opposes frost_risk;\non drone_read sample aerial = celsius supports frost_risk;\non drone_read sample overhead = celsius opposes frost_risk;\n",
        // Two names may not become one.
        "on probe_read sample soil = celsius supports frost_risk;\non probe_read sample aerial = celsius opposes frost_risk;\non drone_read sample overhead = celsius supports frost_risk;\non drone_read sample overhead = celsius opposes frost_risk;\n",
        // Different messages.
        "on probe_read when celsius > 60 reject \"probe\";\non probe_read sample soil = celsius supports frost_risk;\non drone_read when celsius > 60 reject \"drone\";\non drone_read sample aerial = celsius supports frost_risk;\n",
        // Already one procedure.
        "proc record(s readings, celsius) { when celsius <= 2 sample s = celsius supports frost_risk; when celsius > 2 sample s = celsius opposes frost_risk; };\non probe_read call record(soil, celsius);\non drone_read call record(aerial, celsius);\n",
    ] {
        let report = check(&format!("{DECLARATIONS}{rules}"));
        assert_eq!(codes(&report), [], "{rules}");
    }
}

#[test]
fn a_for_block_is_already_shared() {
    let source = r#"
place field kind field;
entity north kind plot at field;
entity south kind plot at field;
claim frost_risk;
for plot as $p {
    evidence $p_probe from "a probe";
    readings $p_soil from $p_probe limit 4;
    event $p_read celsius min -40 max 60;
    on $p_read when celsius <= 2 sample $p_soil = celsius supports frost_risk;
    on $p_read when celsius > 2 sample $p_soil = celsius opposes frost_risk;
};
"#;
    assert_eq!(codes(&check(source)), []);
}

const DECIDE: &str =
    "on decide when not committed(uncover) commit uncover because enough using latest(soil);\n";

#[test]
fn a_decision_on_readings_that_nothing_reopens_is_reported_at_its_declaration() {
    let source = format!("{DECLARATIONS}{DECIDE}");
    let report = check(&source);
    assert_eq!(
        codes(&report),
        [("C002", line_of(&source, "decisions uncover"))]
    );
    let warning = &report.diagnostics[0];
    assert_eq!(warning.name, "no-reopening-path");
    assert!(
        warning.message.starts_with(
            "`uncover` is decided on readings from `soil`, and no reopening path was detected"
        ),
        "{}",
        warning.message
    );
    assert!(!warning.message.contains("stale"));
    assert!(warning
        .suggestion
        .starts_with("Confirm that keeping `uncover` fixed is intended."));
    assert!(warning
        .suggestion
        .contains("`decisions uncover limit N reopened by soil`"));
}

#[test]
fn a_guard_on_readings_counts_and_so_does_a_procedure() {
    for rules in [
        "on decide when latest(soil) < 3 and not committed(uncover) commit uncover because enough;\n",
        "on decide when has_sample(aerial) and not committed(uncover) commit uncover because enough;\n",
        "proc decide_on(s readings, d decisions) { when not committed(d) commit d because enough using latest(s); };\non decide call decide_on(soil, uncover);\n",
    ] {
        let source = format!("{DECLARATIONS}{rules}");
        assert_eq!(
            codes(&check(&source)),
            [("C002", line_of(&source, "decisions uncover"))],
            "{rules}"
        );
    }
}

#[test]
fn every_reopening_path_keeps_it_quiet() {
    for reopening in [
        // A rule.
        "on rethink when not observed(thaw) reveal thaw opposes frost_risk;\non rethink when committed(uncover) and not reopened(uncover) reopen uncover because thaw;\n",
        // A procedure taking the series as a parameter.
        "proc reconsider(d decisions, s readings) { when committed(d) and not reopened(d) reopen d because latest(s); };\non probe_read sample soil = celsius supports frost_risk;\non probe_read call reconsider(uncover, soil);\n",
    ] {
        let source = format!("{DECLARATIONS}{DECIDE}{reopening}");
        assert_eq!(codes(&check(&source)), [], "{reopening}");
    }
    // A declared trigger.
    let source = format!(
        "{}{DECIDE}",
        DECLARATIONS.replace(
            "decisions uncover limit 4;",
            "decisions uncover limit 4 reopened by soil;"
        )
    );
    assert_eq!(codes(&check(&source)), []);
}

#[test]
fn a_decision_not_made_on_readings_is_not_reported() {
    for rules in [
        "on decide when not committed(uncover) commit uncover because enough using last_probe;\n",
        "on decide when not committed(uncover) commit uncover because enough;\n",
        // Declared but never made.
        "",
    ] {
        let report = check(&format!("{DECLARATIONS}{rules}"));
        assert_eq!(codes(&report), [], "{rules}");
    }
}

#[test]
fn a_decision_declared_in_a_for_block_is_reported_at_the_template() {
    let source = r#"place field kind field;
entity north kind plot at field;
entity south kind plot at field;
claim frost_risk;
event decide;
for plot as $p {
    evidence $p_probe from "a probe";
    readings $p_soil from $p_probe limit 4;
    decisions $p_cover limit 2;
    on decide when has_sample($p_soil) and not committed($p_cover) commit $p_cover because enough using latest($p_soil);
};
"#;
    let report = check(source);
    let line = line_of(source, "decisions $p_cover");
    assert_eq!(codes(&report), [("C002", line), ("C002", line)]);
    assert_eq!(report.diagnostics[0].column, 5);
    assert!(report.diagnostics[0].message.starts_with("`north_cover`"));
    assert!(report.diagnostics[1].message.starts_with("`south_cover`"));

    // Reopening inside the block keeps it quiet.
    let reopened = source.replace(
        "};\n",
        "    on decide when committed($p_cover) reopen $p_cover because latest($p_soil);\n};\n",
    );
    assert_eq!(codes(&check(&reopened)), []);
}

#[test]
fn an_allow_comment_on_the_line_above_moves_a_warning_to_suppressed() {
    let allowed = format!(
        "{}{}",
        DECLARATIONS.replace(
            "decisions uncover limit 4;",
            "# One decision per season, on purpose.\n# caveat check: allow no-reopening-path\ndecisions uncover limit 4;"
        ),
        BY_HAND.replace(
            "on drone_read when celsius <= 2",
            "// caveat check: allow C001\non drone_read when celsius <= 2"
        )
    ) + DECIDE;
    let report = check(&allowed);
    assert_eq!(codes(&report), []);
    assert_eq!(
        report
            .suppressed
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        ["C002", "C001"]
    );

    // Only the line directly above, and only the check it names.
    for (comment, before) in [
        ("# caveat check: allow C001\n", "decisions uncover"),
        (
            "# caveat check: allow no-reopening-path\n\n",
            "decisions uncover",
        ),
        ("# caveat check: allow no-reopening\n", "decisions uncover"),
        ("# allow no-reopening-path\n", "decisions uncover"),
    ] {
        let source =
            format!("{DECLARATIONS}{DECIDE}").replacen(before, &format!("{comment}{before}"), 1);
        let report = check(&source);
        assert_eq!(report.diagnostics.len(), 1, "{comment:?}");
        assert!(report.suppressed.is_empty(), "{comment:?}");
    }
}

#[test]
fn allow_must_be_a_whole_word() {
    for comment in [
        "# caveat check: allowance C002",
        "# caveat check: allowC002",
        "// caveat check: allowed C002",
        "# caveat check: allow,C002",
    ] {
        let source = format!("{DECLARATIONS}{DECIDE}").replacen(
            "decisions uncover",
            &format!("{comment}\ndecisions uncover"),
            1,
        );
        let report = check(&source);
        assert_eq!(
            codes(&report),
            [("C002", line_of(&source, "decisions uncover"))],
            "{comment}"
        );
        assert!(report.suppressed.is_empty(), "{comment}");
    }
    // Names after the word, separated by commas or spaces, still work.
    for comment in [
        "# caveat check: allow C002",
        "# caveat check: allow C001, no-reopening-path",
        "//caveat check:   allow\tC002",
    ] {
        let source = format!("{DECLARATIONS}{DECIDE}").replacen(
            "decisions uncover",
            &format!("{comment}\ndecisions uncover"),
            1,
        );
        let report = check(&source);
        assert_eq!(codes(&report), [], "{comment}");
        assert_eq!(report.suppressed.len(), 1, "{comment}");
    }
}

#[test]
fn a_last_statement_without_its_semicolon_is_checked() {
    let source = format!("{DECLARATIONS}{}", DECIDE.trim_end().trim_end_matches(';'));
    assert_eq!(
        codes(&check(&source)),
        [("C002", line_of(&source, "decisions uncover"))]
    );
}

#[test]
fn a_program_that_does_not_load_is_an_error_not_a_report() {
    let broken = format!("{DECLARATIONS}on decide sample nowhere = 1 supports frost_risk;");
    let expected = ReactiveSession::from_source(&broken).err().unwrap();
    assert_eq!(check_source(&broken).unwrap_err(), expected);
    assert!(check_source(&format!(
        "{}\n{DECLARATIONS}",
        caveat_runtime::link::BUNDLE_MARKER
    ))
    .unwrap_err()
    .contains("not a bundle"));
}

fn read(file: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
    std::fs::read_to_string(&path).unwrap()
}

#[test]
fn the_shipped_examples_check_clean() {
    for file in [
        "../examples/thermostat_history.cav",
        "../kit/templates/umbrella.cav",
        "../experiments/agent-ledger/ledger.cav",
        "../experiments/agent-ledger/ledger-approved-head.cav",
        "../experiments/agent-ledger/ledger-identifiers.cav",
        "../experiments/agent-ledger/release.cav",
        // The other programs with `for` blocks.
        "../game/glowcap.cav",
        "../experiments/glowcap/caveat/glowcap.cav",
        "../experiments/glowcap/caveat2/glowcap.cav",
        "../experiments/glowcap/caveat3/glowcap.cav",
        "../experiments/glowcap/caveat4/glowcap.cav",
        "../experiments/glowcap/caveat5/glowcap.cav",
    ] {
        let report = check(&read(file));
        assert_eq!(codes(&report), [], "{file}: {:#?}", report.diagnostics);
    }
}

// Trail Rescue resets every tunnel's tallies on each observation, on purpose.
// C003 reports both rules. They carry no allow comment because the Trail
// Rescue dispatch audit pins the file byte for byte.
#[test]
fn trail_rescue_recounts_every_tunnel_on_purpose() {
    let source = read("../game/trail_rescue.cav");
    let support = line_of(&source, "on observe set $t_support");
    let opposition = line_of(&source, "on observe set $t_opposition");
    assert_eq!(
        codes(&check(&source)),
        [
            ("C003", support),
            ("C003", support),
            ("C003", opposition),
            ("C003", opposition)
        ]
    );
}

/// Two plots; `read` names one, `swap` names a probe and `tick` names none.
const PLOTS: &str = r#"place field kind field;
entity north kind plot at field;
entity south kind plot at field;
entity handheld kind probe at field;
state seen = 0;
fn same(a, b) = a == b;
fn slot(i) = i;
event read target kind plot, celsius min -40 max 60;
event swap device kind probe;
event tick dt min 0 max 0.1;
"#;

/// The plots with these rules in a `for` block.
fn plots(rules: &str) -> String {
    format!(
        "{PLOTS}for plot as $p {{
    state $p_last = 0 min -40 max 60;
    state $p_slot = $index;
    define $p_here = target == $index;
{rules}}};
"
    )
}

/// The C003 warnings for the first rule on `read`, one per plot.
fn both_plots(source: &str) -> [(&'static str, usize); 2] {
    let line = line_of(source, "on read");
    [("C003", line), ("C003", line)]
}

#[test]
fn a_member_rule_its_guard_does_not_route_is_reported_for_each_member() {
    let source = plots("    on read when celsius > 2 set $p_last = celsius;\n");
    let report = check(&source);
    assert_eq!(codes(&report), both_plots(&source));
    let (north, south) = (&report.diagnostics[0], &report.diagnostics[1]);
    assert_eq!(
        (north.name, north.severity, north.column),
        ("unrouted-member-rule", "warning", 5)
    );
    assert_eq!(
        north.message,
        "the rule on `read` for `north` runs whichever plot `target` names: its guard does not select one"
    );
    assert_eq!(
        south.message,
        "the rule on `read` for `south` runs whichever plot `target` names: its guard does not select one"
    );
    assert_eq!(
        north.suggestion,
        "If the rule is about the plot the event names, add the selection to its guard, such as `target == $index`. If it should run for every plot on each `read`, put `# caveat check: allow unrouted-member-rule` on the line above it."
    );
    assert!(north.related.is_empty());
}

#[test]
fn every_way_of_selecting_the_member_routes_the_rule() {
    for rules in [
        "on read when target == $index set $p_last = celsius;",
        "on read when $index == target set $p_last = celsius;",
        // A member constant, even another member's.
        "on read when target == target.north set $p_last = celsius;",
        // Expressions that mention the binding or its index.
        "on read when target == target.$p set $p_last = celsius;",
        "on read when target == $p_slot set $p_last = celsius;",
        "on read when $index + 0 == target set $p_last = celsius;",
        "on read when target == -(-$index) set $p_last = celsius;",
        "on read when target == if(celsius > 0, $index, $index) set $p_last = celsius;",
        "on read when target == round($index) set $p_last = celsius;",
        "on read when target == slot($index) set $p_last = celsius;",
        // Through a define, and a define that uses another.
        "on read when $p_here set $p_last = celsius;",
        "define $p_warm = $p_here and celsius > 2;\n    on read when $p_warm set $p_last = celsius;",
        // Among other conjuncts, and inside parentheses.
        "on read when celsius > 2 and target == $index and $p_last < 60 set $p_last = celsius;",
        "on read when (celsius > 2 and target == $index) and $p_last < 60 set $p_last = celsius;",
        "on read when ((target == $index)) set $p_last = celsius;",
        // The member only in quoted text is still a rule about it.
        "on read when target == $index and celsius > 59 reject \"$p is flooded\";",
    ] {
        let source = plots(&format!("    {rules}\n"));
        assert_eq!(codes(&check(&source)), [], "{rules}");
    }
}

#[test]
fn a_selection_that_is_not_a_top_level_conjunct_does_not_count() {
    for rules in [
        "on read when target == $index or celsius > 50 set $p_last = celsius;",
        "on read when not (target != $index) set $p_last = celsius;",
        "on read when not (target == $index) set $p_last = 0;",
        "on read when target != $index set $p_last = 0;",
        "on read when same(target, $index) set $p_last = celsius;",
        "on read when if(celsius > 0, target == $index, false) set $p_last = celsius;",
        // A number, a state, a probe or another parameter is not a selection.
        "on read when target == 1 set $p_last = celsius;",
        "on read when target == device.handheld set $p_last = celsius;",
        "on read when target == seen set $p_last = celsius;",
        "on read when celsius == $index set $p_last = celsius;",
        // A define counts as a whole conjunct or disjunct, and this one is a
        // disjunct beside one that does not select.
        "define $p_either = $p_here or celsius > 50;\n    on read when $p_either set $p_last = celsius;",
        // `$p` only in quoted text.
        "on read when celsius > 59 reject \"$p is flooded\";",
    ] {
        let source = plots(&format!("    {rules}\n"));
        assert_eq!(codes(&check(&source)), both_plots(&source), "{rules}");
    }
}

#[test]
fn rules_that_reach_every_member_by_design_are_not_checked() {
    for rules in [
        // No parameter names a member, or none names a plot.
        "on tick set $p_last = 0;",
        "on swap set $p_last = 0;",
        // The rule does not mention the member, so every copy is the same.
        "on read set seen = seen + 1;",
    ] {
        let source = plots(&format!("    {rules}\n"));
        assert_eq!(codes(&check(&source)), [], "{rules}");
    }
    // Outside a block, `target` may name any plot.
    let source = format!("{PLOTS}on read set seen = target;\n");
    assert_eq!(codes(&check(&source)), []);
}

#[test]
fn a_rule_on_the_members_own_event_is_not_checked() {
    let source = format!(
        "{PLOTS}event north_read target kind plot, celsius min -40 max 60;
event south_read target kind plot, celsius min -40 max 60;
for plot as $p {{
    state $p_last = 0 min -40 max 60;
    state $p_spread_to = 0;
    event absorb_$p spread kind plot;
    on $p_read set $p_last = celsius;
    on absorb_$p set $p_spread_to = spread;
}};
"
    );
    assert_eq!(codes(&check(&source)), []);
}

#[test]
fn a_member_rule_is_allowed_by_code_or_by_name() {
    let source = plots(
        "    # caveat check: allow C003
    on read set $p_last = celsius;
    // caveat check: allow unrouted-member-rule
    on read when celsius < -30 set $p_last = -30;
",
    );
    let report = check(&source);
    assert_eq!(codes(&report), []);
    let (first, second) = (
        line_of(&source, "on read set"),
        line_of(&source, "on read when celsius < -30"),
    );
    assert_eq!(
        report
            .suppressed
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.line))
            .collect::<Vec<_>>(),
        [
            ("C003", first),
            ("C003", first),
            ("C003", second),
            ("C003", second)
        ]
    );
}

#[test]
fn a_define_from_another_block_over_the_same_kind_routes_the_rule() {
    // The define is written in one block and used in another, with another
    // binding. A block over another kind writes a define of the same shape.
    let source = format!(
        "{PLOTS}for plot as $p {{
    state $p_last = 0 min -40 max 60;
    define $p_here = target == $index;
}};
for probe as $d {{
    define $d_here = device == $index;
}};
for plot as $q {{
    on read when $q_here set $q_last = celsius;
    define $q_warm = $q_here and celsius > 2;
    on read when $q_warm set $q_last = 2;
}};
"
    );
    assert_eq!(codes(&check(&source)), []);
}

#[test]
fn a_define_outside_the_blocks_counts_when_it_names_a_member() {
    let source = format!(
        "{PLOTS}define north_watched = target == target.north;
define south_here = target == 2;
for plot as $p {{
    state $p_last = 0 min -40 max 60;
    on read when north_watched set $p_last = celsius;
    on read when south_here set $p_last = 0;
}};
"
    );
    // A member constant is a selection. A hand-written number is not.
    let line = line_of(&source, "on read when south_here");
    assert_eq!(codes(&check(&source)), [("C003", line), ("C003", line)]);
}

#[test]
fn a_rule_is_routed_by_any_parameter_of_the_kind() {
    let source = format!(
        "{PLOTS}event move from kind plot, to kind plot;
for plot as $p {{
    state $p_moves = 0;
    on move when to == $index set $p_moves = $p_moves + 1;
    on move when $index == from set $p_moves = $p_moves - 1;
    on move set $p_moves = 0;
}};
"
    );
    let report = check(&source);
    let line = line_of(&source, "on move set");
    assert_eq!(codes(&report), [("C003", line), ("C003", line)]);
    assert!(
        report.diagnostics[0]
            .message
            .ends_with("runs whichever plot `from` or `to` names: its guard does not select one"),
        "{}",
        report.diagnostics[0].message
    );
}

/// The plots with these rules in a `for` block, and events that name two or
/// three plots, or a plot and a probe.
fn moves(rules: &str) -> String {
    plots(rules).replace(
        "event tick",
        "event move from kind plot, to kind plot;
event relay from kind plot, via kind plot, to kind plot;
event lend from kind plot, device kind probe;
event tick",
    )
}

#[test]
fn a_disjunction_of_selections_routes_the_rule() {
    for rules in [
        // Either of two parameters of the kind, with or without parentheses.
        "on move when from == $index or to == $index set $p_last = 1;",
        "on move when (from == $index or to == $index) and $p_last < 9 set $p_last = 1;",
        "on move when $p_last < 9 and ($index == to or from == $index) set $p_last = 1;",
        // Three, however the chain is grouped.
        "on relay when from == $index or via == $index or to == $index set $p_last = 1;",
        "on relay when (from == $index or (via == $index)) or to == $index set $p_last = 1;",
        // Each disjunct in any form a selection takes.
        "on move when from == from.north or to == $p_slot or from == round($index) set $p_last = 1;",
        // Through a define: the disjunction as a conjunct, or one disjunct.
        "define $p_moved = from == $index or to == $index;\n    on move when $p_moved and $p_last < 9 set $p_last = 1;",
        "define $p_left = from == $index;\n    on move when $p_left or to == $index set $p_last = 1;",
        "define $p_left = from == $index;\n    define $p_moved = $p_left or to == $index;\n    on move when $p_moved set $p_last = 1;",
    ] {
        let source = moves(&format!("    {rules}\n"));
        assert_eq!(codes(&check(&source)), [], "{rules}");
    }
}

#[test]
fn a_disjunction_with_a_disjunct_that_does_not_select_is_reported() {
    for rules in [
        "on move when from == $index or $p_last > 0 set $p_last = 1;",
        "on move when (from == $index or to == 1) and $p_last < 9 set $p_last = 1;",
        "on move when from == $index or from != $index set $p_last = 1;",
        // A parameter of another kind.
        "on lend when from == $index or device == $index set $p_last = 1;",
        "on lend when from == $index or device == device.handheld set $p_last = 1;",
        // A disjunct that is a conjunction, and a disjunction under `not`.
        "on move when (from == $index and $p_last > 0) or to == $index set $p_last = 1;",
        "on move when not (from == $index or to == $index) set $p_last = 1;",
        // Through a define, as for a conjunct.
        "define $p_left = from == $index or $p_last > 0;\n    on move when $p_left or to == $index set $p_last = 1;",
    ] {
        let source = moves(&format!("    {rules}\n"));
        let line = line_of(&source, "on ");
        assert_eq!(
            codes(&check(&source)),
            [("C003", line), ("C003", line)],
            "{rules}"
        );
    }
}

#[test]
fn a_hold_that_presents_two_exhibits_selects_each_by_either_parameter() {
    // The shape of the hold in Before the Rain, a game written in Caveat
    // outside this repository, where C003 reported every rule on `hold` for
    // every exhibit.
    let source = r#"place yard kind farm;
entity farmhand kind suspect at yard;
entity x_farmer kind exhibit at yard;
entity x_prints kind exhibit at yard;
entity x_can kind exhibit at yard;
event hold suspect kind suspect, first kind exhibit, second kind exhibit;
for exhibit as $e {
    state $e_points = 1 min 0 max 3;
    state $e_staked = 0 min 0 max 1;
    on hold when (first == $index or second == $index) and $e_points == 0 reject "That exhibit is not established.";
    on hold when first == $index or second == $index set $e_staked = 1;
};
"#;
    assert_eq!(codes(&check(source)), []);
    // What it says: each copy runs only for an exhibit the hold presents.
    let mut session = ReactiveSession::from_source(source).expect("loads");
    let values = session
        .dispatch_json(
            "hold",
            r#"{"suspect":"farmhand","first":"x_can","second":"x_farmer"}"#,
        )
        .expect("hold is accepted")
        .values;
    assert_eq!(
        (
            values["x_farmer_staked"],
            values["x_prints_staked"],
            values["x_can_staked"]
        ),
        (1.0, 0.0, 1.0)
    );
    // A disjunct on the suspect, another kind, is no selection of an exhibit.
    let suspect = source.replace(
        "on hold when first == $index or second == $index set",
        "on hold when first == $index or suspect == $index set",
    );
    let line = line_of(&suspect, "on hold when first");
    let report = check(&suspect);
    assert_eq!(
        codes(&report),
        [("C003", line), ("C003", line), ("C003", line)]
    );
    assert_eq!(
        report.diagnostics[0].message,
        "the rule on `hold` for `x_farmer` runs whichever exhibit `first` or `second` names: its guard does not select one"
    );
}

#[test]
fn a_kind_with_one_member_warns_once() {
    let source = "place field kind field;
entity north kind plot at field;
event read target kind plot, celsius min -40 max 60;
for plot as $p {
    state $p_last = 0 min -40 max 60;
    on read set $p_last = celsius;
};
";
    assert_eq!(
        codes(&check(source)),
        [("C003", line_of(source, "on read set"))]
    );
}

#[test]
fn every_member_the_loader_declares_is_checked() {
    // A comment in an entity statement, and a last one without its `;`, do
    // not hide a member from the block (#47).
    let source = "place field kind field;
entity north # the first plot
    kind plot at field;
event read target kind plot, celsius min -40 max 60;
for plot as $p {
    state $p_last = 0 min -40 max 60;
    on read set $p_last = celsius;
};
entity south kind plot at field
";
    let report = check(source);
    let line = line_of(source, "on read set");
    assert_eq!(codes(&report), [("C003", line), ("C003", line)]);
    assert!(report.diagnostics[0].message.contains("for `north`"));
    assert!(report.diagnostics[1].message.contains("for `south`"));
}

#[test]
fn a_brace_outside_a_procedure_or_block_does_not_hide_a_member_from_the_check() {
    // The loader ends this evidence, with unquoted provenance, at its `;`,
    // so it declares all three plots.
    let source = "place field kind field;
entity east kind plot at field;
event read target kind plot, celsius min -40 max 60;
for plot as $p {
    state $p_last = 0 min -40 max 60;
    on read set $p_last = celsius;
};
evidence manual from see{appendix;
entity north kind plot at field;
entity south kind plot at field;
";
    let report = check(source);
    let line = line_of(source, "on read set");
    assert_eq!(
        codes(&report),
        [("C003", line), ("C003", line), ("C003", line)]
    );
    for (diagnostic, member) in report.diagnostics.iter().zip(["east", "north", "south"]) {
        assert!(
            diagnostic.message.contains(&format!("for `{member}`")),
            "{}",
            diagnostic.message
        );
    }
}

#[test]
fn a_first_word_that_only_begins_with_for_does_not_hide_a_member_from_the_check() {
    // `for{ supports c;` is a relation about an evidence named `for{`, not
    // a block, so the loader declares all three plots.
    let source = "place field kind field;
entity east kind plot at field;
event read target kind plot, celsius min -40 max 60;
for plot as $p {
    state $p_last = 0 min -40 max 60;
    on read set $p_last = celsius;
};
claim c;
evidence for{ from log;
for{ supports c;
entity north kind plot at field;
entity south kind plot at field;
";
    let report = check(source);
    let line = line_of(source, "on read set");
    assert_eq!(
        codes(&report),
        [("C003", line), ("C003", line), ("C003", line)]
    );
    for (diagnostic, member) in report.diagnostics.iter().zip(["east", "north", "south"]) {
        assert!(
            diagnostic.message.contains(&format!("for `{member}`")),
            "{}",
            diagnostic.message
        );
    }
}

#[test]
fn a_block_with_a_comment_in_its_header_is_checked() {
    // Comments count as whitespace, in a header too.
    let source = "place field kind field;
entity north kind plot at field;
entity south kind plot at field;
event read target kind plot, celsius min -40 max 60;
for plot # each plot
    as $p {
    state $p_last = 0 min -40 max 60;
    on read set $p_last = celsius;
};
";
    let report = check(source);
    let line = line_of(source, "on read set");
    assert_eq!(codes(&report), [("C003", line), ("C003", line)]);
}

#[test]
fn a_last_block_without_its_semicolon_is_checked() {
    let source = "place field kind field;
entity north kind plot at field;
entity south kind plot at field;
event read target kind plot, celsius min -40 max 60;
for plot as $p {
    state $p_last = 0 min -40 max 60;
    on read set $p_last = celsius;
}
";
    let report = check(source);
    let line = line_of(source, "on read set");
    assert_eq!(codes(&report), [("C003", line), ("C003", line)]);
}

#[test]
fn a_brace_pair_in_a_body_statement_is_checked() {
    // A `}` that closes a `{` earlier in its statement does not close the
    // body; the loader reads both as the provenance's text.
    let source = "place field kind field;
entity north kind plot at field;
entity south kind plot at field;
event read target kind plot, celsius min -40 max 60;
for plot as $p {
    evidence $p_manual from see{appendix}b;
    state $p_last = 0 min -40 max 60;
    on read set $p_last = celsius;
};
";
    let report = check(source);
    let line = line_of(source, "on read set");
    assert_eq!(codes(&report), [("C003", line), ("C003", line)]);
}

#[test]
fn a_last_block_after_an_empty_statement_is_checked() {
    // The loader accepts empty statements, and begins a last statement
    // without its `;` at its first word.
    let source = "place field kind field;
entity north kind plot at field;
entity south kind plot at field;
event read target kind plot, celsius min -40 max 60;;
for plot as $p {
    state $p_last = 0 min -40 max 60;
    on read set $p_last = celsius;
}
";
    let report = check(source);
    let line = line_of(source, "on read set");
    assert_eq!(codes(&report), [("C003", line), ("C003", line)]);
}

#[test]
fn a_rule_that_reads_only_once_expanded_is_skipped_not_an_error() {
    // Written, `"\$p"` holds an escape the scanner refuses. Expanded, it is
    // `"\north"` and `"\tarn"`, which load.
    let source = "place field kind field;
entity north kind plot at field;
entity tarn kind plot at field;
event read target kind plot;
for plot as $p {
    on read reject \"\\$p\";
};
";
    let report = check_source(source).expect("a program that loads is checked");
    assert_eq!(codes(&report), []);
}

// ── Routed blocks (spec/caveat-routed-repetition-0.1.md section 8) ────────

/// The plots with these rules in a block routed by `target`.
fn routed_plots(rules: &str) -> String {
    plots(rules).replace("for plot as $p {", "for plot as $p routed by target {")
}

/// Each line, twice: a C003 warning for each plot.
fn for_both(lines: &[usize]) -> Vec<(&'static str, usize)> {
    lines
        .iter()
        .flat_map(|line| [("C003", *line), ("C003", *line)])
        .collect()
}

#[test]
fn a_rule_in_a_routed_block_is_not_checked() {
    // In a plain block, C003 reports each of these.
    let rules = "    on read set $p_last = celsius;
    on read when celsius > 2 set $p_last = celsius;
    on read when target == 1 set $p_last = 0;
    on read when celsius > 59 reject \"$p is flooded\";
";
    let report = check(&routed_plots(rules));
    assert_eq!(codes(&report), []);
    assert!(report.suppressed.is_empty());

    // Dropping `routed by target` makes the block plain, and C003 reports
    // each rule that has no selection it recognizes.
    let plain = plots(rules);
    assert_eq!(
        codes(&check(&plain)),
        for_both(&[
            line_of(&plain, "on read set"),
            line_of(&plain, "on read when celsius > 2"),
            line_of(&plain, "on read when target == 1"),
            line_of(&plain, "on read when celsius > 59"),
        ])
    );
}

#[test]
fn dropping_routed_by_changes_two_kinds_of_rule_with_no_report() {
    // A conjunct that C003 counts as a selection but that is not
    // `target == $index` alone, and a rule C003 cannot read as written.
    // Routed, each copy runs only for its own plot. Plain, none is reported.
    let rules = "    caveat $p_fog consequence low;
    on read when target == target.north set $p_last = celsius;
    on pass when target == $index or from == $index set $p_last = 5;
    on read when celsius < -30 examine $p_fog cost $index;
";
    let pass = |source: String| {
        format!("budget 4;\n{source}").replace(
            "event tick",
            "event pass target kind plot, from kind plot;\nevent tick",
        )
    };
    let (routed, plain) = (pass(routed_plots(rules)), pass(plots(rules)));
    for source in [&routed, &plain] {
        assert_eq!(codes(&check(source)), []);
    }
    // A pass from south to north: routed, only north's copy runs; plain,
    // south's does too, through `from`.
    let passed = |source: &str| {
        let mut session = ReactiveSession::from_source(source).expect("loads");
        let values = session
            .dispatch_json("pass", r#"{"target":"north","from":"south"}"#)
            .expect("pass is accepted")
            .values;
        (values["north_last"], values["south_last"])
    };
    assert_eq!(passed(&routed), (5.0, 0.0));
    assert_eq!(passed(&plain), (5.0, 5.0));
}

#[test]
fn a_plain_block_beside_a_routed_one_over_the_same_kind_is_still_checked() {
    let source = format!(
        "{PLOTS}for plot as $p routed by target {{
    state $p_last = 0 min -40 max 60;
    state $p_count = 0;
    define $p_here = target == $index;
    on read set $p_last = celsius;
}};
for plot as $q {{
    on read set $q_count = 0;
    on read when $q_here set $q_count = $q_count + 1;
    # caveat check: allow unrouted-member-rule
    on read set $q_last = 0;
}};
"
    );
    let report = check(&source);
    // The define written in the routed block routes the plain block's rule.
    assert_eq!(
        codes(&report),
        for_both(&[line_of(&source, "on read set $q_count")])
    );
    assert_eq!(
        report
            .suppressed
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.line))
            .collect::<Vec<_>>(),
        for_both(&[line_of(&source, "on read set $q_last")])
    );
}

/// A routed block that commits each plot's cover on its readings.
fn covers(rules: &str) -> String {
    format!(
        "{PLOTS}for plot as $p routed by target {{
    claim $p_frost;
    evidence $p_probe from \"a probe in $p\";
    readings $p_readings from $p_probe limit 4;
    decisions $p_cover limit 2;
    on read sample $p_readings = celsius supports $p_frost;
    on read when celsius < 0 commit $p_cover because enough using latest($p_readings);
{rules}}};
"
    )
}

#[test]
fn c002_reads_a_routed_blocks_rules_expanded_and_places_them_as_written() {
    // The rules are read with their routes, and a route reads no stream.
    let source = covers("");
    let line = line_of(&source, "decisions $p_cover");
    assert_eq!(codes(&check(&source)), [("C002", line), ("C002", line)]);
    // A reopening rule in the block is a reopening path, routed or not.
    for rule in [
        "    on read when celsius > 5 reopen $p_cover because latest($p_readings);\n",
        "    on swap reopen $p_cover because latest($p_readings);\n",
    ] {
        assert_eq!(codes(&check(&covers(rule))), [], "{rule}");
    }
}

#[test]
fn c001_does_not_compare_the_rules_in_a_routed_block() {
    let source = format!(
        "{PLOTS}event reread target kind plot, celsius min -40 max 60;
for plot as $p routed by target {{
    state $p_last = 0 min -40 max 60;
    on read when celsius > 2 set $p_last = celsius;
    on read when celsius <= 2 set $p_last = 0;
    on reread when celsius > 2 set $p_last = celsius;
    on reread when celsius <= 2 set $p_last = 0;
}};
"
    );
    assert_eq!(codes(&check(&source)), []);
}

#[test]
fn a_rule_for_every_member_moves_to_a_plain_block_with_its_allow_comment() {
    // The example in section 6 of the routed repetition spec.
    let source = r#"place trail kind trail;
entity north kind tunnel at trail;
entity south kind tunnel at trail;
event observe target kind tunnel, method in report scout;

for tunnel as $t routed by target {
    state $t_reported = 0 min 0 max 1;
    state $t_support = 0;
    on observe when method == method.report and $t_reported == 1 reject "That visitor has already reported.";
    on observe when method == method.report set $t_reported = 1;
};

for tunnel as $t {
    # caveat check: allow unrouted-member-rule
    on observe set $t_support = 0;
};
"#;
    let report = check(source);
    assert_eq!(codes(&report), []);
    let line = line_of(source, "on observe set $t_support");
    assert_eq!(
        report
            .suppressed
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.line))
            .collect::<Vec<_>>(),
        [("C003", line), ("C003", line)]
    );
    // Without the comment, the moved rule is reported.
    let unallowed = source.replace("    # caveat check: allow unrouted-member-rule\n", "");
    let line = line_of(&unallowed, "on observe set $t_support");
    assert_eq!(codes(&check(&unallowed)), [("C003", line), ("C003", line)]);
}

// ── C004: a member's `$index` and its number in a `kind` parameter ────────

// The pieces of the programs from the evaluation behind C004. `read` numbers
// every plot the loaded program declares, north too when a zone block
// declares it; `$index` numbers the top-level plots only.
const FIELD: &str = "place field kind field;\n";
const EAST: &str = "entity east kind plot at field;\n";
const SOUTH: &str = "entity south kind plot at field;\n";
const ZONE: &str = "entity z kind zone at field;\n";
const READ: &str = "event read target kind plot;\n";
/// A zone block that declares north, and one that also counts north's reads
/// by name.
const ZONE_BARE: &str = "for zone as $z {\n    entity north kind plot at field;\n};\n";
const ZONE_OWN: &str = "for zone as $z {
    entity north kind plot at field;
    state north_n = 0;
    on read when target == target.north set north_n = north_n + 1;
};
";
/// A plot block that selects its member by `$index`, and one by name.
const BY_INDEX: &str = "for plot as $p {
    state $p_n = 0;
    on read when target == $index set $p_n = $p_n + 1;
};
";
const BY_NAME: &str = "for plot as $p {
    state $p_n = 0;
    on read when target == target.$p set $p_n = $p_n + 1;
};
";

/// A plot block that declares a twin plot for each member, and counts the
/// reads of both, its member's selected by `selection`.
fn twins(selection: &str) -> String {
    format!(
        "for plot as $p {{
    entity $p_twin kind plot at field;
    state $p_n = 0;
    state $p_twin_n = 0;
    on read when {selection} set $p_n = $p_n + 1;
    on read when target == target.$p_twin set $p_twin_n = $p_twin_n + 1;
}};
"
    )
}

fn program(parts: &[&str]) -> String {
    format!("{FIELD}{}", parts.concat())
}

/// The programs in which a plot block selects its member by `$index` and
/// `read` counts a plot that a for block declares before a top-level plot,
/// each with the members whose copy acts for another plot.
fn shifted() -> Vec<(&'static str, String, Vec<&'static str>)> {
    vec![
        (
            "i1",
            program(&[EAST, ZONE, ZONE_BARE, SOUTH, READ, BY_INDEX]),
            vec!["south"],
        ),
        (
            "i2",
            program(&[ZONE, ZONE_BARE, EAST, SOUTH, READ, BY_INDEX]),
            vec!["east", "south"],
        ),
        (
            "i5",
            program(&[EAST, READ, BY_INDEX, ZONE, ZONE_BARE, SOUTH]),
            vec!["south"],
        ),
        (
            "c2",
            program(&[EAST, ZONE, READ, ZONE_OWN, SOUTH, BY_INDEX]),
            vec!["south"],
        ),
        (
            "a5c",
            program(&[READ, &twins("target == $index"), EAST, SOUTH]),
            vec!["east", "south"],
        ),
        (
            "a6b",
            program(&[ZONE, READ, ZONE_OWN, EAST, BY_INDEX]),
            vec!["east"],
        ),
    ]
}

#[test]
fn a_member_index_the_event_numbers_otherwise_is_reported_for_each_member() {
    for (name, source, members) in shifted() {
        let report = check(&source);
        let line = line_of(&source, "on read when target == $index");
        assert_eq!(
            codes(&report),
            vec![("C004", line); members.len()],
            "{name}\n{source}"
        );
        for (diagnostic, member) in report.diagnostics.iter().zip(&members) {
            assert_eq!(
                (diagnostic.name, diagnostic.severity, diagnostic.column),
                ("shifted-member-index", "warning", 5)
            );
            assert!(
                diagnostic.message.starts_with(&format!(
                    "the rule on `read` for `{member}` runs when `target` names `"
                )),
                "{name}: {}",
                diagnostic.message
            );
        }
    }
}

#[test]
fn the_warning_says_which_member_the_copy_acts_for_and_why() {
    let source = program(&[EAST, ZONE, ZONE_BARE, SOUTH, READ, BY_INDEX]);
    let report = check(&source);
    let [south] = report.diagnostics.as_slice() else {
        panic!("{:?}", codes(&report));
    };
    assert_eq!(
        south.message,
        "the rule on `read` for `south` runs when `target` names `north`, not `south`: `$index` is 2 in its copy, and `target` numbers `south` 3, because it also counts `north`, declared in a for block"
    );
    assert_eq!(
        south.suggestion,
        "If the rule is about the plot the event names, select it by name, such as `target == target.$p`, which names its own plot however the entities are counted. If comparing with `$index` is intended, put `# caveat check: allow shifted-member-index` on the line above it."
    );
    let north = line_of(&source, "entity north");
    assert_eq!(
        south
            .related
            .iter()
            .map(|related| (related.line, related.column, related.note.as_str()))
            .collect::<Vec<_>>(),
        [(north, 5, "where a for block declares `north`")]
    );
    // What it describes: `read north` counts for south, and `read south`
    // for no plot.
    let mut session = ReactiveSession::from_source(&source).expect("loads");
    let mut read = |target: &str| {
        let values = session
            .dispatch_json("read", &format!(r#"{{"target":"{target}"}}"#))
            .expect("read is accepted")
            .values;
        (values["east_n"], values["south_n"])
    };
    assert_eq!(read("north"), (0.0, 1.0));
    assert_eq!(read("south"), (0.0, 1.0));
    assert_eq!(read("east"), (1.0, 1.0));

    // Several plots that a block declares, each counted.
    let source = program(&[READ, &twins("target == $index"), EAST, SOUTH]);
    let report = check(&source);
    assert_eq!(
        report.diagnostics[1].message,
        "the rule on `read` for `south` runs when `target` names `south_twin`, not `south`: `$index` is 2 in its copy, and `target` numbers `south` 4, because it also counts `east_twin` and `south_twin`, each declared in a for block"
    );
    let twin = line_of(&source, "entity $p_twin");
    assert_eq!(
        report.diagnostics[1]
            .related
            .iter()
            .map(|related| (related.line, related.note.as_str()))
            .collect::<Vec<_>>(),
        [
            (twin, "where a for block declares `east_twin`"),
            (twin, "where a for block declares `south_twin`")
        ]
    );
}

#[test]
fn nothing_is_reported_where_every_member_is_numbered_alike_or_selected_by_name() {
    let value_index = "for plot as $p {
    state $p_n = 0;
    on read when target == target.$p set $p_n = $p_n + $index;
};
";
    let ping = "for plot as $p {
    state $p_ticks = 0;
    on ping set $p_ticks = $p_ticks + 1;
};
";
    let zones_by_name = "for zone as $z {
    entity $z_plot kind plot at field;
    state $z_plot_n = 0;
    on read when target == target.$z_plot set $z_plot_n = $z_plot_n + 1;
};
";
    let marker = "for zone as $z {\n    entity $z_marker kind marker at field;\n};\n";
    let labelled = "for plot as $p {
    state $p_n = 0;
    on read when target == target.$p set $p_n = $p_n + 1;
    bind $p.label.text = \"plot $index\";
};
";
    let routed_zone = "for zone as $z routed by target {
    entity $z_plot kind plot at field;
    state $z_visits = 0;
    on visit set $z_visits = $z_visits + 1;
};
";
    let no_event = "for plot as $p {\n    state $p_seen = 0;\n};\n";
    let two_zones = "entity zone_a kind zone at field;\nentity zone_b kind zone at field;\n";
    let zone_a = "entity zone_a kind zone at field;\nevent visit target kind zone;\n";
    for (name, source) in [
        // Every plot that a block declares comes after the top-level plots.
        (
            "c1",
            program(&[EAST, SOUTH, ZONE, READ, ZONE_OWN, BY_INDEX]),
        ),
        ("a6", program(&[EAST, ZONE, READ, ZONE_OWN, BY_INDEX])),
        (
            "a5b",
            program(&[EAST, SOUTH, READ, &twins("target == $index")]),
        ),
        // The block declares an entity of another kind.
        ("a4", program(&[EAST, ZONE, READ, marker, SOUTH, BY_INDEX])),
        // Selected by name, wherever north is declared.
        ("c3", program(&[EAST, ZONE, READ, ZONE_OWN, SOUTH, BY_NAME])),
        (
            "c3a",
            program(&[ZONE, READ, ZONE_OWN, EAST, SOUTH, BY_NAME]),
        ),
        (
            "c3b",
            program(&[ZONE, READ, EAST, SOUTH, BY_NAME, ZONE_OWN]),
        ),
        (
            "a1",
            program(&[EAST, ZONE, READ, ZONE_OWN, SOUTH, value_index]),
        ),
        (
            "a3",
            program(&[EAST, two_zones, READ, zones_by_name, SOUTH, BY_NAME]),
        ),
        (
            "a5",
            program(&[EAST, SOUTH, READ, &twins("target == target.$p")]),
        ),
        // No rule on an event that names a plot.
        (
            "c5",
            program(&[EAST, ZONE, ZONE_BARE, SOUTH, READ, no_event]),
        ),
        (
            "a2",
            program(&[EAST, ZONE, READ, "event ping;\n", ZONE_BARE, SOUTH, ping]),
        ),
        // North gets no copy of the plot block, so `read north` changes
        // nothing there, and nothing reports it (spec/caveat-check-0.1.md,
        // C004).
        (
            "i3",
            program(&[EAST, SOUTH, ZONE, ZONE_BARE, READ, BY_INDEX]),
        ),
        (
            "i4",
            program(&[EAST, SOUTH, ZONE, READ, BY_INDEX, ZONE_BARE]),
        ),
        (
            "c4",
            program(&[EAST, ZONE, ZONE_BARE, SOUTH, READ, BY_NAME]),
        ),
        (
            "a7",
            program(&[EAST, ZONE, READ, ZONE_BARE, SOUTH, labelled]),
        ),
        (
            "a8",
            program(&[EAST, zone_a, READ, routed_zone, SOUTH, BY_NAME]),
        ),
    ] {
        let report = check(&source);
        assert_eq!(codes(&report), [], "{name}\n{source}");
        assert!(report.suppressed.is_empty(), "{name}");
    }
}

#[test]
fn every_selection_by_index_that_c003_reads_is_checked() {
    // North, which a zone block declares, comes before both top-level
    // plots, so each plot's copy acts for another plot.
    let north_first = |rules: &str| {
        program(&[
            ZONE,
            ZONE_BARE,
            EAST,
            SOUTH,
            "event move from kind plot, to kind plot;\n",
            READ,
            &format!("for plot as $p {{\n    state $p_n = 0;\n{rules}}};\n"),
        ])
    };
    let rule_line = |source: &str| line_of(source, "on ");
    for rule in [
        "on read when $index == target set $p_n = 1;",
        "on read when target == $index + 0 set $p_n = 1;",
        "on read when $p_n < 9 and (target == $index and $p_n >= 0) set $p_n = 1;",
        "define $p_here = target == $index;\n    on read when $p_here set $p_n = 1;",
        "on move when to == $index set $p_n = 1;",
        // A disjunction of selections, where only `to` reads `$index`.
        "on move when to == $index or from == from.east set $p_n = 1;",
        // On the member's own event, which C003 does not check.
        "event $p_read target kind plot;\n    on $p_read when target == $index set $p_n = 1;",
    ] {
        let source = north_first(&format!("    {rule}\n"));
        let line = rule_line(&source);
        assert_eq!(
            codes(&check(&source)),
            [("C004", line), ("C004", line)],
            "{rule}"
        );
    }
    let report = check(&north_first(
        "    on move when to == $index set $p_n = 1;\n",
    ));
    assert!(
        report.diagnostics[0]
            .message
            .starts_with("the rule on `move` for `east` runs when `to` names `north`, not `east`"),
        "{}",
        report.diagnostics[0].message
    );
    // A comparison in a disjunction that does not select is C003's to report,
    // unless another conjunct selects the member: then neither check reports
    // it. A selection by name is not reported.
    for (rule, code) in [
        (
            "on read when target == $index or $p_n > 5 set $p_n = 1;",
            Some("C003"),
        ),
        (
            "on move when to == to.$p and (from == $index or $p_n > 5) set $p_n = 1;",
            None,
        ),
        ("on read when target == target.$p set $p_n = 1;", None),
        ("on read when target == target.east set $p_n = 1;", None),
    ] {
        let source = north_first(&format!("    {rule}\n"));
        let line = rule_line(&source);
        let expected = code
            .map(|code| vec![(code, line), (code, line)])
            .unwrap_or_default();
        assert_eq!(codes(&check(&source)), expected, "{rule}");
    }
}

#[test]
fn each_disjunct_that_selects_by_index_is_checked() {
    // North, which a zone block declares, comes before both top-level plots.
    // Each disjunct is a way a copy runs, so each is reported.
    let north_first = |rule: &str| {
        program(&[
            ZONE,
            ZONE_BARE,
            EAST,
            SOUTH,
            "event move from kind plot, to kind plot;\n",
            &format!("for plot as $p {{\n    state $p_n = 0;\n    {rule}\n}};\n"),
        ])
    };
    for (rule, order) in [
        (
            "on move when from == $index or to == $index set $p_n = $p_n + 1;",
            ["from", "to"],
        ),
        (
            "on move when $p_n < 9 and (to == $index or $index == from) set $p_n = $p_n + 1;",
            ["to", "from"],
        ),
        (
            "define $p_moved = from == $index or to == $index;\n    on move when $p_moved set $p_n = $p_n + 1;",
            ["from", "to"],
        ),
    ] {
        let source = north_first(rule);
        let report = check(&source);
        let line = line_of(&source, "on move");
        assert_eq!(codes(&report), vec![("C004", line); 4], "{rule}");
        // East's copy runs when either names north, and south's when either
        // names east.
        let expected = [("east", "north"), ("south", "east")]
            .into_iter()
            .flat_map(|(member, acts_for)| {
                order.map(|parameter| {
                    format!("the rule on `move` for `{member}` runs when `{parameter}` names `{acts_for}`, not `{member}`: ")
                })
            });
        for (diagnostic, prefix) in report.diagnostics.iter().zip(expected) {
            assert!(
                diagnostic.message.starts_with(&prefix),
                "{rule}: {}",
                diagnostic.message
            );
        }
        // What they describe: a move from north counts for east, and a move
        // to east for south.
        let moved = |from: &str, to: &str| {
            let mut session = ReactiveSession::from_source(&source).expect("loads");
            let values = session
                .dispatch_json("move", &format!(r#"{{"from":"{from}","to":"{to}"}}"#))
                .expect("move is accepted")
                .values;
            (values["east_n"], values["south_n"])
        };
        assert_eq!(moved("north", "south"), (1.0, 0.0), "{rule}");
        assert_eq!(moved("south", "east"), (0.0, 1.0), "{rule}");
    }
}

/// A plot block with this rule, after a zone block that declares north before
/// both top-level plots, so that `from`, `via` and `to` number east 2 and
/// south 3.
fn relays(rule: &str) -> String {
    program(&[
        ZONE,
        ZONE_BARE,
        EAST,
        SOUTH,
        "state seen = 0;\n",
        "event move from kind plot, to kind plot;\n",
        "event relay from kind plot, via kind plot, to kind plot;\n",
        &format!("for plot as $p {{\n    state $p_n = 0;\n    {rule}\n}};\n"),
    ])
}

/// Each C004 at the rule, in order, as `MEMBER: P names ENTITY`: the member
/// whose copy it reports, the parameter, and the entity the copy runs for.
fn shifted_at_rule(source: &str) -> Vec<String> {
    let report = check(source);
    let line = line_of(source, "on ");
    assert_eq!(
        codes(&report),
        vec![("C004", line); report.diagnostics.len()],
        "{source}"
    );
    report
        .diagnostics
        .iter()
        .map(|diagnostic| {
            // "the rule on `EVENT` for `MEMBER` runs when `P` names `ENTITY`, ..."
            let quoted = diagnostic.message.split('`').collect::<Vec<_>>();
            format!("{}: {} names {}", quoted[3], quoted[5], quoted[7])
        })
        .collect()
}

#[test]
fn every_conjunct_that_selects_by_index_is_checked_not_only_the_first() {
    for (rule, reported) in [
        // A chain of `or` first, whose comparisons do not shift, does not hide
        // a later conjunct that does.
        (
            "on move when (from == $index + 1 or from == from.east) and to == $index set $p_n = $p_n + 1;",
            vec!["east: to names north", "south: to names east"],
        ),
        (
            "on move when (from == $index + 1 or to == $index + 1) and to == $index set $p_n = $p_n + 1;",
            vec!["east: to names north", "south: to names east"],
        ),
        (
            "on relay when (from == from.east or to == $index + 1) and via == $index set $p_n = $p_n + 1;",
            vec!["east: via names north", "south: via names east"],
        ),
        // Nor does one whose number reads a state, and is not worked out.
        (
            "on move when (from == $index + seen or to == to.east) and to == $index set $p_n = $p_n + 1;",
            vec!["east: to names north", "south: to names east"],
        ),
        // A conjunct first that does not shift does not hide a later one, a
        // comparison or a chain of `or`.
        (
            "on relay when via == $index + 1 and from == $index set $p_n = $p_n + 1;",
            vec!["east: from names north", "south: from names east"],
        ),
        (
            "on relay when via == $index + 1 and (from == $index or to == $index) set $p_n = $p_n + 1;",
            vec![
                "east: from names north",
                "east: to names north",
                "south: from names east",
                "south: to names east",
            ],
        ),
        // Two conjuncts that shift are both reported.
        (
            "on move when from == $index and to == $index set $p_n = $p_n + 1;",
            vec![
                "east: from names north",
                "east: to names north",
                "south: from names east",
                "south: to names east",
            ],
        ),
    ] {
        assert_eq!(shifted_at_rule(&relays(rule)), reported, "{rule}");
    }
    // What the later conjunct does: east's copy runs on a move to north, and
    // on a relay from north.
    let counted = |rule: &str, event: &str, parameters: &str| {
        let mut session = ReactiveSession::from_source(&relays(rule)).expect("loads");
        let values = session
            .dispatch_json(event, parameters)
            .expect("the event is accepted")
            .values;
        (values["east_n"], values["south_n"])
    };
    assert_eq!(
        counted(
            "on move when (from == $index + 1 or from == from.east) and to == $index set $p_n = $p_n + 1;",
            "move",
            r#"{"from":"east","to":"north"}"#
        ),
        (1.0, 0.0)
    );
    assert_eq!(
        counted(
            "on relay when via == $index + 1 and (from == $index or to == $index) set $p_n = $p_n + 1;",
            "relay",
            r#"{"from":"north","via":"east","to":"south"}"#
        ),
        (1.0, 0.0)
    );
}

#[test]
fn the_order_of_the_conjuncts_does_not_change_what_is_reported() {
    // South's copy compares `from` with 1, the `$index` of east, which `from`
    // gives north. `$index + 1` comes to each plot's own number.
    let (either, via) = (
        "(from == $index - 1 or from == $index + 1)",
        "via == $index",
    );
    for guard in [format!("{either} and {via}"), format!("{via} and {either}")] {
        let mut reported = shifted_at_rule(&relays(&format!(
            "on relay when {guard} set $p_n = $p_n + 1;"
        )));
        reported.sort();
        assert_eq!(
            reported,
            [
                "east: via names north",
                "south: from names north",
                "south: via names east"
            ],
            "{guard}"
        );
    }
}

#[test]
fn the_same_comparison_twice_is_reported_once() {
    for rule in [
        "on move when to == $index or to == $index set $p_n = 1;",
        "on move when to == $index and $index == to set $p_n = 1;",
        "on move when (to == $index or to == $index + 0) and to == round($index) set $p_n = 1;",
    ] {
        assert_eq!(
            shifted_at_rule(&relays(rule)),
            ["east: to names north", "south: to names east"],
            "{rule}"
        );
    }
}

#[test]
fn a_chain_of_selections_is_checked_where_c003_does_not_need_it() {
    // C003 reports none of these: another conjunct selects the member by
    // name, the rule is on the member's own event, or C003 is allowed. C004
    // still checks each comparison with `$index` in the chain. Before the
    // chain was read, each of them checked clean.
    for rule in [
        "on move when to == to.$p and (from == $index or to == $index) set $p_n = $p_n + 1;",
        "define $p_moved = from == $index or to == $index;\n    on move when to == to.$p and $p_moved set $p_n = $p_n + 1;",
        "event $p_move from kind plot, to kind plot;\n    on $p_move when from == $index or to == $index set $p_n = $p_n + 1;",
        "# caveat check: allow unrouted-member-rule\n    on move when from == $index or to == $index set $p_n = $p_n + 1;",
    ] {
        let source = relays(rule);
        assert_eq!(
            shifted_at_rule(&source),
            [
                "east: from names north",
                "east: to names north",
                "south: from names east",
                "south: to names east",
            ],
            "{rule}"
        );
        assert!(check(&source).suppressed.is_empty(), "{rule}");
    }
    // What they describe: a move from south to east does not count for
    // east, whose copy compares with north.
    let moved = |rule: &str, event: &str, from: &str, to: &str| {
        let mut session = ReactiveSession::from_source(&relays(rule)).expect("loads");
        let values = session
            .dispatch_json(event, &format!(r#"{{"from":"{from}","to":"{to}"}}"#))
            .expect("the move is accepted")
            .values;
        (values["east_n"], values["south_n"])
    };
    let by_name =
        "on move when to == to.$p and (from == $index or to == $index) set $p_n = $p_n + 1;";
    assert_eq!(moved(by_name, "move", "south", "east"), (0.0, 0.0));
    assert_eq!(moved(by_name, "move", "north", "east"), (1.0, 0.0));
    let own = "event $p_move from kind plot, to kind plot;\n    on $p_move when from == $index or to == $index set $p_n = $p_n + 1;";
    assert_eq!(moved(own, "east_move", "south", "east"), (0.0, 0.0));
    assert_eq!(moved(own, "east_move", "north", "south"), (1.0, 0.0));
}

#[test]
fn a_shifted_member_index_is_allowed_by_code_or_by_name() {
    for comment in [
        "# caveat check: allow shifted-member-index",
        "// caveat check: allow C004",
    ] {
        let source = program(&[ZONE, ZONE_BARE, EAST, SOUTH, READ, BY_INDEX]).replace(
            "    on read when target == $index",
            &format!("    {comment}\n    on read when target == $index"),
        );
        let report = check(&source);
        assert_eq!(codes(&report), []);
        let line = line_of(&source, "on read when target == $index");
        assert_eq!(
            report
                .suppressed
                .iter()
                .map(|diagnostic| (diagnostic.code, diagnostic.line))
                .collect::<Vec<_>>(),
            [("C004", line), ("C004", line)]
        );
    }
}

#[test]
fn a_routed_block_still_refuses_an_entity_of_its_kind_in_a_for_block() {
    // Routed Repetition 0.1 section 7, unchanged: such a program does not
    // load, so there is nothing to check.
    let zones = "entity zone_a kind zone at field;\nentity zone_b kind zone at field;\n";
    let zone_plots = "for zone as $z {\n    entity $z_plot kind plot at field;\n};\n";
    for (before, entity) in [
        (program(&[EAST, SOUTH, ZONE, READ, ZONE_OWN]), "north"),
        (program(&[EAST, ZONE, READ, ZONE_OWN, SOUTH]), "north"),
        (program(&[EAST, zones, READ, zone_plots, SOUTH]), "$z_plot"),
    ] {
        let source = format!(
            "{before}for plot as $p routed by target {{\n    state $p_n = 0;\n    on read set $p_n = $p_n + 1;\n}};\n"
        );
        assert_eq!(
            check_source(&source).unwrap_err(),
            format!("`for plot as $p routed by target`: `$index` does not count entity `{entity}` of kind plot, declared in a for block, but `target` does; declare it at the top level of this part")
        );
    }
}

/// Which of `plots` count `read target`, sent once to a fresh session.
fn counted_on_read(source: &str, target: &str, plots: &[&str]) -> Vec<String> {
    let mut session = ReactiveSession::from_source(source).expect("loads");
    let values = session
        .dispatch_json("read", &format!(r#"{{"target":"{target}"}}"#))
        .expect("read is accepted")
        .values;
    plots
        .iter()
        .filter(|plot| values[format!("{plot}_n").as_str()] != 0.0)
        .map(|plot| plot.to_string())
        .collect()
}

#[test]
fn an_index_adjusted_on_purpose_is_read_as_the_number_it_comes_to() {
    let plus_one = "for plot as $p {
    state $p_n = 0;
    on read when target == $index + 1 set $p_n = $p_n + 1;
};
";
    let plots = ["east", "south"];
    // North comes first, so `$index + 1` is each plot's own number in
    // `read`: every copy acts for its own plot, and nothing is reported.
    let compensated = program(&[ZONE, ZONE_BARE, EAST, SOUTH, READ, plus_one]);
    assert_eq!(codes(&check(&compensated)), []);
    for (target, counted) in [
        ("north", &[][..]),
        ("east", &["east"][..]),
        ("south", &["south"][..]),
    ] {
        assert_eq!(
            counted_on_read(&compensated, target, &plots),
            counted,
            "read {target}"
        );
    }

    // North between them: east's copy compares `target` with 2, south's
    // `$index`, and `read` numbers north 2. South's copy compares it with
    // 3, south's own number there, so only east's is reported.
    let between = program(&[EAST, ZONE, ZONE_BARE, SOUTH, READ, plus_one]);
    let report = check(&between);
    assert_eq!(codes(&report), [("C004", line_of(&between, "on read"))]);
    assert_eq!(
        report.diagnostics[0].message,
        "the rule on `read` for `east` runs when `target` names `north`, not `south`: its copy compares `target` with 2, the `$index` of `south`, and `target` numbers `south` 3, because it also counts `north`, declared in a for block"
    );
    assert_eq!(counted_on_read(&between, "north", &plots), ["east"]);
    assert_eq!(counted_on_read(&between, "south", &plots), ["south"]);

    // Where the two numberings agree at the number compared, the copy acts
    // for the plot `$index` gives it, and is not reported: east's copy acts
    // for south. South's compares with 3, west's `$index`, which `read`
    // gives north.
    let west = "entity west kind plot at field;\n";
    let after_south = program(&[EAST, SOUTH, ZONE, ZONE_BARE, west, READ, plus_one]);
    let report = check(&after_south);
    assert_eq!(codes(&report), [("C004", line_of(&after_south, "on read"))]);
    assert_eq!(
        report.diagnostics[0].message,
        "the rule on `read` for `south` runs when `target` names `north`, not `west`: its copy compares `target` with 3, the `$index` of `west`, and `target` numbers `west` 4, because it also counts `north`, declared in a for block"
    );
    let plots = ["east", "south", "west"];
    assert_eq!(counted_on_read(&after_south, "south", &plots), ["east"]);
    assert_eq!(counted_on_read(&after_south, "north", &plots), ["south"]);

    // E comes to its number through a function, the prelude's or the
    // program's, too. An E that reads a state has no number the check can
    // work out, and is not checked.
    for (rule, reported) in [
        ("target == round($index)", true),
        ("target == min($index, 9)", true),
        ("target == slot($index)", true),
        ("target == $index + $p_offset", false),
    ] {
        let block = format!(
            "for plot as $p {{
    state $p_n = 0;
    state $p_offset = 0;
    on read when {rule} set $p_n = $p_n + 1;
}};
"
        );
        let source = program(&[
            "fn slot(at) = at;\n",
            EAST,
            ZONE,
            ZONE_BARE,
            SOUTH,
            READ,
            &block,
        ]);
        let expected = if reported {
            vec![("C004", line_of(&source, "on read"))]
        } else {
            Vec::new()
        };
        assert_eq!(codes(&check(&source)), expected, "{rule}");
        // Each of them acts as `target == $index` does: `read north` counts
        // for south.
        assert_eq!(
            counted_on_read(&source, "north", &["east", "south"]),
            ["south"],
            "{rule}"
        );
    }
}

#[test]
fn a_selection_by_index_through_a_value_define_is_checked() {
    let reported = "the rule on `read` for `south` runs when `target` names `north`, not `south`: `$index` is 2 in its copy, and `target` numbers `south` 3, because it also counts `north`, declared in a for block";
    for (defines, rule) in [
        (
            "",
            "    define $p_slot = $index;\n    on read when target == $p_slot set $p_n = $p_n + 1;\n",
        ),
        (
            "",
            "    define $p_base = $index;\n    define $p_slot = $p_base + 0;\n    on read when $p_slot == target set $p_n = $p_n + 1;\n",
        ),
        // Written in another block over the same kind.
        (
            "for plot as $q {\n    define $q_slot = $index;\n};\n",
            "    on read when target == $p_slot set $p_n = $p_n + 1;\n",
        ),
    ] {
        let block = format!("for plot as $p {{\n    state $p_n = 0;\n{rule}}};\n");
        let source = program(&[EAST, ZONE, ZONE_BARE, SOUTH, READ, defines, &block]);
        let report = check(&source);
        assert_eq!(
            codes(&report),
            [("C004", line_of(&source, "on read"))],
            "{source}"
        );
        assert_eq!(report.diagnostics[0].message, reported);
        // As for `target == $index`: `read north` counts for south, and
        // `read south` for no plot.
        let plots = ["east", "south"];
        assert_eq!(counted_on_read(&source, "north", &plots), ["south"]);
        assert_eq!(counted_on_read(&source, "south", &plots), [] as [&str; 0]);
    }
    // A state holding `$index` is not checked, as the spec says: a state can
    // change.
    let state = "for plot as $p {
    state $p_n = 0;
    state $p_slot = $index;
    on read when target == $p_slot set $p_n = $p_n + 1;
};
";
    let source = program(&[EAST, ZONE, ZONE_BARE, SOUTH, READ, state]);
    assert_eq!(codes(&check(&source)), []);
}

#[test]
fn an_event_that_names_an_entity_a_block_declares_runs_every_copy_that_does_not_select_it_by_name()
{
    // North gets no copy of the plot block (spec/caveat-repetition-0.1.md
    // section 1), so a copy that selects its plot by name does not run on
    // `read north`. One that selects by `$index`, or selects none, can.
    let unselected = "for plot as $p {
    state $p_n = 0;
    # caveat check: allow unrouted-member-rule
    on read set $p_n = $p_n + 1;
};
";
    let plots = ["east", "south"];
    for (block, counted) in [
        (BY_NAME, &[][..]),
        (BY_INDEX, &["south"][..]),
        (unselected, &["east", "south"][..]),
    ] {
        let source = program(&[EAST, ZONE, ZONE_BARE, SOUTH, READ, block]);
        assert_eq!(
            counted_on_read(&source, "north", &plots),
            counted,
            "{block}"
        );
    }
}

// C006: round 7's C2 and C3 authors each wrote, at CR16, a journal line that
// cites a state its text never reads, and saw the program refused at dispatch
// as `evaluation/ungrounded_citation` (experiments/glowcap/round7/runs/C2 and
// C3, notes for cr16). The fixtures are those first-run programs and the
// versions that passed, copied from each run's snapshots.tgz: C2
// ba8e9e46a0f2 then 108652e6414a, C3 efb58da1bec5 then dfc0e7e6355a.
#[test]
fn round7_cr16_first_runs_report_the_citation_that_was_refused() {
    let c2 = read("tests/fixtures/c006-round7-c2-cr16-first.cav");
    let site = line_of(&c2, "bind journal_$index.text");
    assert_eq!(
        codes(&check(&c2)),
        vec![("C006", site); 6],
        "one per journal slot, at the template"
    );
    let warning = &check(&c2).diagnostics[0];
    assert_eq!(warning.name, "citation-unreachable");
    assert!(
        warning.message.contains("`journal_cite_1`"),
        "{}",
        warning.message
    );

    let c3 = read("tests/fixtures/c006-round7-c3-cr16-first.cav");
    let first = line_of(&c3, "bind $j.text = \"Absorbed the \"");
    let c3_report = check(&c3);
    let reported = codes(&c3_report);
    let lines = reported
        .iter()
        .map(|(_, line)| *line)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        reported.iter().all(|(code, _)| *code == "C006"),
        "{reported:?}"
    );
    assert_eq!(
        lines.into_iter().collect::<Vec<_>>(),
        (first..first + 8).collect::<Vec<_>>(),
        "each cited text line, not the uncited empty one"
    );

    for fixed in [
        "tests/fixtures/c006-round7-c2-cr16-fixed.cav",
        "tests/fixtures/c006-round7-c3-cr16-fixed.cav",
    ] {
        let report = check(&read(fixed));
        assert_eq!(codes(&report), [], "{fixed}: {:#?}", report.diagnostics);
    }
}

#[test]
fn a_citation_the_binding_reads_or_a_constant_is_quiet() {
    let program = |bind: &str| {
        format!(
            "claim frost;\nevidence probe from \"a probe\";\nreadings soil from probe limit 4;\nstate seen = 0;\nstate other = 0;\ndefine limit_c = 3;\nevent read celsius min -40 max 60;\non read sample soil = celsius supports frost;\non read set seen = seen + 1;\n{bind}\n"
        )
    };
    for quiet in [
        "bind ui.text = \"seen\" when seen > 0 because seen;",
        "bind ui.text = \"seen\" when seen > limit_c because seen;",
        "bind ui.text = \"soil\" when history_count(soil) > 0 because latest(soil);",
        "bind ui.text = \"x\" because nothing;",
        "bind ui.text = \"x\" when seen > 0;",
    ] {
        assert_eq!(codes(&check(&program(quiet))), [], "{quiet}");
    }
    let loud = program("bind ui.text = \"seen\" when seen > 0 because other;");
    assert_eq!(
        codes(&check(&loud)),
        [("C006", line_of(&loud, "bind ui.text"))]
    );
    let allowed = program("# caveat check: allow citation-unreachable\nbind ui.text = \"seen\" when seen > 0 because other;");
    let report = check(&allowed);
    assert_eq!(codes(&report), []);
    assert_eq!(report.suppressed.len(), 1);
}

// Every program in the repository that checks as a single file reports no
// C006, so the check adds nothing to programs that run today. The only
// reports are the first-run fixtures above.
#[test]
fn no_repository_program_reports_c006() {
    fn walk(directory: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.is_dir() {
                if !matches!(
                    name.as_str(),
                    ".git" | "node_modules" | "target" | "fixtures"
                ) {
                    walk(&path, found);
                }
            } else if name.ends_with(".cav") {
                found.push(path);
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut programs = Vec::new();
    walk(&root, &mut programs);
    let mut checked = 0;
    for program in &programs {
        let Ok(source) = std::fs::read_to_string(program) else {
            continue;
        };
        let Ok(report) = check_source(&source) else {
            continue;
        };
        checked += 1;
        let c006 = report
            .diagnostics
            .iter()
            .filter(|d| d.code == "C006")
            .count();
        assert_eq!(c006, 0, "{}: {:#?}", program.display(), report.diagnostics);
    }
    assert!(
        checked >= 50,
        "checked {checked} of {} programs",
        programs.len()
    );
}
