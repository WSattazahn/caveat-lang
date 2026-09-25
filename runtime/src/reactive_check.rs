//! `caveat check`: advisory warnings about patterns in a program that loads.
//! See spec/caveat-check-0.1.md. Nothing here changes what a program means or
//! does, and a warning describes a pattern, not a proven fault.

use super::{parse_directive_at, Directive, Effect, Expr, ReactiveSession};
use crate::link::{is_identifier_char, is_identifier_start, statement_spans, statement_words};
use crate::parser::{scan_statements_at, Position};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};

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
const ALLOW: &str = "caveat check: allow";

/// Loads the program, then looks for the patterns in
/// spec/caveat-check-0.1.md. A program that does not load is an error, as for
/// `ReactiveSession::from_source`.
pub fn check_source(source: &str) -> Result<CheckReport, String> {
    if source.starts_with(crate::link::BUNDLE_MARKER) {
        return Err("caveat check reads a single-file program, not a bundle".into());
    }
    let session = ReactiveSession::from_source(source)?;
    let statements = statements(source)?;
    let mut found = repeated_rules(&session, &statements);
    found.extend(unreopened_decisions(&session, &statements));
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

fn statements(source: &str) -> Result<Vec<Statement>, String> {
    let kinds = crate::repeat::declared_kinds(source);
    let mut spans = statement_spans(source);
    // The parser accepts a last statement without its semicolon.
    let tail = spans.last().map_or(0, |(_, end)| *end);
    if !scan_statements_at(&source[tail..], position_of(source, tail))?.is_empty() {
        spans.push((tail, source.len()));
    }
    let mut out = Vec::new();
    for (start, end) in spans {
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
        let (Some(open), Some(close)) = (text.find('{'), text.rfind('}')) else {
            continue;
        };
        let ["for", kind, "as", binding] = statement_words(&text[..open])[..] else {
            continue;
        };
        let binding = binding.trim_start_matches('$');
        let members = kinds
            .iter()
            .find(|(declared, _)| declared == kind)
            .map(|(_, members)| members.as_slice())
            .unwrap_or_default();
        let body_start = start + open + 1;
        let body = &source[body_start..start + close];
        for (inner, inner_end) in statement_spans(body) {
            let at = position_of(source, body_start + inner);
            for (index, member) in members.iter().enumerate() {
                let expanded = crate::repeat::instantiate(
                    &body[inner..inner_end],
                    binding,
                    member,
                    index + 1,
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
    }
    Ok(out)
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
