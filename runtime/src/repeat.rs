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

use crate::link::{
    blank_comments, block_braces, is_identifier_char, is_identifier_start, statement_words,
    statements_of,
};
use crate::parser::position_of;

/// Expand every `for` block in one source. A source with no `for` block is
/// returned unchanged, so an existing program expands to itself byte for byte.
pub fn expand(source: &str) -> Result<String, String> {
    // A last block without its `;` is a block too: the loader reads a last
    // statement without one.
    let statements = statements_of(source);
    let blocks: Vec<(usize, usize)> = statements
        .iter()
        .copied()
        .filter(|(start, end)| statement_words(&source[*start..*end]).first() == Some(&"for"))
        .collect();
    if blocks.is_empty() {
        return Ok(source.to_string());
    }
    let members = entity_kinds(source, &statements);
    let part = Part {
        source,
        statements: &statements,
        kinds: &members,
    };

    let mut out = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end) in blocks {
        out.push_str(&source[cursor..start]);
        out.push_str(&expand_block((start, end), &part)?);
        cursor = end;
    }
    out.push_str(&source[cursor..]);
    Ok(out)
}

/// Members of each entity kind in one source, in declaration order.
pub(crate) fn declared_kinds(source: &str) -> Vec<(String, Vec<String>)> {
    entity_kinds(source, &statements_of(source))
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

/// Members of each entity kind, in declaration order: every top-level
/// `entity` statement the loader declares, among `statements`, which hold a
/// last statement without its `;` too. Each is read as the loader reads it,
/// comments blanked, whatever its name, as a `kind` parameter counts it, so
/// `$index` numbers the members the parameter numbers.
fn entity_kinds(source: &str, statements: &[(usize, usize)]) -> Vec<(String, Vec<String>)> {
    let mut kinds: Vec<(String, Vec<String>)> = Vec::new();
    for (start, end) in statements.iter().copied() {
        let Some((name, kind)) = entity_of(&source[start..end]) else {
            continue;
        };
        match kinds.iter_mut().find(|(declared, _)| *declared == kind) {
            Some((_, members)) => members.push(name),
            None => kinds.push((kind, vec![name])),
        }
    }
    kinds
}

fn expand_block(span: (usize, usize), part: &Part) -> Result<String, String> {
    let block = read_block(part.source, span, part.kinds)?;
    // Which rules are routed is read from the body as written, once.
    let routed = match block.route {
        Some(parameter) => Routing {
            header: block.header.join(" "),
            kind: block.kind,
            binding: block.binding,
            parameter,
        }
        .rules(block.body, part)?,
        None => Vec::new(),
    };

    let mut out = String::new();
    for (position, member) in block.members.iter().enumerate() {
        let index = (position + 1).to_string();
        let bindings = [(block.name, member.as_str()), ("index", index.as_str())];
        match block.route {
            None => out.push_str(&substitute(block.body, &bindings)?),
            Some(parameter) => out.push_str(&routed_copy(
                block.body,
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

/// A `for` block whose header, binding, body and kind have passed Repetition
/// 0.1's checks, and a routed header's.
struct Block<'s, 'k> {
    header: Vec<&'s str>,
    kind: &'s str,
    /// As written, such as `$p`.
    binding: &'s str,
    /// The binding without its `$`.
    name: &'s str,
    route: Option<&'s str>,
    body: &'s str,
    members: &'k [String],
}

/// The block statement at `start..end` of a part's `source`.
fn read_block<'s, 'k>(
    source: &'s str,
    (start, end): (usize, usize),
    kinds: &'k [(String, Vec<String>)],
) -> Result<Block<'s, 'k>, String> {
    let statement = &source[start..end];
    let braces = block_braces(statement);
    let open = braces
        .open
        .ok_or_else(|| format!("for block has no body: {}", head(statement)))?;
    // A brace in unquoted text pairs only within its statement, so a `{` a
    // body statement leaves open is refused, not paired with a `}` in a later
    // statement, which would end the body there instead.
    if let Some(brace) = braces.unclosed {
        return Err(unclosed_brace(source, start, statement, open, brace));
    }
    let close = braces
        .close
        .ok_or_else(|| format!("for block is not closed: {}", head(statement)))?;
    // The body's copies replace the whole statement, so anything but
    // comments between the body and the block's end would be lost.
    let after = without_terminator(&statement[close + 1..]);
    if !blank_comments(after).trim().is_empty() {
        return Err(format!(
            "for block has text after its body: {}; end the block with `}};`",
            head(after)
        ));
    }
    let header = statement_words(&statement[..open]);
    let (kind, binding, route) = match header.as_slice() {
        ["for", kind, "as", binding] => (*kind, *binding, None),
        // The fifth word says a routed block was meant. Its header is
        // `for KIND as $NAME routed by P`, or error 1 of section 7.
        [_, _, _, _, "routed", ..] => match header.as_slice() {
            ["for", kind, "as", binding, "routed", "by", parameter]
                if binding.strip_prefix('$').is_some_and(plain_name) && plain_name(parameter) =>
            {
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
    // A nested block is a statement that begins with `for`, the body's last
    // statement without its `;` too. The word inside a comment or quoted
    // text is not one.
    if statements_of(body)
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
    Ok(Block {
        header,
        kind,
        binding,
        name,
        route,
        body,
        members,
    })
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
    /// has passed: an entity of KIND in a `for` block, which `$index` does
    /// not count, then each rule in the order written, then whether any rule
    /// is routed. A rule is routed from its event alone; what the rule does
    /// is not read.
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
        // The parser reads a last statement without its `;`, so it is a rule
        // written in the body like any other.
        for (start, end) in statements_of(body) {
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
/// and its entities. Nothing from another part.
struct Part<'a> {
    source: &'a str,
    /// The part's statements as the loader reads them: each `;`-terminated
    /// one, and a last one without its `;`.
    statements: &'a [(usize, usize)],
    kinds: &'a [(String, Vec<String>)],
}

impl Part<'_> {
    /// Every event the part declares, each parameter with the words declared
    /// after its name. A declaration a `for` block writes counts once
    /// expanded: only a block whose body Repetition 0.1 expands for every
    /// member counts. A block that does not expand is left out here and
    /// reports its own error when it is expanded.
    fn events(&self) -> Vec<(String, Vec<(String, String)>)> {
        let mut events = Vec::new();
        for (start, end) in self.statements {
            let text = &self.source[*start..*end];
            if statement_words(text).first() != Some(&"for") {
                events.extend(event_declaration(text));
                continue;
            }
            let Some(copies) = self.copies((*start, *end)) else {
                continue;
            };
            for copy in &copies {
                for (inner, inner_end) in statements_of(copy) {
                    events.extend(event_declaration(&copy[inner..inner_end]));
                }
            }
        }
        events
    }

    /// Each member's copy of a `for` block's body, as Repetition 0.1 expands
    /// it. None when the block does not expand.
    fn copies(&self, span: (usize, usize)) -> Option<Vec<String>> {
        let block = read_block(self.source, span, self.kinds).ok()?;
        block
            .members
            .iter()
            .enumerate()
            .map(|(position, member)| {
                let index = (position + 1).to_string();
                substitute(
                    block.body,
                    &[(block.name, member.as_str()), ("index", index.as_str())],
                )
                .ok()
            })
            .collect()
    }

    /// The first entity of `kind` that a `for` block declares, by its name as
    /// written there. `$index` does not count it, and a `P kind KIND`
    /// parameter does.
    fn entity_in_block(&self, kind: &str) -> Option<String> {
        self.statements.iter().find_map(|(start, end)| {
            let text = &self.source[*start..*end];
            if statement_words(text).first() != Some(&"for") {
                return None;
            }
            let body = body_of(text)?;
            statements_of(body)
                .into_iter()
                .find_map(|(inner, inner_end)| {
                    entity_of(&body[inner..inner_end])
                        .filter(|(_, declared)| declared == kind)
                        .map(|(name, _)| name)
                })
        })
    }
}

/// The name and kind an `entity` statement declares, read as the parser
/// reads it: comments blanked, then split at whitespace (parser.rs
/// `parse_statement`).
fn entity_of(statement: &str) -> Option<(String, String)> {
    let code = blank_comments(without_terminator(statement));
    match code.split_whitespace().collect::<Vec<_>>()[..] {
        ["entity", name, "kind", kind, "at", _] => Some((name.to_string(), kind.to_string())),
        _ => None,
    }
}

/// A block statement's body, between its first `{` and its last `}`.
fn body_of(statement: &str) -> Option<&str> {
    let (open, close) = body_braces(statement);
    let (open, close) = (open?, close?);
    (open < close).then(|| &statement[open + 1..close])
}

/// Where a block statement's body opens and closes, as the statement reader
/// reads them (`link::block_braces`): its first `{`, and the first `}` after
/// it outside quoted text and comments that closes no `{` earlier in its own
/// statement.
pub(crate) fn body_braces(statement: &str) -> (Option<usize>, Option<usize>) {
    let braces = block_braces(statement);
    (braces.open, braces.close)
}

/// The refusal of the `{` at `brace` in the body of `statement`, a block
/// statement at `start` of `source`, which the body statement holding it
/// does not close. `open` is where the body opens.
fn unclosed_brace(
    source: &str,
    start: usize,
    statement: &str,
    open: usize,
    brace: usize,
) -> String {
    let body = &statement[open + 1..];
    let inner = brace - open - 1;
    let written = statements_of(body)
        .into_iter()
        .find(|(from, to)| (*from..*to).contains(&inner))
        .map_or_else(String::new, |(from, to)| {
            statement_words(&body[from..to]).join(" ")
        });
    position_of(source, start + brace).error(format!(
        "for block `{}`: `{written}` has a `{{` that its statement does not close; a brace in unquoted text pairs only within its statement, so quote the text",
        statement_words(&statement[..open]).join(" ")
    ))
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
    // The loader reads the copy once linked, with `glow::level` written
    // `glow__level`, so the guard's end and its `or` are found in that text.
    let code = as_linked(&blank_comments(without_terminator(copy)));
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

/// A statement, comments already blanked, with each `WORD::SYMBOL` outside
/// quoted text written `WORD__SYMBOL`, as linking rewrites it (link.rs
/// `rewrite`). Both are two bytes, so a word's range in it is its range in
/// the statement.
fn as_linked(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let (mut quoted, mut escaped, mut word) = (false, false, false);
    let mut rest = code;
    while let Some(ch) = rest.chars().next() {
        if !quoted && word && rest.starts_with("::") {
            // The symbol after `::` is consumed with it, as linking does.
            let symbol = rest[2..]
                .find(|next: char| !is_identifier_char(next))
                .map_or(rest.len(), |end| end + 2);
            out.push_str("__");
            out.push_str(&rest[2..symbol]);
            rest = &rest[symbol..];
            word = false;
            continue;
        }
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
        } else {
            quoted = ch == '"';
            word = if word {
                is_identifier_char(ch)
            } else {
                is_identifier_start(ch)
            };
        }
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
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

/// The first words of a statement, its comments read as whitespace.
fn head(statement: &str) -> String {
    blank_comments(statement)
        .split_whitespace()
        .take(5)
        .collect::<Vec<_>>()
        .join(" ")
}
