//! `caveat check`: advisory warnings about patterns in a program that loads.
//! See spec/caveat-check-0.1.md. Nothing here changes what a program means or
//! does, and a warning describes a pattern, not a proven fault.

use super::{
    parse_directive_at, Directive, Effect, Expr, FunctionDef, Parameter, ParameterDomain,
    ReactiveSession, Reads, Value,
};
use crate::link::{
    is_identifier_char, is_identifier_start, statement_spans, statement_words, statements_of,
};
use crate::parser::{scan_statements_at, Position};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub const CHECK_SCHEMA: &str = "caveat-check/0.1";

/// What `check_source` found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckReport {
    pub schema: &'static str,
    pub diagnostics: Vec<Diagnostic>,
    /// Warnings an `allow` comment silenced; listed so nothing is hidden.
    pub suppressed: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub name: &'static str,
    pub severity: &'static str,
    /// One-based line and Unicode column of the statement it is about.
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub suggestion: String,
    pub related: Vec<Related>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Related {
    pub line: usize,
    pub column: usize,
    pub note: String,
}

const REPEATED_RULES: (&str, &str) = ("C001", "repeated-rules");
const NO_REOPENING_PATH: (&str, &str) = ("C002", "no-reopening-path");
const UNROUTED_MEMBER_RULE: (&str, &str) = ("C003", "unrouted-member-rule");
const SHIFTED_MEMBER_INDEX: (&str, &str) = ("C004", "shifted-member-index");
const ALLOW: &str = "caveat check: allow";

/// Loads the program, then looks for the patterns in
/// spec/caveat-check-0.1.md. A program that does not load is an error, as for
/// `ReactiveSession::from_source`.
pub fn check_source(source: &str) -> Result<CheckReport, String> {
    if source.starts_with(crate::link::BUNDLE_MARKER) {
        return Err("caveat check reads a single-file program, not a bundle".into());
    }
    let session = ReactiveSession::from_source(source)?;
    let (statements, blocks) = statements(source)?;
    let mut found = repeated_rules(&session, &statements);
    found.extend(unreopened_decisions(&session, &statements));
    found.extend(member_rules(&session, &statements, &blocks, source));
    found.sort_by_key(|diagnostic| (diagnostic.line, diagnostic.column, diagnostic.code));

    let lines = source.lines().collect::<Vec<_>>();
    let (suppressed, diagnostics) = found
        .into_iter()
        .partition(|diagnostic| allowed(&lines, diagnostic));
    Ok(CheckReport {
        schema: CHECK_SCHEMA,
        diagnostics,
        suppressed,
    })
}

/// One source statement, comments blanked. A statement inside a `for` block
/// appears once per member, expanded, at the template's position.
struct Statement {
    text: String,
    at: Position,
    repeated: bool,
}

/// A `for` block as written, before expansion.
struct Block {
    kind: String,
    binding: String,
    /// Written `routed by P` (spec/caveat-routed-repetition-0.1.md).
    routed: bool,
    members: Vec<String>,
    /// Each statement of the body, comments blanked, at its own position.
    body: Vec<(String, Position)>,
}

