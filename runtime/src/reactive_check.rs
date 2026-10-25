//! `caveat check`: advisory warnings about patterns in a program that loads.
//! See spec/caveat-check-0.1.md. Nothing here changes what a program means or
//! does, and a warning describes a pattern, not a proven fault.

use super::{
    parse_directive_at, BindingExpression, DecisionChange, Directive, Effect, EvidenceSelector,
    Expr, FunctionDef, Parameter, ParameterDomain, Provenance, ReactiveSession, Reads, Value,
};
use crate::fast_hash::{HashMap, HashSet};
use crate::link::{
    is_identifier_char, is_identifier_start, statement_spans, statement_words, statements_of,
};
use crate::parser::{position_of, scan_statements_at, Position};
use crate::NodeId;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

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
const CITATION_UNREACHABLE: (&str, &str) = ("C006", "citation-unreachable");
const UNUSED_DECLARATION: (&str, &str) = ("C007", "unused-declaration");
const RECORD_READS_LATE_CAVEAT: (&str, &str) = ("C008", "record-reads-late-caveat");
const SAVE_LACKS_LATE_CAVEAT: (&str, &str) = ("C009", "save-lacks-late-caveat");
const COMMIT_ON_WITHDRAWABLE: (&str, &str) = ("C010", "commit-on-withdrawable");
const WITHDRAWAL_NOT_REOPENED: (&str, &str) = ("C011", "withdrawal-not-reopened");
const ALLOW: &str = "caveat check: allow";

/// Which checks run beyond the default ones. Each is asked for, so a program
/// that checked clean before still does (docs/COMPATIBILITY.md).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CheckOptions {
    /// C007 `unused-declaration`.
    pub unused: bool,
    /// C008 `record-reads-late-caveat`, and C009 `save-lacks-late-caveat`
    /// when a save is checked (`check_save_with`).
    pub records: bool,
    /// C010 `commit-on-withdrawable` and C011 `withdrawal-not-reopened`.
    pub withdrawals: bool,
}

/// Loads the program, then looks for the patterns in
/// spec/caveat-check-0.1.md. A program that does not load is an error, as for
/// `ReactiveSession::from_source`.
pub fn check_source(source: &str) -> Result<CheckReport, String> {
    check_source_with(source, CheckOptions::default())
}

/// `check_source`, with the checks `options` asks for as well.
pub fn check_source_with(source: &str, options: CheckOptions) -> Result<CheckReport, String> {
    report(source, options, None)
}

/// `check_source_with`, with C009 for the session that `save`, a save's
/// JSON, restores: each value it holds without a late caveat that evidence
/// it lists now carries. C009 belongs to the records check, so `options`
/// must ask for it. An error when the program does not load or the save
/// does not restore.
pub fn check_save_with(
    source: &str,
    save: &str,
    options: CheckOptions,
) -> Result<CheckReport, String> {
    if !options.records {
        return Err("a save is checked only with the records check".into());
    }
    report(source, options, Some(save))
}

fn report(source: &str, options: CheckOptions, save: Option<&str>) -> Result<CheckReport, String> {
    if source.starts_with(crate::link::BUNDLE_MARKER) {
        return Err("caveat check reads a single-file program, not a bundle".into());
    }
    let session = ReactiveSession::from_source(source)?;
    let saved = save
        .map(|save| ReactiveSession::restore_json(source, save))
        .transpose()?;
    let (statements, blocks) = statements(source)?;
    let mut found = repeated_rules(&session, &statements);
    found.extend(unreopened_decisions(&session, &statements));
    found.extend(member_rules(&session, &statements, &blocks, source));
    found.extend(unreachable_citations(&session, &statements));
    if options.unused {
        found.extend(unused_declarations(&statements));
    }
    if options.records {
        found.extend(record_reads(&session, &statements));
    }
    if let Some(saved) = &saved {
        found.extend(saved_without_late_caveats(saved, &statements));
    }
    if options.withdrawals {
        found.extend(withdrawal_checks(&statements));
    }
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
        let walk = Walk {
            source,
            kinds: &kinds,
        };
        walk.block((start, end), &[], &mut out, &mut blocks)?;
    }
    Ok((out, blocks))
}

/// What reading a `for` block, and the blocks nested in it, needs.
struct Walk<'a> {
    source: &'a str,
    kinds: &'a [(String, Vec<String>)],
}

/// A `for` block's header and members, read as written.
struct Header<'a> {
    kind: &'a str,
    binding: &'a str,
    routed: bool,
    members: &'a [String],
    body_start: usize,
    body: &'a str,
}

