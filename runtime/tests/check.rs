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
        "../experiments/agent-ledger/ledger-identifiers.cav",
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
        // A define counts only as a whole conjunct.
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
    // A comparison of P that C003 counts as a selection, and a rule C003
    // cannot read as written. Routed, each copy runs only for its own plot.
    // Plain, neither is reported.
    let rules = "    caveat $p_fog consequence low;
    on read when target == target.north set $p_last = celsius;
    on read when celsius < -30 examine $p_fog cost $index;
";
    for source in [routed_plots(rules), plots(rules)] {
        assert_eq!(codes(&check(&format!("budget 4;\n{source}"))), []);
    }
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