fn statements(source: &str) -> Result<(Vec<Statement>, Vec<Block>), String> {
    let kinds = crate::repeat::declared_kinds(source);
    let (mut out, mut blocks) = (Vec::new(), Vec::new());
    // The parser accepts a last statement without its semicolon, after any
    // empty statements, and Repetition expands a block there.
    for (start, end) in statements_of(source) {
        let text = &source[start..end];
        let at = position_of(source, start);
        if statement_words(text).first() != Some(&"for") {
            for (text, at) in scan_statements_at(text, at)? {
                out.push(Statement {
                    text,
                    at,
                    repeated: false,
                });
            }
            continue;
        }
        let (Some(open), Some(close)) = crate::repeat::body_braces(text) else {
            continue;
        };
        let (kind, binding, routed) = match statement_words(&text[..open])[..] {
            ["for", kind, "as", binding] => (kind, binding, false),
            ["for", kind, "as", binding, "routed", "by", _] => (kind, binding, true),
            _ => continue,
        };
        let binding = binding.trim_start_matches('$');
        let members = kinds
            .iter()
            .find(|(declared, _)| declared == kind)
            .map(|(_, members)| members.as_slice())
            .unwrap_or_default();
        let body_start = start + open + 1;
        let body = &source[body_start..start + close];
        let mut templates = Vec::new();
        for (inner, inner_end) in statement_spans(body) {
            let template = &body[inner..inner_end];
            let at = position_of(source, body_start + inner);
            // A template can scan only once expanded, as `"\$p"` does; C003
            // then skips it.
            templates.extend(scan_statements_at(template, at).unwrap_or_default());
            for (index, member) in members.iter().enumerate() {
                let expanded = crate::repeat::instantiate(
                    template,
                    binding,
                    member,
                    &(index + 1).to_string(),
                )?;
                for (text, _) in scan_statements_at(&expanded, at)? {
                    out.push(Statement {
                        text,
                        at,
                        repeated: true,
                    });
                }
            }
        }
        blocks.push(Block {
            kind: kind.to_string(),
            binding: binding.to_string(),
            routed,
            members: members.to_vec(),
            body: templates,
        });
    }
    Ok((out, blocks))
}

fn position_of(source: &str, offset: usize) -> Position {
    let before = &source[..offset];
    let line_start = before.rfind('\n').map_or(0, |at| at + 1);
    Position {
        line: before.matches('\n').count() + 1,
        column: before[line_start..].chars().count() + 1,
    }
}

/// A comment line directly above the statement: `# caveat check: allow
/// NAME, …`, naming the check by code or name.
fn allowed(lines: &[&str], diagnostic: &Diagnostic) -> bool {
    let Some(above) = diagnostic
        .line
        .checked_sub(2)
        .and_then(|index| lines.get(index))
    else {
        return false;
    };
    let comment = above.trim();
    let Some(comment) = comment
        .strip_prefix('#')
        .or_else(|| comment.strip_prefix("//"))
    else {
        return false;
    };
    // Whole words, so `allowance C002` is not a directive.
    let words = comment.split_whitespace().collect::<Vec<_>>();
    let ["caveat", "check:", "allow", names @ ..] = words.as_slice() else {
        return false;
    };
    names
        .iter()
        .flat_map(|word| word.split(','))
        .any(|name| name == diagnostic.code || name == diagnostic.name)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Name(String),
    Number(String),
    Other(String),
}

/// The words of a statement: names (with `.` members), numbers, quoted text
/// and marks. Comments are already blanked.
fn tokens(text: &str) -> Vec<Token> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut out = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let start = index;
        let ch = chars[index];
        index += 1;
        if ch.is_whitespace() {
            continue;
        }
        let token = if ch == '"' {
            while index < chars.len() && chars[index] != '"' {
                index += if chars[index] == '\\' { 2 } else { 1 };
            }
            index = (index + 1).min(chars.len());
            Token::Other(chars[start..index].iter().collect())
        } else if is_identifier_start(ch) {
            while index < chars.len() && (is_identifier_char(chars[index]) || chars[index] == '.') {
                index += 1;
            }
            Token::Name(chars[start..index].iter().collect())
        } else if ch.is_ascii_digit() {
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric()
                    || chars[index] == '.'
                    || (matches!(chars[index], '+' | '-') && matches!(chars[index - 1], 'e' | 'E')))
            {
                index += 1;
            }
            Token::Number(chars[start..index].iter().collect())
        } else {
            if matches!(ch, '<' | '>' | '=' | '!') && chars.get(index) == Some(&'=') {
                index += 1;
            }
            Token::Other(chars[start..index].iter().collect())
        };
        out.push(token);
    }
    out
}