impl<'a> Walk<'a> {
    fn header(&self, (start, end): (usize, usize)) -> Option<Header<'a>> {
        let text = &self.source[start..end];
        let (Some(open), Some(close)) = crate::repeat::body_braces(text) else {
            return None;
        };
        let (kind, binding, routed) = match statement_words(&text[..open])[..] {
            ["for", kind, "as", binding] => (kind, binding, false),
            ["for", kind, "as", binding, "routed", "by", _] => (kind, binding, true),
            _ => return None,
        };
        let members = self
            .kinds
            .iter()
            .find(|(declared, _)| declared == kind)
            .map(|(_, members)| members.as_slice())
            .unwrap_or_default();
        Some(Header {
            kind,
            binding: binding.trim_start_matches('$'),
            routed,
            members,
            body_start: start + open + 1,
            body: &text[open + 1..close],
        })
    }

    /// The block at `span`, inside the blocks whose bindings and members
    /// `scope` gives, outermost first. Each of its statements goes to `out`
    /// once per member, expanded, and the block to `blocks` with its body
    /// as written, the enclosing members put in. A nested block is read once
    /// per member of this one, and its copies are statements of this
    /// block's body.
    fn block(
        &self,
        span: (usize, usize),
        scope: &[(&str, &str)],
        out: &mut Vec<Statement>,
        blocks: &mut Vec<Block>,
    ) -> Result<(), String> {
        let Some(header) = self.header(span) else {
            return Ok(());
        };
        let mut templates = Vec::new();
        for (template, at) in self.templates(&header, scope, &[])? {
            // A template can scan only once expanded, as `"\$p"` does; C003
            // then skips it.
            templates.extend(scan_statements_at(&template, at).unwrap_or_default());
            for (index, member) in header.members.iter().enumerate() {
                let expanded = crate::repeat::instantiate(
                    &template,
                    header.binding,
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
            kind: header.kind.to_string(),
            binding: header.binding.to_string(),
            routed: header.routed,
            members: header.members.to_vec(),
            body: templates,
        });
        // Each nested block is a block of its own too, once per member.
        for (inner, inner_end) in statement_spans(header.body) {
            if statement_words(&header.body[inner..inner_end]).first() != Some(&"for") {
                continue;
            }
            for member in header.members {
                let scope = [scope, &[(header.binding, member.as_str())][..]].concat();
                self.block(
                    (header.body_start + inner, header.body_start + inner_end),
                    &scope,
                    &mut Vec::new(),
                    blocks,
                )?;
            }
        }
        Ok(())
    }

    /// The statements of a block's body as one member's copy would read
    /// them before its own binding is put in: the enclosing members in
    /// `scope` put in, and each nested block expanded, each at the position
    /// of the statement it comes from.
    fn templates(
        &self,
        header: &Header,
        scope: &[(&str, &str)],
        enclosing: &[&str],
    ) -> Result<Vec<(String, Position)>, String> {
        let kept = [&[header.binding, "index"][..], enclosing].concat();
        let mut out = Vec::new();
        for (inner, inner_end) in statement_spans(header.body) {
            let written = &header.body[inner..inner_end];
            let at = position_of(self.source, header.body_start + inner);
            if statement_words(written).first() != Some(&"for") {
                out.push((
                    crate::repeat::substitute_keeping(written, scope, &kept)?,
                    at,
                ));
                continue;
            }
            let span = (header.body_start + inner, header.body_start + inner_end);
            let Some(nested) = self.header(span) else {
                continue;
            };
            let templates = self.templates(&nested, scope, &kept)?;
            for (index, member) in nested.members.iter().enumerate() {
                let index = (index + 1).to_string();
                for (template, at) in &templates {
                    out.push((
                        crate::repeat::substitute_keeping(
                            template,
                            &[(nested.binding, member), ("index", &index)],
                            &kept,
                        )?,
                        *at,
                    ));
                }
            }
        }
        Ok(out)
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
    let (mut forward, mut backward) = (HashMap::default(), HashMap::default());
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

/// C006: a binding's `because` cites something its value and condition never
/// read, on any path. Compared by name after defines and calls are expanded:
/// the sources a citation names (states, evidence, caveats, histories,
/// `elapsed()`) against those its binding names. A citation sharing none of
/// them is the shape `ungrounded_citation` refuses at dispatch; one sharing a
/// name may still be refused, and one sharing none may hold through a state
/// whose lineage carries what it cites, which is why this only warns.
fn unreachable_citations(session: &ReactiveSession, statements: &[Statement]) -> Vec<Diagnostic> {
    // Each `bind` statement's position, matched to the compiled binding by
    // target, property and order: the bindings of one target and property
    // keep their source order however a `for` block is expanded.
    let mut positions: HashMap<(String, String), Vec<Position>> = HashMap::default();
    for statement in statements {
        if let Some(Ok(Directive::Binding(binding))) =
            parse_directive_at(statement.text.trim(), statement.at)
        {
            positions
                .entry((binding.target, binding.property))
                .or_default()
                .push(statement.at);
        }
    }
    let mut seen: HashMap<(String, String), usize> = HashMap::default();
    let mut found = Vec::new();
    for binding in session.binding_rules.iter() {
        let key = (binding.target.clone(), binding.property.clone());
        let index = seen.entry(key.clone()).or_default();
        let at = positions
            .get(&key)
            .and_then(|list| list.get(*index))
            .copied()
            .unwrap_or(Position { line: 1, column: 1 });
        *index += 1;
        let Some(citations) = &binding.because else {
            continue;
        };
        let mut read = BTreeSet::new();
        binding.condition.collect_sources(&mut read);
        if let super::BindingExpression::Expression(value) = &binding.value {
            value.collect_sources(&mut read);
        }
        for citation in citations {
            let mut cited = BTreeSet::new();
            citation.collect_sources(&mut cited);
            cited.retain(|name| !session.constants.contains_key(name));
            if cited.is_empty() || !cited.is_disjoint(&read) {
                continue;
            }
            let cited = cited
                .iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>();
            let read = read
                .iter()
                .filter(|name| !session.constants.contains_key(*name))
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>();
            found.push(Diagnostic {
                code: CITATION_UNREACHABLE.0,
                name: CITATION_UNREACHABLE.1,
                severity: "warning",
                line: at.line,
                column: at.column,
                message: format!(
                    "`{}.{}` cites {}, which its value and condition never read{}: when it wins, the event is refused as `evaluation/ungrounded_citation` unless what it reads carries what the citation cites",
                    binding.target,
                    binding.property,
                    cited.join(", "),
                    if read.is_empty() {
                        String::new()
                    } else {
                        format!(" (they read {})", read.join(", "))
                    }
                ),
                suggestion: format!(
                    "Read what you cite: use it in the value or the `when` condition, cite something the binding already reads, or drop the citation. If a state it reads carries what it cites, put `# {ALLOW} {}` on the line above.",
                    CITATION_UNREACHABLE.1
                ),
                related: Vec::new(),
            });
        }
    }
    found
}

/// C007. A claim, evidence, caveat or state that no statement names but its
/// own declaration. Names are compared as written, after `for` blocks are
/// expanded, so a name read only in an unused `define` or never-called `proc`
/// still counts as named: the check can miss a leftover, never invent one.
fn unused_declarations(statements: &[Statement]) -> Vec<Diagnostic> {
    let mut named: HashMap<String, usize> = HashMap::default();
    let mut declared = Vec::new();
    for statement in statements {
        let tokens = tokens(&statement.text);
        for token in &tokens {
            if let Token::Name(name) = token {
                for part in name.split('.') {
                    *named.entry(part.to_string()).or_default() += 1;
                }
            }
        }
        let kind = match tokens.as_slice() {
            [Token::Name(word), Token::Name(_), ..] if word == "claim" => "claim",
            [Token::Name(word), Token::Name(_), Token::Name(from), ..]
                if word == "evidence" && from == "from" =>
            {
                "evidence"
            }
            [Token::Name(word), Token::Name(_), Token::Name(consequence), ..]
                if word == "caveat" && consequence == "consequence" =>
            {
                "caveat"
            }
            [Token::Name(word), Token::Name(_), Token::Other(equals), ..]
                if word == "state" && equals == "=" =>
            {
                "state"
            }
            _ => continue,
        };
        let Token::Name(name) = &tokens[1] else {
            continue;
        };
        declared.push((kind, name.clone(), statement.at));
    }
    declared
        .into_iter()
        // Named once: by its own declaration.
        .filter(|(_, name, _)| named.get(name).copied().unwrap_or_default() <= 1)
        .map(|(kind, name, at)| {
            let (what, effect) = match kind {
                "state" => (
                    "state",
                    "no rule sets or reads it and no binding shows it, so it keeps its initial value",
                ),
                "evidence" => (
                    "evidence",
                    "no rule reveals, withdraws or cites it and no stream reads from it",
                ),
                "caveat" => (
                    "caveat",
                    "nothing qualifies evidence with it and no rule reads it",
                ),
                _ => (
                    "claim",
                    "no evidence or reading supports or opposes it and no trigger names it",
                ),
            };
            Diagnostic {
                code: UNUSED_DECLARATION.0,
                name: UNUSED_DECLARATION.1,
                severity: "warning",
                line: at.line,
                column: at.column,
                message: format!(
                    "the {what} `{name}` is declared, and nothing else in the program names it: {effect}"
                ),
                suggestion: format!(
                    "If it is left over from an earlier version, remove it, so readers do not take it for part of the policy. If a host reads it on purpose, put `# {ALLOW} {}` on the line above.",
                    UNUSED_DECLARATION.1
                ),
                related: Vec::new(),
            }
        })
        .collect()
}

/// The expressions an effect evaluates, taken or not.
fn effect_expressions(effect: &Effect) -> Vec<&Expr> {
    fn selector(selector: &EvidenceSelector) -> Option<&Expr> {
        match selector {
            EvidenceSelector::Occurrence { index, .. } => Some(index),
            _ => None,
        }
    }
    match effect {
        Effect::Call { arguments, .. } => arguments.iter().collect(),
        Effect::Sample { value, note, .. } => {
            std::iter::once(value).chain(note.as_deref()).collect()
        }
        Effect::Qualify { after, index, .. } => after.iter().chain(index.as_deref()).collect(),
        Effect::Unqualify { index, .. } => index.as_deref().into_iter().collect(),
        Effect::Withdraw { target, .. } => selector(target).into_iter().collect(),
        Effect::Reopen { because, .. } => selector(because).into_iter().collect(),
        Effect::Resolve(resolution) => std::iter::once(&resolution.value)
            .chain(&resolution.revision)
            .chain(selector(&resolution.because))
            .collect(),
        Effect::Set { value, because, .. } => std::iter::once(value)
            .chain(because.iter().flatten())
            .collect(),
        Effect::Commit {
            using,
            citing,
            reasons,
            ..
        } => using
            .iter()
            .chain(citing)
            .chain(reasons.iter().map(|(_, _, read, _)| read))
            .collect(),
        Effect::Emit { .. }
        | Effect::Reject { .. }
        | Effect::Renew { .. }
        | Effect::Observe { .. }
        | Effect::Reveal { .. }
        | Effect::Examine { .. } => Vec::new(),
    }
}

/// The records `value` reads (`Expr::collect_record_reads`), itself or
/// through the defines it reads.
fn records_through<'d>(
    value: &Expr,
    defines: &'d HashMap<String, Expr>,
    seen: &mut HashSet<&'d str>,
    records: &mut BTreeSet<(&'static str, String)>,
) {
    value.collect_record_reads(records);
    let mut reads = Reads::default();
    value.collect_reads(&mut reads);
    for read in &reads.names {
        if let Some((define, expression)) = defines.get_key_value(read.as_str()) {
            if seen.insert(define.as_str()) {
                records_through(expression, defines, seen, records);
            }
        }
    }
}

/// Each statement with a `qualify` effect, with the caveat it names: in a
/// rule, or in a procedure's body, at the procedure.
fn late_qualifications(statements: &[Statement]) -> Vec<(String, Related)> {
    let mut found = Vec::new();
    for statement in statements {
        let effects = match parse_directive_at(statement.text.trim(), statement.at) {
            Some(Ok(Directive::Rule(rule))) => vec![rule.effect],
            Some(Ok(Directive::Procedure(procedure))) => {
                procedure.body.into_iter().map(|step| step.effect).collect()
            }
            _ => continue,
        };
        for effect in effects {
            if let Effect::Qualify {
                evidence,
                caveat,
                index,
                latest,
                ..
            } = effect
            {
                let target = if latest {
                    format!("latest({evidence})")
                } else if index.is_some() {
                    format!("occurrence({evidence}, …)")
                } else {
                    evidence
                };
                let note = format!("`qualify {target} with {caveat}`");
                if !found.iter().any(|(_, related): &(String, Related)| {
                    related.line == statement.at.line && related.note == note
                }) {
                    found.push((
                        caveat,
                        Related {
                            line: statement.at.line,
                            column: statement.at.column,
                            note,
                        },
                    ));
                }
            }
        }
    }
    found
}

/// C008. A rule, procedure or binding that reads a decision's record, in a
/// program that qualifies evidence late: `committed(D)` or `reopened(D)`, or
/// `latest`, `history_count`, `history_at` or a fold of a decision series.
/// From 1.1 a value read through a record reads the evidence the record
/// lists live, so a later `qualify` of that evidence adds its caveat to a
/// value set this way whenever it was set; a 1.0.x runtime did not (D015,
/// spec/caveat-caveat-sources-0.1.md). Compared by name after defines are
/// expanded, so a read on a branch never taken is reported too.
fn record_reads(session: &ReactiveSession, statements: &[Statement]) -> Vec<Diagnostic> {
    let qualifications = late_qualifications(statements);
    if qualifications.is_empty() {
        return Vec::new();
    }
    let mut defines = HashMap::default();
    for statement in statements {
        if let Some(Ok(Directive::Define { name, expression })) =
            parse_directive_at(statement.text.trim(), statement.at)
        {
            defines.insert(name, expression);
        }
    }
    let related = qualifications
        .into_iter()
        .map(|(_, related)| related)
        .collect::<Vec<_>>();
    let mut found = Vec::new();
    for statement in statements {
        // What the statement is, its expressions, and the names it may take
        // as decision series: a procedure's `decisions` parameters.
        let (what, expressions, series) =
            match parse_directive_at(statement.text.trim(), statement.at) {
                Some(Ok(Directive::Rule(rule))) => {
                    let mut expressions = vec![rule.condition.clone()];
                    expressions.extend(effect_expressions(&rule.effect).into_iter().cloned());
                    ("rule", expressions, Vec::new())
                }
                Some(Ok(Directive::Procedure(procedure))) => {
                    let mut expressions = Vec::new();
                    for step in &procedure.body {
                        expressions.push(step.condition.clone());
                        expressions.extend(effect_expressions(&step.effect).into_iter().cloned());
                    }
                    let series = procedure
                        .symbol_parameters
                        .into_iter()
                        .filter(|parameter| parameter.kind == "decisions")
                        .map(|parameter| parameter.name)
                        .collect();
                    ("procedure", expressions, series)
                }
                Some(Ok(Directive::Binding(binding))) => {
                    let mut expressions = vec![binding.condition];
                    if let BindingExpression::Expression(value) = binding.value {
                        expressions.push(value);
                    }
                    expressions.extend(binding.because.into_iter().flatten());
                    ("binding", expressions, Vec::new())
                }
                _ => continue,
            };
        let mut records = BTreeSet::new();
        for expression in &expressions {
            records_through(expression, &defines, &mut HashSet::default(), &mut records);
        }
        let reads = records
            .into_iter()
            .filter(|(form, name)| {
                matches!(*form, "committed" | "reopened")
                    || session.decision_series.contains_key(name)
                    || series.contains(name)
            })
            .map(|(form, name)| {
                let read = if form == "fold" {
                    format!("a fold of `{name}`")
                } else if form == "history_at" {
                    format!("`history_at({name}, …)`")
                } else {
                    format!("`{form}({name})`")
                };
                (name, read)
            })
            .collect::<Vec<_>>();
        if reads.is_empty() {
            continue;
        }
        let decisions = reads
            .iter()
            .map(|(name, _)| format!("`{name}`"))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let through = reads
            .iter()
            .map(|(_, read)| read.as_str())
            .collect::<Vec<_>>();
        found.push(Diagnostic {
            code: RECORD_READS_LATE_CAVEAT.0,
            name: RECORD_READS_LATE_CAVEAT.1,
            severity: "warning",
            line: statement.at.line,
            column: statement.at.column,
            message: format!(
                "this {what} reads the record of {} through {}, and the program qualifies evidence later: from 1.1 a value set this way reads the evidence that record lists live, so a later `qualify` of that evidence adds its caveat to the value, whenever the value was set (D015). A 1.0.x runtime did not add it",
                decisions.join(", "),
                through.join(", ")
            ),
            suggestion: format!(
                "Nothing needs to change if a value read through a decision should carry what its evidence is qualified with later. To find values a 1.0.x runtime saved without such a caveat, check each save with `caveat check --records --save SAVE PROGRAM`. To keep this read without the warning, put `# {ALLOW} {}` on the line above.",
                RECORD_READS_LATE_CAVEAT.1
            ),
            related: related.clone(),
        });
    }
    found
}

/// A value a session holds without a late caveat that evidence its lineage
/// lists now carries through a live relation: a caveat some `qualify` names,
/// whose relation has not been taken back. A 1.0.x runtime saved a value
/// read through a decision's record this way; 1.1 reads the evidence a
/// record lists live (D015), so a session 1.1 produced holds none. See
/// spec/caveat-check-0.1.md, C009.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LateCaveatGap {
    /// `state` for a state's value, `grounds` for its grounds, `binding` for
    /// a shown binding.
    pub place: &'static str,
    /// The state, or `TARGET.PROPERTY` for a binding.
    pub name: String,
    /// The caveat the value lacks.
    pub caveat: String,
    /// The evidence its lineage or grounds lists that the caveat qualifies.
    pub evidence: Vec<String>,
}

impl ReactiveSession {
    /// Every state's value and grounds, and every shown binding, that lists
    /// evidence a late caveat qualifies through a live relation while
    /// lacking that caveat, by state or binding name. Restore keeps a saved
    /// value as it was saved, so this is how to find, after restoring a save
    /// a 1.0.x runtime wrote, the values that 1.1 would compute with a
    /// caveat they lack (spec/caveat-caveat-sources-0.1.md).
    pub fn late_caveat_gaps(&self) -> Vec<LateCaveatGap> {
        let names = self
            .symbols
            .iter()
            .map(|(name, id)| (*id, name.as_str()))
            .collect::<HashMap<NodeId, &str>>();
        let mut gaps = Vec::new();
        let mut look = |place: &'static str, name: &str, provenance: &Provenance| {
            let mut lacking: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
            for (caveat, evidence) in self.live_late_relations(provenance) {
                lacking
                    .entry(names[&caveat])
                    .or_default()
                    .insert(names[&evidence].to_string());
            }
            for (caveat, evidence) in lacking {
                gaps.push(LateCaveatGap {
                    place,
                    name: name.to_string(),
                    caveat: caveat.to_string(),
                    evidence: evidence.into_iter().collect(),
                });
            }
        };
        for name in self.states.names().collect::<BTreeSet<_>>() {
            let cell = self.states.get(name).expect("a declared state");
            look("state", name, &cell.value.provenance);
            look("grounds", name, &cell.grounds);
        }
        for (target, properties) in &self.bindings {
            for property in properties.keys() {
                if let Some(lineage) = self
                    .binding_qualifications
                    .get(target)
                    .and_then(|shown| shown.get(property))
                {
                    look("binding", &format!("{target}.{property}"), lineage);
                }
            }
        }
        gaps
    }
}

