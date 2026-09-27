//! Repetition 0.1: `for KIND as $NAME { ... };`
//!
//! A source-to-source pre-pass that expands a block once per member of a
//! declared entity kind. See spec/caveat-repetition-0.1.md. Nothing here adds
//! runtime semantics: the output is ordinary statements the author could have
//! written by hand, which is what keeps order — the thing that decides which
//! `on` rule fires first and which `bind` wins — the author's.
//!
//! Routed Repetition 0.1 adds `for KIND as $NAME routed by P { ... };`. Each
//! member's copy of a rule on an event that names a member by P gets the
//! selection `P == $index` at the front of its guard. See
//! spec/caveat-routed-repetition-0.1.md. The copy is the text a hand-routed
//! block expands to, so routing adds no runtime semantics either.

use crate::link::{is_identifier_char, is_identifier_start, statement_spans, statement_words};

/// Expand every `for` block in one source. A source with no `for` block is
/// returned unchanged, so an existing program expands to itself byte for byte.
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
    let part = Part {
        source,
        spans: &spans,
        kinds: &members,
    };

    let mut out = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end) in blocks {
        out.push_str(&source[cursor..start]);
        out.push_str(&expand_block(&source[start..end], &part)?);
        cursor = end;
    }
    out.push_str(&source[cursor..]);
    Ok(out)
}

/// Members of each entity kind in one source, in declaration order.
pub(crate) fn declared_kinds(source: &str) -> Vec<(String, Vec<String>)> {
    entity_kinds(source, &statement_spans(source))
}

/// A block statement with `$NAME` replaced by `member` and `$index` by
/// `index`: the same substitution `expand` applies to the whole body, where
/// `index` is the member's position from 1.
pub(crate) fn instantiate(
    template: &str,
    binding: &str,
    member: &str,
    index: &str,
) -> Result<String, String> {
    substitute(template, &[(binding, member), ("index", index)])
}

/// Members of each entity kind, in declaration order.
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

fn expand_block(statement: &str, part: &Part) -> Result<String, String> {
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
    let (kind, binding, route) = match header.as_slice() {
        ["for", kind, "as", binding] => (*kind, *binding, None),
        // The fifth word says a routed block was meant.
        [_, _, _, _, "routed", ..] => match header.as_slice() {
            ["for", kind, "as", binding, "routed", "by", parameter] if plain_name(parameter) => {
                (*kind, *binding, Some(*parameter))
            }
            _ => {
                return Err(format!(
                    "expected `for KIND as $NAME routed by PARAM {{ ... }}`, found: {}",
                    header.join(" ")
                ))
            }
        },
        _ => {
            return Err(format!(
                "expected `for KIND as $NAME {{ ... }}`, found: {}",
                head(statement)
            ))
        }
    };
    let Some(name) = binding.strip_prefix('$') else {
        return Err(format!("{binding} is not a binding; write $name"));
    };
    check_binding(name)?;
    if name == "index" {
        return Err("$index is provided by the block and cannot be rebound".into());
    }

    let body = &statement[open + 1..close];
    // A nested block is a statement that begins with `for`. The word inside a
    // comment or quoted text is not one.
    if statement_spans(body)
        .into_iter()
        .any(|(start, end)| statement_words(&body[start..end]).first() == Some(&"for"))
    {
        return Err(format!(
            "for blocks do not nest; see spec/caveat-repetition-0.1.md section 4: {}",
            head(statement)
        ));
    }
    let Some((_, members)) = part.kinds.iter().find(|(declared, _)| declared == kind) else {
        return Err(format!(
            "no entity is declared `kind {kind}`, so `for {kind}` has nothing to expand"
        ));
    };
    // Which rules are routed is read from the body as written, once.
    let routed = match route {
        Some(parameter) => Routing {
            header: header.join(" "),
            kind,
            binding,
            parameter,
        }
        .rules(body, part)?,
        None => Vec::new(),
    };

    let mut out = String::new();
    for (position, member) in members.iter().enumerate() {
        let index = (position + 1).to_string();
        let bindings = [(name, member.as_str()), ("index", index.as_str())];
        match route {
            None => out.push_str(&substitute(body, &bindings)?),
            Some(parameter) => out.push_str(&routed_copy(
                body,
                &routed,
                &bindings,
                &format!("{parameter} == {index}"),
            )?),
        }
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }
    Ok(out)
}