/// C001. Two events whose rules are the same rules with other names, where
/// every name that differs is one a procedure can take.
fn repeated_rules(session: &ReactiveSession, statements: &[Statement]) -> Vec<Diagnostic> {
    // Each event's top-level rules, in source order, as written.
    let mut blocks: Vec<(String, Position, Vec<Vec<Token>>)> = Vec::new();
    for statement in statements.iter().filter(|statement| !statement.repeated) {
        let Some(Ok(Directive::Rule(rule))) =
            parse_directive_at(statement.text.trim(), statement.at)
        else {
            continue;
        };
        // Everything after `on EVENT`.
        let words = tokens(&statement.text).split_off(2);
        match blocks.iter_mut().find(|(event, _, _)| *event == rule.event) {
            Some((_, _, rules)) => rules.push(words),
            None => blocks.push((rule.event, statement.at, vec![words])),
        }
    }

    let mut found = Vec::new();
    for (later, (event, at, rules)) in blocks.iter().enumerate() {
        if rules.len() < 2 {
            continue;
        }
        let Some((earlier, earlier_at, (renamed, numbers))) =
            blocks[..later]
                .iter()
                .find_map(|(other, other_at, other_rules)| {
                    let found = correspondence(other_rules, rules)?;
                    found
                        .0
                        .iter()
                        .all(|(from, to)| passable(session, other, from, event, to))
                        .then_some((other, other_at, found))
                })
        else {
            continue;
        };
        let mut names = renamed
            .iter()
            .filter(|(from, to)| from != to)
            .map(|(from, to)| format!("`{to}` for `{from}`"))
            .collect::<Vec<_>>();
        if numbers {
            names.push("other numbers".into());
        }
        let difference = match names.len() {
            0 => String::new(),
            1..=4 => format!(", with {}", names.join(", ")),
            count => format!(
                ", with {} and {} other names",
                names[..3].join(", "),
                count - 3
            ),
        };
        found.push(Diagnostic {
            code: REPEATED_RULES.0,
            name: REPEATED_RULES.1,
            severity: "warning",
            line: at.line,
            column: at.column,
            message: format!(
                "the {} rules on `{event}` repeat the rules on `{earlier}` (line {}){difference}",
                rules.len(),
                earlier_at.line
            ),
            suggestion: "If both events should keep following the same rules, they may be candidates for one `proc` that takes the names that differ, called from each event. This check compares text; whether one procedure keeps what both mean is for you to judge.".into(),
            related: vec![Related {
                line: earlier_at.line,
                column: earlier_at.column,
                note: format!("the rules on `{earlier}`"),
            }],
        });
    }
    found
}

/// Name pairs (earlier, later) for rules that match word for word apart from
/// names renamed consistently and numbers, and whether any number differs.
/// None when they differ otherwise.
fn correspondence(
    earlier: &[Vec<Token>],
    later: &[Vec<Token>],
) -> Option<(Vec<(String, String)>, bool)> {
    if earlier.len() != later.len() {
        return None;
    }
    let (mut forward, mut backward) = (HashMap::new(), HashMap::new());
    let (mut pairs, mut numbers) = (Vec::new(), false);
    for (one, two) in earlier.iter().zip(later) {
        if one.len() != two.len() {
            return None;
        }
        for (a, b) in one.iter().zip(two) {
            match (a, b) {
                (Token::Name(a), Token::Name(b)) => {
                    if *forward.entry(a.clone()).or_insert(b.clone()) != *b
                        || *backward.entry(b.clone()).or_insert(a.clone()) != *a
                    {
                        return None;
                    }
                    if !pairs.contains(&(a.clone(), b.clone())) {
                        pairs.push((a.clone(), b.clone()));
                    }
                }
                (Token::Number(a), Token::Number(b)) => numbers |= a != b,
                (a, b) if a == b => {}
                _ => return None,
            }
        }
    }
    Some((pairs, numbers))
}