/// C009. Each state and shown binding of a restored save that lacks a late
/// caveat evidence it lists now carries (`late_caveat_gaps`), at the state's
/// declaration or the binding's first `bind`.
fn saved_without_late_caveats(
    saved: &ReactiveSession,
    statements: &[Statement],
) -> Vec<Diagnostic> {
    let gaps = saved.late_caveat_gaps();
    if gaps.is_empty() {
        return Vec::new();
    }
    let mut declared: HashMap<String, Position> = HashMap::default();
    for statement in statements {
        match parse_directive_at(statement.text.trim(), statement.at) {
            Some(Ok(Directive::State { name, .. })) => {
                declared.entry(name).or_insert(statement.at);
            }
            Some(Ok(Directive::Binding(binding))) => {
                declared
                    .entry(format!("{}.{}", binding.target, binding.property))
                    .or_insert(statement.at);
            }
            _ => {}
        }
    }
    let qualifications = late_qualifications(statements);
    // One warning for each state or binding, in the order found.
    let mut by_value: Vec<(&str, &str, Vec<&LateCaveatGap>)> = Vec::new();
    for gap in &gaps {
        let what = if gap.place == "binding" {
            "binding"
        } else {
            "state"
        };
        match by_value
            .iter_mut()
            .find(|(kind, name, _)| *kind == what && *name == gap.name)
        {
            Some((_, _, list)) => list.push(gap),
            None => by_value.push((what, &gap.name, vec![gap])),
        }
    }
    by_value
        .into_iter()
        .map(|(what, name, list)| {
            let at = declared.get(name).copied().unwrap_or(Position { line: 1, column: 1 });
            let mut lacking: Vec<(String, Vec<&str>)> = Vec::new();
            for gap in &list {
                let place = match gap.place {
                    "state" => "value",
                    "grounds" => "grounds",
                    _ => "lineage",
                };
                let evidence = gap
                    .evidence
                    .iter()
                    .map(|evidence| format!("`{evidence}`"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let text = format!("`{}`, which qualifies {evidence}", gap.caveat);
                match lacking.iter_mut().find(|(known, _)| *known == text) {
                    Some((_, places)) => places.push(place),
                    None => lacking.push((text, vec![place])),
                }
            }
            let lacking = lacking
                .into_iter()
                .map(|(text, places)| format!("{text} (in its {})", places.join(" and ")))
                .collect::<Vec<_>>();
            let caveats = list.iter().map(|gap| gap.caveat.as_str()).collect::<BTreeSet<_>>();
            Diagnostic {
                code: SAVE_LACKS_LATE_CAVEAT.0,
                name: SAVE_LACKS_LATE_CAVEAT.1,
                severity: "warning",
                line: at.line,
                column: at.column,
                message: format!(
                    "the save holds the {what} `{name}` without {}: the evidence is listed and the caveat qualifies it through a live relation. A 1.0.x runtime saved a value read through a decision's record this way; 1.1 reads that evidence live, so the same value computed now carries the caveat (D015)",
                    lacking.join("; ")
                ),
                suggestion: "Restore keeps a saved value as it was saved and does not add the caveat. Replaying the session's events on this runtime gives the value the caveat, and so does any event that sets it again.".into(),
                related: qualifications
                    .iter()
                    .filter(|(caveat, _)| caveats.contains(caveat.as_str()))
                    .map(|(_, related)| related.clone())
                    .collect(),
            }
        })
        .collect()
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
            // A hold reopens nothing (spec/caveat-holds-0.1.md).
            Effect::Reopen { action, form, .. } if *form != DecisionChange::Hold => {
                reopened.insert(action.clone());
            }
            Effect::Commit {
                action,
                using,
                citing,
                ..
            } if session.decision_series.contains_key(action) => {
                let mut histories = BTreeSet::new();
                condition.collect_histories(&mut histories);
                using
                    .iter()
                    .chain(citing)
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
        // An irreversible series holds one revision by declaration, so a
        // reopening path would never revise it (spec/caveat-foreclosure-0.1.md).
        .filter(|(series, streams)| {
            !streams.is_empty()
                && !reopened.contains(series)
                && !session.irreversible.contains_key(series)
        })
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

/// One rule or procedure step as written: its event (none for a procedure
/// step), guard and effect, at its statement.
struct Step {
    event: Option<String>,
    condition: Expr,
    effect: Effect,
    at: Position,
}

/// The names `expression` reads (`Expr::collect_sources`), itself or through
/// the defines it reads.
fn sources_through<'d>(
    expression: &Expr,
    defines: &'d HashMap<String, Expr>,
    seen: &mut HashSet<&'d str>,
    sources: &mut BTreeSet<String>,
) {
    expression.collect_sources(sources);
    let mut reads = Reads::default();
    expression.collect_reads(&mut reads);
    for read in &reads.names {
        if let Some((define, value)) = defines.get_key_value(read.as_str()) {
            if seen.insert(define.as_str()) {
                sources_through(value, defines, seen, sources);
            }
        }
    }
}

/// `Expr::collect_withdrawal_reads`, itself or through the defines it reads.
fn withdrawal_reads_through<'d>(
    expression: &Expr,
    defines: &'d HashMap<String, Expr>,
    seen: &mut HashSet<&'d str>,
    tested: &mut BTreeSet<String>,
    bases: &mut BTreeSet<String>,
) {
    expression.collect_withdrawal_reads(tested, bases);
    let mut reads = Reads::default();
    expression.collect_reads(&mut reads);
    for read in &reads.names {
        if let Some((define, value)) = defines.get_key_value(read.as_str()) {
            if seen.insert(define.as_str()) {
                withdrawal_reads_through(value, defines, seen, tested, bases);
            }
        }
    }
}

/// C010 and C011, asked for with `withdrawals`. Compared by name after
/// `for` blocks and defines are expanded, as C006 and C008 are.
///
/// A name is *withdrawable* when some `withdraw` names it: evidence, an
/// occurrence of it, or a reading stream; `withdraw every` names the evidence
/// and each stream read from it. A state or stream *carries* the
/// names its `set` or `sample` values read, and what those carry in turn. A
/// commit *reads* what its `using`, `citing` and retention reasons name, and
/// what those carry.
///
/// C010: a commit reads a withdrawable name, and neither its own guard and
/// grounds nor a `reject` on its event asks whether that name, or something
/// that carries it, is withdrawn. After the withdrawal the commit records a
/// decision on withdrawn grounds.
///
/// C011: a decision series is committed on a withdrawable name, and no
/// `reopen` of it asks about its basis (`rests_on_withdrawn` or `in_basis`)
/// or runs on an event that withdraws that name. A decision made before the
/// withdrawal stays in force after it.
fn withdrawal_checks(statements: &[Statement]) -> Vec<Diagnostic> {
    let mut defines = HashMap::default();
    let mut declared = HashMap::default();
    let mut streams_from: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut steps = Vec::new();
    for statement in statements {
        match parse_directive_at(statement.text.trim(), statement.at) {
            Some(Ok(Directive::Define { name, expression })) => {
                defines.insert(name, expression);
            }
            Some(Ok(Directive::Decisions { name, .. })) => {
                declared.insert(name, statement.at);
            }
            Some(Ok(Directive::Readings { name, template, .. })) => {
                streams_from.entry(template).or_default().push(name);
            }
            Some(Ok(Directive::Rule(rule))) => steps.push(Step {
                event: Some(rule.event),
                condition: rule.condition,
                effect: rule.effect,
                at: statement.at,
            }),
            Some(Ok(Directive::Procedure(procedure))) => {
                steps.extend(procedure.body.into_iter().map(|step| Step {
                    event: None,
                    condition: step.condition,
                    effect: step.effect,
                    at: statement.at,
                }))
            }
            _ => {}
        }
    }
    let sources = |expression: &Expr| {
        let mut found = BTreeSet::new();
        sources_through(expression, &defines, &mut HashSet::default(), &mut found);
        found
    };
    let withdrawal_reads = |expression: &Expr| {
        let (mut tested, mut bases) = (BTreeSet::new(), BTreeSet::new());
        withdrawal_reads_through(
            expression,
            &defines,
            &mut HashSet::default(),
            &mut tested,
            &mut bases,
        );
        (tested, bases)
    };

    // Each withdrawable name, with the events that withdraw it.
    let mut withdrawn_on: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut carries: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for step in &steps {
        match &step.effect {
            Effect::Withdraw { target, .. } => {
                for name in withdrawn_names(target, &streams_from) {
                    let events = withdrawn_on.entry(name.clone()).or_default();
                    events.extend(step.event.clone());
                }
            }
            Effect::Set { name, value, .. } => {
                carries
                    .entry(name.clone())
                    .or_default()
                    .extend(sources(value));
            }
            Effect::Sample { stream, value, .. } => {
                carries
                    .entry(stream.clone())
                    .or_default()
                    .extend(sources(value));
            }
            _ => {}
        }
    }
    if withdrawn_on.is_empty() {
        return Vec::new();
    }
    let closure = |start: BTreeSet<String>| {
        let mut reached = BTreeSet::new();
        let mut pending = start.into_iter().collect::<Vec<_>>();
        while let Some(name) = pending.pop() {
            if reached.insert(name.clone()) {
                pending.extend(carries.get(&name).into_iter().flatten().cloned());
            }
        }
        reached
    };

    let mut found = Vec::new();
    // Each series, with the withdrawable names its commits read.
    let mut committed_on: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for step in &steps {
        let Effect::Commit {
            action,
            using,
            citing,
            reasons,
            ..
        } = &step.effect
        else {
            continue;
        };
        let mut read = BTreeSet::new();
        for expression in using
            .iter()
            .chain(citing)
            .chain(reasons.iter().map(|(_, _, read, _)| read))
        {
            read.extend(sources(expression));
        }
        let withdrawable = closure(read)
            .into_iter()
            .filter(|name| withdrawn_on.contains_key(name))
            .collect::<BTreeSet<_>>();
        if withdrawable.is_empty() {
            continue;
        }
        committed_on
            .entry(action.clone())
            .or_default()
            .extend(withdrawable.iter().cloned());

        // What this commit, or a refusal on its event, asks is withdrawn.
        let mut asked = withdrawal_reads(&step.condition).0;
        for expression in using.iter().chain(citing) {
            asked.extend(withdrawal_reads(expression).0);
        }
        if let Some(event) = &step.event {
            for other in &steps {
                if other.event.as_ref() == Some(event)
                    && matches!(other.effect, Effect::Reject { .. })
                {
                    asked.extend(withdrawal_reads(&other.condition).0);
                }
            }
        }
        let asked = closure(asked);
        let unguarded = withdrawable
            .into_iter()
            .filter(|name| !asked.contains(name))
            .collect::<Vec<_>>();
        if unguarded.is_empty() {
            continue;
        }
        let names = unguarded
            .iter()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>();
        let related = withdrawal_sites(&steps, &streams_from, &unguarded);
        let on = step
            .event
            .as_ref()
            .map(|event| format!(" on `{event}`"))
            .unwrap_or_default();
        found.push(Diagnostic {
            code: COMMIT_ON_WITHDRAWABLE.0,
            name: COMMIT_ON_WITHDRAWABLE.1,
            severity: "warning",
            line: step.at.line,
            column: step.at.column,
            message: format!(
                "this commit of `{action}`{on} reads {}, which the program can withdraw, and nothing on this commit or its event asks whether it is withdrawn: after the withdrawal, the commit records a decision whose grounds carry the `withdrawn` caveat",
                names.join(", ")
            ),
            suggestion: format!(
                "If a withdrawn source should not support a new decision, refuse it first, for example `on EVENT when withdrawn({0}) reject \"…\";` (for a stream, `withdrawn(latest({0}))`), or guard the commit with `not withdrawn({0})`. If committing on it is intended, put `# {ALLOW} {1}` on the line above.",
                unguarded[0],
                COMMIT_ON_WITHDRAWABLE.1
            ),
            related,
        });
    }

    for (series, names) in committed_on {
        let reopened = steps.iter().any(|step| match &step.effect {
            Effect::Reopen { action, form, .. }
                if action == &series && *form != DecisionChange::Hold =>
            {
                withdrawal_reads(&step.condition).1.contains(&series)
                    || step.event.as_ref().is_some_and(|event| {
                        names.iter().any(|name| withdrawn_on[name].contains(event))
                    })
            }
            _ => false,
        });
        if reopened {
            continue;
        }
        let at = declared
            .get(&series)
            .copied()
            .unwrap_or(Position { line: 1, column: 1 });
        let listed = names
            .iter()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>();
        let related = withdrawal_sites(
            &steps,
            &streams_from,
            &names.iter().cloned().collect::<Vec<_>>(),
        );
        found.push(Diagnostic {
            code: WITHDRAWAL_NOT_REOPENED.0,
            name: WITHDRAWAL_NOT_REOPENED.1,
            severity: "warning",
            line: at.line,
            column: at.column,
            message: format!(
                "`{series}` is committed on {}, which the program can withdraw, and no rule reopens it when that happens: a revision made before the withdrawal stays in force after it",
                listed.join(", ")
            ),
            suggestion: format!(
                "If a withdrawal should reconsider `{series}`, reopen it on the withdrawing event, for example `when committed({series}) and not reopened({series}) and rests_on_withdrawn({series}) reopen {series} because REASON`. If it should stay in force, put `# {ALLOW} {}` on the line above its declaration.",
                WITHDRAWAL_NOT_REOPENED.1
            ),
            related,
        });
    }
    found
}

/// The names a `withdraw` target withdraws. `every NAME` withdraws NAME and,
/// when NAME is evidence, each reading stream declared `from` it.
fn withdrawn_names<'a>(
    target: &'a EvidenceSelector,
    streams_from: &'a BTreeMap<String, Vec<String>>,
) -> Vec<&'a String> {
    match target {
        EvidenceSelector::Named(name) | EvidenceSelector::Latest(name) => vec![name],
        EvidenceSelector::Occurrence { evidence, .. } => vec![evidence],
        EvidenceSelector::Caveated { state, .. } => vec![state],
        EvidenceSelector::Every(name) => std::iter::once(name)
            .chain(streams_from.get(name).into_iter().flatten())
            .collect(),
    }
}