/// A routed block's header and what it names, for its checks and errors.
/// `header` is HEADER in spec/caveat-routed-repetition-0.1.md section 7.
struct Routing<'a> {
    header: String,
    kind: &'a str,
    binding: &'a str,
    parameter: &'a str,
}

impl Routing<'_> {
    /// The routed rules of the body, by span, once every check of section 7
    /// has passed: an entity of KIND in a `for` block, then each rule in the
    /// order written, then whether any rule is routed. A rule is routed from
    /// its event alone; what the rule does is not read.
    fn rules(&self, body: &str, part: &Part) -> Result<Vec<(usize, usize)>, String> {
        let Routing {
            header,
            kind,
            binding,
            parameter,
        } = self;
        if let Some(entity) = part.entity_in_block(kind) {
            return Err(format!(
                "`{header}`: `$index` does not count entity `{entity}` of kind {kind}, declared in a for block, but `{parameter}` does; declare it at the top level of this part"
            ));
        }
        let events = part.events();
        let member = format!("kind {kind}");
        let mut routed = Vec::new();
        for (start, end) in statement_spans(body) {
            let code = blank_comments(without_terminator(&body[start..end]));
            let ["on", event, ..] = words(&code)[..] else {
                continue;
            };
            // An event written with `$NAME` or `$index` is the member's own,
            // so it already selects the member.
            if event.contains('$') {
                continue;
            }
            let Some((_, parameters)) = events.iter().find(|(declared, _)| declared == event)
            else {
                return Err(format!(
                    "`on {event}` in `{header}`: no event `{event}` is declared in this part, so the block cannot tell whether it names a {kind}"
                ));
            };
            match parameters.iter().find(|(name, _)| name == parameter) {
                Some((_, form)) if *form == member => {}
                Some((_, form)) => {
                    let declared = format!("{parameter} {form}");
                    return Err(format!(
                        "`on {event}` in `{header}`: `{event}` declares `{}`, not `{parameter} {member}`; keep the rules on `{event}` in a plain for block",
                        declared.trim_end()
                    ));
                }
                None => {
                    let others = parameters
                        .iter()
                        .filter(|(_, form)| *form == member)
                        .map(|(name, _)| format!("`{name}`"))
                        .collect::<Vec<_>>();
                    match others.as_slice() {
                        // The event names no member of the kind: every
                        // member's copy runs on it by design.
                        [] => continue,
                        [other] => {
                            return Err(format!(
                                "`on {event}` in `{header}`: `{event}` names a {kind} by {other}, not by `{parameter}`; route the block by {other}, or keep the rules on `{event}` in a plain for block"
                            ))
                        }
                        _ => {
                            return Err(format!(
                                "`on {event}` in `{header}`: `{event}` names a {kind} by {}, not by `{parameter}`; route the block by one of them, or keep the rules on `{event}` in a plain for block",
                                others.join(", ")
                            ))
                        }
                    }
                }
            }
            // Quoted text counts; a comment does not, as for C003.
            if !code.contains('$') {
                return Err(format!(
                    "`on {event}` in `{header}` mentions neither `{binding}` nor `$index`; its copies are all the same rule, so write it once, outside the block"
                ));
            }
            routed.push((start, end));
        }
        if routed.is_empty() {
            return Err(format!(
                "`{header}` routes no rule: no rule in it is on an event that declares `{parameter} {member}`"
            ));
        }
        Ok(routed)
    }
}

/// What routing reads from the part a block is in: its `event` declarations
/// and the entities its `for` blocks declare. Nothing from another part.
struct Part<'a> {
    source: &'a str,
    spans: &'a [(usize, usize)],
    kinds: &'a [(String, Vec<String>)],
}