/// Whether a procedure could take this pair as one parameter: two reading
/// streams, two decision series, two evidence, claims or caveats, or two event
/// parameters (numbers). The same name passes trivially.
fn passable(
    session: &ReactiveSession,
    one_event: &str,
    one: &str,
    two_event: &str,
    two: &str,
) -> bool {
    if one == two {
        return true;
    }
    let parameter = |event: &str, name: &str| {
        session
            .events
            .get(event)
            .is_some_and(|parameters| parameters.iter().any(|parameter| parameter.name == name))
    };
    (session.reading_streams.contains_key(one) && session.reading_streams.contains_key(two))
        || (session.decision_series.contains_key(one) && session.decision_series.contains_key(two))
        || ["evidence", "claim", "caveat"].iter().any(|kind| {
            session.require_kind(one, kind).is_ok() && session.require_kind(two, kind).is_ok()
        })
        || (parameter(one_event, one) && parameter(two_event, two))
}

/// C002. A decision series committed on readings that nothing reopens: no
/// `reopen` of it in any rule or procedure, and no declared trigger.
fn unreopened_decisions(session: &ReactiveSession, statements: &[Statement]) -> Vec<Diagnostic> {
    let steps = session
        .rules
        .iter()
        .map(|rule| (&rule.condition, &rule.effect))
        .chain(
            session
                .procedures
                .values()
                // A template runs only as its specialized copies.
                .filter(|procedure| procedure.symbol_parameters.is_empty())
                .flat_map(|procedure| {
                    procedure
                        .body
                        .iter()
                        .map(|step| (&step.condition, &step.effect))
                }),
        );
    let mut reopened = session
        .reopening_triggers
        .iter()
        .map(|(series, _, _)| series.clone())
        .collect::<BTreeSet<_>>();
    let mut read_by: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (condition, effect) in steps {
        match effect {
            Effect::Reopen { action, .. } => {
                reopened.insert(action.clone());
            }
            Effect::Commit { action, using, .. }
                if session.decision_series.contains_key(action) =>
            {
                let mut histories = BTreeSet::new();
                condition.collect_histories(&mut histories);
                using
                    .iter()
                    .for_each(|using: &Expr| using.collect_histories(&mut histories));
                read_by.entry(action.clone()).or_default().extend(
                    histories
                        .into_iter()
                        .filter(|history| session.reading_streams.contains_key(history)),
                );
            }
            _ => {}
        }
    }

    let declared = statements
        .iter()
        .filter_map(
            |statement| match parse_directive_at(statement.text.trim(), statement.at) {
                Some(Ok(Directive::Decisions { name, .. })) => Some((name, statement.at)),
                _ => None,
            },
        )
        .collect::<HashMap<_, _>>();
    read_by
        .into_iter()
        .filter(|(series, streams)| !streams.is_empty() && !reopened.contains(series))
        .map(|(series, streams)| {
            let at = declared.get(&series).copied().unwrap_or(Position { line: 1, column: 1 });
            let streams = streams.into_iter().map(|stream| format!("`{stream}`")).collect::<Vec<_>>();
            let first = streams[0].trim_matches('`').to_string();
            Diagnostic {
                code: NO_REOPENING_PATH.0,
                name: NO_REOPENING_PATH.1,
                severity: "warning",
                line: at.line,
                column: at.column,
                message: format!(
                    "`{series}` is decided on readings from {}, and no reopening path was detected: nothing in the program reopens it, so once made it stays in force whatever those streams record later, and the series never holds more than its first revision",
                    streams.join(", ")
                ),
                suggestion: format!(
                    "Confirm that keeping `{series}` fixed is intended. If a later reading should reconsider it, declare `decisions {series} limit N reopened by {first}` or add a rule that reopens it; if it should stay fixed, put `# {ALLOW} {}` on the line above its declaration.",
                    NO_REOPENING_PATH.1
                ),
                related: Vec::new(),
            }
        })
        .collect()
}