/// Where each of `names` is withdrawn, as related locations, each once.
fn withdrawal_sites(
    steps: &[Step],
    streams_from: &BTreeMap<String, Vec<String>>,
    names: &[String],
) -> Vec<Related> {
    let mut sites = Vec::new();
    for name in names {
        for step in steps {
            let Effect::Withdraw { target, .. } = &step.effect else {
                continue;
            };
            let note = format!("`{name}` is withdrawn here");
            if withdrawn_names(target, streams_from).contains(&name)
                && !sites
                    .iter()
                    .any(|site: &Related| site.line == step.at.line && site.note == note)
            {
                sites.push(Related {
                    line: step.at.line,
                    column: step.at.column,
                    note,
                });
            }
        }
    }
    sites
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
    let mut program_defines = HashMap::default();
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
    let mut kind_defines: HashMap<&str, Vec<(String, Expr)>> = HashMap::default();
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
    let mut declared_at = HashMap::default();
    for statement in statements.iter().filter(|statement| statement.repeated) {
        if let ["entity", name, "kind", _, "at", _] =
            statement.text.split_whitespace().collect::<Vec<_>>()[..]
        {
            declared_at.entry(name).or_insert(statement.at);
        }
    }
    // The number each define of the program comes to, once read.
    let known = RefCell::new(HashMap::default());

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
        let mut numberings = HashMap::default();

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
            operands(
                &written.condition,
                Expr::conjuncts,
                &defines,
                &mut Vec::new(),
                &mut conjuncts,
            );
            // Each top-level conjunct as its disjuncts, the conjunct itself
            // where it is no chain of `or`. With each disjunct, the (P, E) it
            // compares, `P == E` or `E == P`, where E reads `$index`, itself
            // or through a define.
            let alternatives = conjuncts
                .iter()
                .map(|conjunct| {
                    let mut disjuncts = Vec::new();
                    operands(
                        conjunct,
                        Expr::disjuncts,
                        &defines,
                        &mut Vec::new(),
                        &mut disjuncts,
                    );
                    disjuncts
                        .into_iter()
                        .map(|disjunct| {
                            let by_index = disjunct
                                .equality()
                                .into_iter()
                                .flat_map(|(left, right)| [(left, right), (right, left)])
                                .filter_map(|(parameter, value)| {
                                    Some((parameter.as_name()?, value))
                                })
                                .filter(|(_, value)| {
                                    reads_through(
                                        value,
                                        &index_marker,
                                        &defines,
                                        &mut HashSet::default(),
                                    )
                                })
                                .collect::<Vec<_>>();
                            (disjunct, by_index)
                        })
                        .collect::<Vec<_>>()
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
                // The parameter a comparison `P == E` or `E == P` compares
                // with a value `accepts` takes.
                let compared = |comparison: &Expr, accepts: &dyn Fn(&Expr) -> bool| {
                    let (left, right) = comparison.equality()?;
                    [(left, right), (right, left)]
                        .into_iter()
                        .find_map(|(parameter, value)| {
                            let name = parameter.as_name()?;
                            let declared =
                                parameters.iter().find(|declared| declared.name == name)?;
                            accepts(value).then_some(*declared)
                        })
                };
                // A conjunct that selects the member, as C003 reads it: a
                // selection, or a chain of `or` of selections.
                let selects = |disjuncts: &[(&Expr, Vec<(&str, &Expr)>)]| {
                    disjuncts
                        .iter()
                        .all(|(disjunct, _)| compared(disjunct, &routes).is_some())
                };
                // Each comparison with `$index` in a top-level conjunct that
                // selects by it, in the guard's order: the conjunct itself, or
                // each disjunct of a chain of `or` that selects the member.
                let selections = alternatives
                    .iter()
                    .flat_map(|disjuncts| match disjuncts.as_slice() {
                        [(_, compares)] => compared_by(compares, &parameters).into_iter().collect(),
                        _ if selects(disjuncts) => disjuncts
                            .iter()
                            .filter_map(|(_, compares)| compared_by(compares, &parameters))
                            .collect(),
                        _ => Vec::new(),
                    })
                    .collect::<Vec<_>>();
                // The number E comes to in the member's copy: `$index` is the
                // member's, and a name written with a binding is the copy's.
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
                // Each parameter and number already checked in this copy: the
                // same comparison twice is reported once.
                let mut checked = Vec::new();
                for (parameter, value) in selections {
                    let numbering = numberings
                        .entry((event.as_str(), parameter.name.as_str()))
                        .or_insert_with(|| numbering(parameter, block));
                    if let Some((numbering, number)) = numbering.as_ref().and_then(|numbering| {
                        Some((numbering, constant(value, &functions, &in_copy)?))
                    }) {
                        if checked.contains(&(parameter.name.as_str(), number)) {
                            continue;
                        }
                        checked.push((parameter.name.as_str(), number));
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
                if own_event || alternatives.iter().any(|disjuncts| selects(disjuncts)) {
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

/// The first (P, E) whose P is one of `parameters`, with that parameter.
fn compared_by<'p, 'e>(
    compares: &[(&str, &'e Expr)],
    parameters: &[&'p Parameter],
) -> Option<(&'p Parameter, &'e Expr)> {
    compares.iter().find_map(|(name, value)| {
        let parameter = parameters.iter().find(|declared| declared.name == *name)?;
        Some((*parameter, *value))
    })
}

/// Whether an event parameter is declared `NAME kind KIND`.
fn of_kind(parameter: &Parameter, kind: &str) -> bool {
    matches!(&parameter.domain, ParameterDomain::Entity { kind: declared, .. } if declared == kind)
}

/// The operands of a chain, of `and` with `Expr::conjuncts` or of `or` with
/// `Expr::disjuncts`, each one that names a define replaced by the operands of
/// the same chain in its expression.
fn operands<'a>(
    condition: &'a Expr,
    chain: fn(&Expr) -> Vec<&Expr>,
    defines: &'a HashMap<String, Expr>,
    inlining: &mut Vec<&'a str>,
    out: &mut Vec<&'a Expr>,
) {
    for operand in chain(condition) {
        match operand
            .as_name()
            .and_then(|name| defines.get_key_value(name))
        {
            Some((name, expression)) if !inlining.contains(&name.as_str()) => {
                inlining.push(name);
                operands(expression, chain, defines, inlining, out);
                inlining.pop();
            }
            _ => out.push(operand),
        }
    }
}