impl<'a> Part<'a> {
    /// Every event the part declares, each parameter with the words declared
    /// after its name. A declaration a `for` block writes counts once
    /// expanded. A block that does not expand is left out here and reports
    /// its own error when it is expanded.
    fn events(&self) -> Vec<(String, Vec<(String, String)>)> {
        let mut events = Vec::new();
        for (start, end) in self.spans {
            let text = &self.source[*start..*end];
            if statement_words(text).first() != Some(&"for") {
                events.extend(event_declaration(text));
                continue;
            }
            let (Some(body), Some((binding, members))) = (body_of(text), self.block(text)) else {
                continue;
            };
            for (inner, inner_end) in statement_spans(body) {
                let template = &body[inner..inner_end];
                if words(&blank_comments(template)).first() != Some(&"event") {
                    continue;
                }
                for (position, member) in members.iter().enumerate() {
                    if let Ok(copy) =
                        instantiate(template, binding, member, &(position + 1).to_string())
                    {
                        events.extend(event_declaration(&copy));
                    }
                }
            }
        }
        events
    }

    /// The binding and members of a `for` block whose header reads as one.
    fn block<'s>(&self, text: &'s str) -> Option<(&'s str, &'a [String])> {
        let open = text.find('{')?;
        let (kind, binding) = match statement_words(&text[..open])[..] {
            ["for", kind, "as", binding] | ["for", kind, "as", binding, "routed", "by", _] => {
                (kind, binding)
            }
            _ => return None,
        };
        let (_, members) = self.kinds.iter().find(|(declared, _)| declared == kind)?;
        Some((binding.strip_prefix('$')?, members))
    }

    /// The first entity of `kind` that a `for` block declares, by its name as
    /// written there. `$index` does not count it, and a `P kind KIND`
    /// parameter does.
    fn entity_in_block(&self, kind: &str) -> Option<String> {
        self.spans.iter().find_map(|(start, end)| {
            let text = &self.source[*start..*end];
            if statement_words(text).first() != Some(&"for") {
                return None;
            }
            let body = body_of(text)?;
            statement_spans(body)
                .into_iter()
                .find_map(|(inner, inner_end)| {
                    let code = blank_comments(without_terminator(&body[inner..inner_end]));
                    match words(&code)[..] {
                        ["entity", name, "kind", declared, "at", _] if declared == kind => {
                            Some(name.to_string())
                        }
                        _ => None,
                    }
                })
        })
    }
}

/// A block statement's body, between its first `{` and its last `}`.
fn body_of(statement: &str) -> Option<&str> {
    let open = statement.find('{')?;
    let close = statement.rfind('}')?;
    (open < close).then(|| &statement[open + 1..close])
}

/// An `event` declaration's name and parameters, each parameter as its name
/// and the words declared after it, such as `kind pr` or `in failed passed`.
fn event_declaration(statement: &str) -> Option<(String, Vec<(String, String)>)> {
    let code = blank_comments(without_terminator(statement));
    let written = words(&code);
    let ["event", name, rest @ ..] = written.as_slice() else {
        return None;
    };
    let parameters = rest
        .join(" ")
        .split(',')
        .filter_map(|parameter| {
            let words = parameter.split_whitespace().collect::<Vec<_>>();
            let (name, form) = words.split_first()?;
            Some(((*name).to_string(), form.join(" ")))
        })
        .collect();
    Some(((*name).to_string(), parameters))
}

/// A routed block's body for one member: substituted as any block's body is,
/// with the route written into each copy of a routed rule.
fn routed_copy(
    body: &str,
    routed: &[(usize, usize)],
    bindings: &[(&str, &str); 2],
    route: &str,
) -> Result<String, String> {
    let mut out = String::with_capacity(body.len());
    let mut cursor = 0;
    for (start, end) in routed {
        out.push_str(&substitute(&body[cursor..*start], bindings)?);
        out.push_str(&with_route(
            &substitute(&body[*start..*end], bindings)?,
            route,
        ));
        cursor = *end;
    }
    out.push_str(&substitute(&body[cursor..], bindings)?);
    Ok(out)
}