/// C003 and C004. Each rule written in a plain `for` block about its member,
/// on an event that names a member of the block's kind, read as written and
/// in each member's copy. C003: its guard does not select which member the
/// event names. C004: it selects it by a number made from `$index`, which the
/// event gives another entity than `$index` does.
fn member_rules(
    session: &ReactiveSession,
    statements: &[Statement],
    blocks: &[Block],
    source: &str,
) -> Vec<Diagnostic> {
    // Names the source never uses, put for `$NAME` and for `$index` to read a
    // rule as written: whatever contains one came from that binding.
    let unused = |stem: &str| {
        let mut marker = String::from(stem);
        while source.contains(&marker) {
            marker.push('_');
        }
        marker
    };
    let (member_marker, index_marker) =
        (unused("caveat_check_member"), unused("caveat_check_index"));
    // The program's defines, and the functions an expression can call: the
    // prelude's and the program's.
    let mut program_defines = HashMap::new();
    let mut functions = super::prelude_functions().unwrap_or_default();
    for statement in statements {
        match parse_directive_at(statement.text.trim(), statement.at) {
            Some(Ok(Directive::Define { name, expression })) => {
                program_defines.insert(name, expression);
            }
            Some(Ok(Directive::Function(function))) => {
                functions.insert(function.name.clone(), function);
            }
            _ => {}
        }
    }

    let read_as_written = |text: &str, binding: &str, at: Position| {
        let text = crate::repeat::instantiate(text, binding, &member_marker, &index_marker).ok()?;
        parse_directive_at(text.trim(), at)?.ok()
    };
    // The defines written in blocks, as written, by the kind the block ranges
    // over. The markers stand for any binding, so a rule can use a define
    // that another block over the same kind wrote.
    let mut kind_defines: HashMap<&str, Vec<(String, Expr)>> = HashMap::new();
    for block in blocks {
        for (text, at) in &block.body {
            if let Some(Directive::Define { name, expression }) =
                read_as_written(text, &block.binding, *at)
            {
                kind_defines
                    .entry(block.kind.as_str())
                    .or_default()
                    .push((name, expression));
            }
        }
    }
    // Where a `for` block declares each entity: at its statement in the
    // block, where each of the block's copies is placed.
    let mut declared_at = HashMap::new();
    for statement in statements.iter().filter(|statement| statement.repeated) {
        if let ["entity", name, "kind", _, "at", _] =
            statement.text.split_whitespace().collect::<Vec<_>>()[..]
        {
            declared_at.entry(name).or_insert(statement.at);
        }
    }
    // The number each define of the program comes to, once read.
    let known = RefCell::new(HashMap::new());

    let mut found = Vec::new();
    // A routed block's rules are routed, name no member, or stop loading.
    // Their copies are not read instead: there the route compares with a
    // number, which is not a selection here. Its defines still count above.
    // A routed block refuses an entity of its kind in a `for` block, so its
    // `$index` numbers each member as the event does.
    for block in blocks.iter().filter(|block| !block.routed) {
        let as_written = |text: &str, at: Position| read_as_written(text, &block.binding, at);
        let mut defines = program_defines.clone();
        defines.extend(
            kind_defines
                .get(block.kind.as_str())
                .into_iter()
                .flatten()
                .cloned(),
        );
        // `Q.MEMBER` for a parameter `Q` of the block's kind.
        let member_constant = |name: &str| {
            name.split_once('.').is_some_and(|(parameter, member)| {
                block.members.iter().any(|declared| declared == member)
                    && session.events.values().flatten().any(|declared| {
                        declared.name == parameter && of_kind(declared, &block.kind)
                    })
            })
        };
        let routes = |value: &Expr| {
            value.mentions(&member_marker)
                || value.mentions(&index_marker)
                || value.as_name().is_some_and(member_constant)
        };
        // How each `kind` parameter numbers the block's members, by event
        // and parameter: None where it numbers each as `$index` does.
        let mut numberings = HashMap::new();

        for (template, at) in &block.body {
            // A rule that mentions neither binding is not about one member.
            if !template.contains('$') {
                continue;
            }
            let Some(Directive::Rule(written)) = as_written(template, *at) else {
                continue;
            };
            // An event named after the member is that member's own, so the
            // event already selects it.
            let own_event =
                written.event.contains(&member_marker) || written.event.contains(&index_marker);
            let mut conjuncts = Vec::new();
            guard_conjuncts(
                &written.condition,
                &defines,
                &mut Vec::new(),
                &mut conjuncts,
            );
            // Each conjunct `P == E` or `E == P` whose E reads `$index`,
            // itself or through a define, as (P, E), in the guard's order.
            let by_index = conjuncts
                .iter()
                .filter_map(|conjunct| conjunct.equality())
                .flat_map(|(left, right)| [(left, right), (right, left)])
                .filter_map(|(parameter, value)| Some((parameter.as_name()?, value)))
                .filter(|(_, value)| {
                    reads_through(value, &index_marker, &defines, &mut HashSet::new())
                })
                .collect::<Vec<_>>();
            for (index, member) in block.members.iter().enumerate() {
                let copy = crate::repeat::instantiate(
                    template,
                    &block.binding,
                    member,
                    &(index + 1).to_string(),
                );
                let Some(Ok(Directive::Rule(rule))) = copy
                    .ok()
                    .and_then(|copy| parse_directive_at(copy.trim(), *at))
                else {
                    continue;
                };
                // An event that names no member of the kind reaches them all.
                let Some((event, declared)) = session.events.get_key_value(&rule.event) else {
                    continue;
                };
                let parameters = declared
                    .iter()
                    .filter(|parameter| of_kind(parameter, &block.kind))
                    .collect::<Vec<_>>();
                let Some(first) = parameters.first() else {
                    continue;
                };
                // The parameter a conjunct `P == E` or `E == P` compares with
                // a value `accepts` takes.
                let compared = |conjunct: &Expr, accepts: &dyn Fn(&Expr) -> bool| {
                    let (left, right) = conjunct.equality()?;
                    [(left, right), (right, left)]
                        .into_iter()
                        .find_map(|(parameter, value)| {
                            let name = parameter.as_name()?;
                            let declared =
                                parameters.iter().find(|declared| declared.name == name)?;
                            accepts(value).then_some(*declared)
                        })
                };
                let selection = by_index.iter().find_map(|(name, value)| {
                    let parameter = parameters.iter().find(|declared| declared.name == *name)?;
                    Some((*parameter, *value))
                });
                if let Some((parameter, value)) = selection {
                    let numbering = numberings
                        .entry((event.as_str(), parameter.name.as_str()))
                        .or_insert_with(|| numbering(parameter, block));
                    // The number E comes to in the member's copy: `$index` is
                    // the member's, and a name written with a binding is the
                    // copy's.
                    let position = index + 1;
                    let in_copy = |name: &str| {
                        if name == index_marker {
                            return Some(position as f64);
                        }
                        let name = name
                            .replace(&member_marker, member)
                            .replace(&index_marker, &position.to_string());
                        defined(&name, (&program_defines, &functions), &known)
                    };
                    if let Some((numbering, number)) = numbering.as_ref().and_then(|numbering| {
                        Some((numbering, constant(value, &functions, &in_copy)?))
                    }) {
                        found.extend(shifted_member_index(
                            block,
                            (position, member),
                            (event, parameter),
                            (number, numbering),
                            *at,
                            &declared_at,
                        ));
                    }
                }
                if own_event
                    || conjuncts
                        .iter()
                        .any(|conjunct| compared(conjunct, &routes).is_some())
                {
                    continue;
                }
                let named = parameters
                    .iter()
                    .map(|parameter| format!("`{}`", parameter.name))
                    .collect::<Vec<_>>()
                    .join(" or ");
                found.push(Diagnostic {
                    code: UNROUTED_MEMBER_RULE.0,
                    name: UNROUTED_MEMBER_RULE.1,
                    severity: "warning",
                    line: at.line,
                    column: at.column,
                    message: format!(
                        "the rule on `{}` for `{member}` runs whichever {} {named} names: its guard does not select one",
                        rule.event, block.kind
                    ),
                    suggestion: format!(
                        "If the rule is about the {kind} the event names, add the selection to its guard, such as `{} == $index`. If it should run for every {kind} on each `{}`, put `# {ALLOW} {}` on the line above it.",
                        first.name,
                        rule.event,
                        UNROUTED_MEMBER_RULE.1,
                        kind = block.kind
                    ),
                    related: Vec::new(),
                });
            }
        }
    }
    found
}

/// How a `kind` parameter numbers the entities of a block's kind, where it
/// numbers a top-level member otherwise than `$index` does.
struct Numbering<'s> {
    /// Each entity's number in the parameter, from 1.
    numbers: HashMap<&'s str, usize>,
    /// The entities it counts and `$index` does not, those a `for` block
    /// declares, each with its number, in order.
    declared: Vec<(usize, &'s str)>,
}

/// None when `parameter` numbers each of the block's members as its `$index`
/// does: every entity of the kind that a `for` block declares comes after
/// them, or there is none.
fn numbering<'s>(parameter: &'s Parameter, block: &Block) -> Option<Numbering<'s>> {
    let ParameterDomain::Entity { members, .. } = &parameter.domain else {
        return None;
    };
    let numbers = members
        .iter()
        .enumerate()
        .map(|(at, name)| (name.as_str(), at + 1))
        .collect::<HashMap<_, _>>();
    let agree = block.members.iter().enumerate().all(|(at, member)| {
        numbers
            .get(member.as_str())
            .is_none_or(|number| *number == at + 1)
    });
    if agree {
        return None;
    }
    let top_level = block
        .members
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let declared = members
        .iter()
        .enumerate()
        .filter(|(_, name)| !top_level.contains(name.as_str()))
        .map(|(at, name)| (at + 1, name.as_str()))
        .collect();
    Some(Numbering { numbers, declared })
}