/// One member's copy of a routed rule, with its route `P == N` written in
/// (spec section 3): `P == N and ` in front of the guard, around which `(`
/// and `)` go when it has an `or` outside parentheses, or ` when P == N`
/// after the event when the rule has no guard. Nothing else changes.
fn with_route(copy: &str, route: &str) -> String {
    let code = blank_comments(without_terminator(copy));
    let spans = crate::reactive::syntax_word_spans(&code);
    let word = |index: usize| spans.get(index).map(|(start, end)| &code[*start..*end]);
    if word(2) != Some("when") {
        // Only `on EVENT` rules are routed, so the event is the second word.
        let at = spans[1].1;
        return format!("{} when {route}{}", &copy[..at], &copy[at..]);
    }
    let Some(&(first, _)) = spans.get(3) else {
        // Nothing follows `when`: the copy cannot load, but gets its route.
        let at = spans[2].1;
        return format!("{} {route} and{}", &copy[..at], &copy[at..]);
    };
    // The guard ends where the loader finds the effect, found in this copy
    // the same way. Without a complete effect the copy cannot load, and the
    // route still goes at the front.
    let after_when = spans[2..]
        .iter()
        .map(|(start, end)| &code[*start..*end])
        .collect::<Vec<_>>();
    // G is read as the loader reads it: its words joined by single spaces.
    let guard = crate::reactive::complete_effect_at(&after_when)
        .filter(|effect| *effect > 1)
        .map(|effect| (after_when[1..effect].join(" "), spans[effect + 1].1));
    match guard {
        Some((guard, last)) if crate::reactive_expr::has_top_level_or(&guard) => format!(
            "{}{route} and ({}){}",
            &copy[..first],
            &copy[first..last],
            &copy[last..]
        ),
        _ => format!("{}{route} and {}", &copy[..first], &copy[first..]),
    }
}

/// A statement without the `;` that ends it.
fn without_terminator(statement: &str) -> &str {
    statement.strip_suffix(';').unwrap_or(statement)
}

/// A statement with each comment replaced by spaces, byte for byte, so a
/// word's range in it is its range in the statement. The loader reads a
/// statement with its comments blanked the same way (parser.rs
/// `scan_statements_at`).
fn blank_comments(statement: &str) -> String {
    let mut out = String::with_capacity(statement.len());
    let (mut quoted, mut escaped, mut comment) = (false, false, false);
    let mut chars = statement.chars().peekable();
    while let Some(ch) = chars.next() {
        if comment {
            if ch == '\n' {
                comment = false;
                out.push('\n');
            } else {
                out.push_str(&" ".repeat(ch.len_utf8()));
            }
        } else if quoted {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
        } else if ch == '#' || (ch == '/' && chars.peek() == Some(&'/')) {
            comment = true;
            out.push(' ');
        } else {
            quoted = ch == '"';
            out.push(ch);
        }
    }
    out
}

/// The words the loader splits a statement into, comments already blanked.
fn words(code: &str) -> Vec<&str> {
    crate::reactive::syntax_word_spans(code)
        .into_iter()
        .map(|(start, end)| &code[start..end])
        .collect()
}

/// Replace every `$binding` in the body. Substitution reaches inside quoted
/// text on purpose: the body is a template, and `evidence $r_reading from
/// "$r";` needs the member's name in both places.
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
        // `$r_reading` is the binding `r` glued to a suffix, so the longest
        // binding that prefixes the run wins, and the rest is kept.
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

/// A name with no `$` and no dots, as a binding's and a route's are.
fn plain_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(is_identifier_start) && chars.all(is_identifier_char)
}

fn check_binding(name: &str) -> Result<(), String> {
    if plain_name(name) {
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