/// C004 for one member's copy of a rule that selects its member by comparing
/// `parameter` with `number`, what E comes to in the copy. `$index` numbers
/// the part's top-level entities of the kind; the parameter numbers every
/// entity of the kind in the loaded program, those a `for` block declares
/// included. None when the copy acts for its own member, or for the member
/// whose `$index` is `number`.
fn shifted_member_index(
    block: &Block,
    (index, member): (usize, &str),
    (event, parameter): (&str, &Parameter),
    (number, numbering): (f64, &Numbering),
    at: Position,
    declared_at: &HashMap<&str, Position>,
) -> Option<Diagnostic> {
    let ParameterDomain::Entity { members, .. } = &parameter.domain else {
        return None;
    };
    if number < 1.0 || number.fract() != 0.0 {
        return None;
    }
    let number = number as usize;
    // The copy runs when the parameter names the entity it numbers so, and
    // `$index` gives that number to `meant`.
    let (acts_for, meant) = (members.get(number - 1)?, block.members.get(number - 1)?);
    if acts_for == member || acts_for == meant {
        return None;
    }
    let meant_number = *numbering.numbers.get(meant.as_str())?;
    // The entities the parameter counts before `meant` and `$index` does not.
    let counted = &numbering.declared[..numbering
        .declared
        .partition_point(|(counted, _)| *counted < meant_number)];
    let names = counted
        .iter()
        .take(3)
        .map(|(_, name)| format!("`{name}`"))
        .collect::<Vec<_>>();
    let also = match (names.as_slice(), counted.len()) {
        ([one], 1) => format!("{one}, declared in a for block"),
        ([rest @ .., last], count) if count == names.len() => {
            format!(
                "{} and {last}, each declared in a for block",
                rest.join(", ")
            )
        }
        (_, count) => format!(
            "{} and {} others, each declared in a for block",
            names.join(", "),
            count - names.len()
        ),
    };
    let related = counted
        .iter()
        .take(3)
        .filter_map(|(_, name)| {
            declared_at.get(name).map(|declared| Related {
                line: declared.line,
                column: declared.column,
                note: format!("where a for block declares `{name}`"),
            })
        })
        .collect();
    let parameter = &parameter.name;
    let compared = if meant == member {
        format!("`$index` is {index} in its copy")
    } else {
        format!("its copy compares `{parameter}` with {number}, the `$index` of `{meant}`")
    };
    Some(Diagnostic {
        code: SHIFTED_MEMBER_INDEX.0,
        name: SHIFTED_MEMBER_INDEX.1,
        severity: "warning",
        line: at.line,
        column: at.column,
        message: format!(
            "the rule on `{event}` for `{member}` runs when `{parameter}` names `{acts_for}`, not `{meant}`: {compared}, and `{parameter}` numbers `{meant}` {meant_number}, because it also counts {also}"
        ),
        suggestion: format!(
            "If the rule is about the {kind} the event names, select it by name, such as `{parameter} == {parameter}.${}`, which names its own {kind} however the entities are counted. If comparing with `$index` is intended, put `# {ALLOW} {}` on the line above it.",
            block.binding,
            SHIFTED_MEMBER_INDEX.1,
            kind = block.kind
        ),
        related,
    })
}

/// Whether `value` reads `name`, itself or through the defines it reads.
fn reads_through<'d>(
    value: &Expr,
    name: &str,
    defines: &'d HashMap<String, Expr>,
    seen: &mut HashSet<&'d str>,
) -> bool {
    if value.mentions(name) {
        return true;
    }
    let mut reads = Reads::default();
    value.collect_reads(&mut reads);
    reads
        .names
        .iter()
        .any(|read| match defines.get_key_value(read.as_str()) {
            Some((define, expression)) if seen.insert(define.as_str()) => {
                reads_through(expression, name, defines, seen)
            }
            _ => false,
        })
}

/// The number `value` comes to, with its calls to `functions` expanded,
/// when each name it reads is what `number` gives; None where it reads
/// anything else, such as a state or the graph.
fn constant(
    value: &Expr,
    functions: &BTreeMap<String, FunctionDef>,
    number: &dyn Fn(&str) -> Option<f64>,
) -> Option<f64> {
    let value = super::reactive_expr::expand(value, functions).ok()?;
    match value.evaluate(&|name: &str| number(name), &|_: &str, _: &str| {
        Err(String::new())
    }) {
        Ok(Value::Number(number)) => Some(number),
        _ => None,
    }
}

/// The number the program's define `name` comes to, as `constant` reads it,
/// through the other defines it reads. Each is read once, into `known`.
fn defined(
    name: &str,
    (defines, functions): (&HashMap<String, Expr>, &BTreeMap<String, FunctionDef>),
    known: &RefCell<HashMap<String, Option<f64>>>,
) -> Option<f64> {
    if let Some(number) = known.borrow().get(name) {
        return *number;
    }
    let expression = defines.get(name)?;
    // None while it is read, so a define that reads itself comes to none.
    known.borrow_mut().insert(name.to_string(), None);
    let number = constant(expression, functions, &|name| {
        defined(name, (defines, functions), known)
    });
    known.borrow_mut().insert(name.to_string(), number);
    number
}

/// Whether an event parameter is declared `NAME kind KIND`.
fn of_kind(parameter: &Parameter, kind: &str) -> bool {
    matches!(&parameter.domain, ParameterDomain::Entity { kind: declared, .. } if declared == kind)
}

/// The operands of a guard's chain of `and`, each one that names a define
/// replaced by the operands of its expression.
fn guard_conjuncts<'a>(
    condition: &'a Expr,
    defines: &'a HashMap<String, Expr>,
    inlining: &mut Vec<&'a str>,
    out: &mut Vec<&'a Expr>,
) {
    for conjunct in condition.conjuncts() {
        match conjunct
            .as_name()
            .and_then(|name| defines.get_key_value(name))
        {
            Some((name, expression)) if !inlining.contains(&name.as_str()) => {
                inlining.push(name);
                guard_conjuncts(expression, defines, inlining, out);
                inlining.pop();
            }
            _ => out.push(conjunct),
        }
    }
}
