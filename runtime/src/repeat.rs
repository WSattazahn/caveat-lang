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
//!
//! Nested Repetition 0.1 lets a block's body hold blocks. A nested block is
//! expanded first, inside its enclosing block's body, with the enclosing
//! blocks' bindings left as written; the enclosing block then expands the
//! result. See spec/caveat-nested-repetition-0.1.md.

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
        out.push_str(&expand_block((start, end), &part, &[], true)?);
        cursor = end;
    }
    out.push_str(&source[cursor..]);
    Ok(out)
}

/// The body at `start` of the part's source, `length` bytes long, with each
/// block nested in it replaced by its copies. `scope` names the bindings of
/// the blocks the body is in, innermost last.
fn expand_nested(
    part: &Part,
    (start, length): (usize, usize),
    scope: &[&str],
    route: bool,
) -> Result<String, String> {
    let body = &part.source[start..start + length];
    let mut out = String::with_capacity(body.len());
    let mut cursor = 0;
    for (inner, inner_end) in statements_of(body) {
        if statement_words(&body[inner..inner_end]).first() != Some(&"for") {
            continue;
        }
        out.push_str(&body[cursor..inner]);
        out.push_str(&expand_block(
            (start + inner, start + inner_end),
            part,
            scope,
            route,
        )?);
        cursor = inner_end;
    }
    out.push_str(&body[cursor..]);
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
    substitute(template, &[(binding, member), ("index", index)], &[])
}

/// `text` with each binding in `bindings` replaced, and each binding named in
/// `kept` left as written: what a nested block's statement reads like before
/// its enclosing blocks expand it.
pub(crate) fn substitute_keeping(
    text: &str,
    bindings: &[(&str, &str)],
    kept: &[&str],
) -> Result<String, String> {
    substitute(text, bindings, kept)
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

/// The copies of the block at `span` of the part's source. `outer` names the
/// bindings of the blocks it is nested in, innermost last, which its copies
/// keep as written for those blocks to replace. `route` is false only where
/// the copies are read for their declarations, which routing does not change.
fn expand_block(
    span: (usize, usize),
    part: &Part,
    outer: &[&str],
    route: bool,
) -> Result<String, String> {
    let block = read_block(part.source, span, part.kinds)?;
    // The body is copied as written and no `;` is added, so its last
    // statement must end with its own `;`, whatever the number of members:
    // without one, each copy's last statement would run into what follows
    // the copy. A body with no statement, empty or of comments and empty
    // statements, needs none. It is refused here, before any copy, and not
    // in `read_block`, so a routed block elsewhere in the part still reads
    // the events this body declares, and the error is this one.
    if let Some(error) = block.unended {
        return Err(error);
    }
    // A name an enclosing block binds would be replaced here first, and the
    // enclosing block's member would never reach it.
    if outer.contains(&block.name) {
        return Err(format!(
            "`{}` binds {}, which an enclosing for block binds too; give the nested block another name",
            block.header.join(" "),
            block.binding
        ));
    }
    // Nested blocks expand first, with this block's names and the enclosing
    // blocks' kept as written. What is left of `$index` is this block's.
    let scope = [outer, &[block.name][..]].concat();
    let body = expand_nested(part, (block.body_start, block.body.len()), &scope, route)?;
    // Which rules are routed is read from the body, once.
    let routed = match block.route.filter(|_| route) {
        Some(parameter) => Routing {
            header: block.header.join(" "),
            kind: block.kind,
            binding: block.binding,
            name: block.name,
            outer,
            parameter,
        }
        .rules(&body, part)?,
        None => Vec::new(),
    };

    let mut out = String::new();
    for (position, member) in block.members.iter().enumerate() {
        // A copy nested in another block is read again when that block
        // expands, so a `$` in a member's name would be read as a binding.
        if !outer.is_empty() && member.contains('$') {
            return Err(format!(
                "`{}`: entity `{member}` has a `$` in its name, which the enclosing for block would read as a binding; rename it, or do not nest the block",
                block.header.join(" ")
            ));
        }
        let index = (position + 1).to_string();
        let bindings = [(block.name, member.as_str()), ("index", index.as_str())];
        match block.route.filter(|_| route) {
            None => out.push_str(&substitute(&body, &bindings, outer)?),
            Some(parameter) => out.push_str(&routed_copy(
                &body,
                &routed,
                &bindings,
                outer,
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
    /// Where the body begins in the part's source.
    body_start: usize,
    members: &'k [String],
    /// The refusal of the body's last statement when it has no `;`.
    unended: Option<String>,
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
    // A body that nothing closes is refused as such, not for a `{` that the
    // reader then finds in it below the `}` that was meant to close it.
    let close = braces
        .close
        .ok_or_else(|| format!("for block is not closed: {}", head(statement)))?;
    // A brace in unquoted text pairs only within its statement, so a `{` a
    // body statement leaves open is refused, not paired with a `}` in a later
    // statement, which would end the body there instead.
    if let Some(brace) = braces.unclosed {
        return Err(unclosed_brace(source, start, statement, open, brace));
    }
    // The body's copies replace the whole statement, so anything but
    // comments between the body and the block's end would be lost.
    let after = without_terminator(&statement[close + 1..]);
    if !blank_comments(after).trim().is_empty() {
        return Err(text_after_body(
            source,
            start,
            statement,
            (close, braces.ends_statement),
            after,
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
    let Some((_, members)) = kinds.iter().find(|(declared, _)| declared == kind) else {
        return Err(format!(
            "no entity is declared `kind {kind}`, so `for {kind}` has nothing to expand"
        ));
    };
    // Past `text_after_body`, the `}` that ends the body ends a statement
    // only when that statement has no `;`: the body's last one.
    let unended = braces
        .ends_statement
        .map(|from| unended_statement(source, start, statement, &header, (from, close)));
    Ok(Block {
        header,
        kind,
        binding,
        name,
        route,
        body,
        body_start: start + open + 1,
        members,
        unended,
    })
}

/// A routed block's header and what it names, for its checks and errors.
/// `header` is HEADER in spec/caveat-routed-repetition-0.1.md section 7.
struct Routing<'a> {
    header: String,
    kind: &'a str,
    binding: &'a str,
    /// The binding without its `$`.
    name: &'a str,
    /// The bindings of the blocks this one is nested in.
    outer: &'a [&'a str],
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
            name,
            outer,
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
        // Every statement in the body ends with its `;`, the last one too
        // (`expand_block`).
        for (start, end) in statements_of(body) {
            let code = blank_comments(without_terminator(&body[start..end]));
            let ["on", event, ..] = words(&code)[..] else {
                continue;
            };
            // An event written with `$NAME` or `$index` is the member's own,
            // so it already selects the member. One written with only an
            // enclosing block's name is not known until that block expands.
            if event.contains('$') {
                if !mentions(event, name, outer) {
                    return Err(format!(
                        "`on {event}` in `{header}`: the event is named with an enclosing for block's binding only, so this block cannot tell whether it names a {kind}; write the rule in the enclosing block"
                    ));
                }
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
            // Quoted text counts; a comment does not, as for C003. An
            // enclosing block's binding does not tell this block's copies
            // apart.
            if !mentions(&code, name, outer) {
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
    /// reports its own error when it is expanded. A body whose last statement
    /// has no `;` is read as its copies read, and its block is refused for
    /// that when it is expanded.
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
        // Routing changes no declaration, and reading the events of a block
        // must not route the blocks nested in it, which read the events.
        let body = expand_nested(
            self,
            (block.body_start, block.body.len()),
            &[block.name],
            false,
        )
        .ok()?;
        block
            .members
            .iter()
            .enumerate()
            .map(|(position, member)| {
                let index = (position + 1).to_string();
                substitute(
                    &body,
                    &[(block.name, member.as_str()), ("index", index.as_str())],
                    &[],
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
            entity_in_body(body_of(text)?, kind)
        })
    }
}

/// The first entity of `kind` declared in a block's body, or in a block
/// nested in it.
fn entity_in_body(body: &str, kind: &str) -> Option<String> {
    statements_of(body)
        .into_iter()
        .find_map(|(inner, inner_end)| {
            let text = &body[inner..inner_end];
            if statement_words(text).first() == Some(&"for") {
                return entity_in_body(body_of(text)?, kind);
            }
            entity_of(text)
                .filter(|(_, declared)| declared == kind)
                .map(|(name, _)| name)
        })
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

/// A block statement's body, between the braces that `body_braces` reads.
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
        "for block `{}`: `{written}` has a `{{` that its statement does not close; a brace in unquoted text pairs only within its statement, so quote the text, or take the brace out of a name",
        statement_words(&statement[..open]).join(" ")
    ))
}

/// The refusal of `after`, the text between the `}` at `close` that ends the
/// body of `statement`, a block statement at `start` of `source`, and the
/// block's end. `ended` is where the body statement that `}` ends begins,
/// if the `}` comes after its text, as in `from a}b;` or `from }{;`: that
/// `}` closes no `{` of its statement, so it ends the body.
fn text_after_body(
    source: &str,
    start: usize,
    statement: &str,
    (close, ended): (usize, Option<usize>),
    after: &str,
) -> String {
    let at = position_of(source, start + close);
    let Some(from) = ended else {
        return at.error(format!(
            "for block has text after its body: {}; end the block with `}};`",
            head(after)
        ));
    };
    // The statement as written, to the end of the word that holds the `}`.
    let word_end = statement[close..]
        .find(|ch: char| ch.is_whitespace() || ch == ';')
        .map_or(statement.len(), |length| close + length);
    let written = statement_words(&statement[from..word_end]).join(" ");
    at.error(format!(
        "for block has text after its body: {}; the `}}` in `{written}` closes no `{{` of its statement, so it ends the body: quote a brace that is text, take it out of a name, or end the block with `}};`",
        head(after)
    ))
}

/// The refusal of a body's last statement without its `;`: the statement at
/// `from` of `statement`, a block statement at `start` of `source` whose
/// body the `}` at `close` ends. The error gives the line and column where
/// that statement is written.
fn unended_statement(
    source: &str,
    start: usize,
    statement: &str,
    header: &[&str],
    (from, close): (usize, usize),
) -> String {
    position_of(source, start + from).error(format!(
        "the last statement in the body of `{}`, `{}`, has no `;` before the `}}` that ends the body; each member's copy of it would run into what follows the copy, so end it with `;`",
        header.join(" "),
        statement_words(&statement[from..close]).join(" ")
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
    outer: &[&str],
    route: &str,
) -> Result<String, String> {
    let mut out = String::with_capacity(body.len());
    let mut cursor = 0;
    for (start, end) in routed {
        out.push_str(&substitute(&body[cursor..*start], bindings, outer)?);
        out.push_str(&with_route(
            &substitute(&body[*start..*end], bindings, outer)?,
            route,
        ));
        cursor = *end;
    }
    out.push_str(&substitute(&body[cursor..], bindings, outer)?);
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

/// Whether `text` names `$NAME`, for the binding `name`, or `$index`, among
/// the `$` words it holds: each is read as `substitute` reads it, so one an
/// enclosing block's binding prefixes further belongs to that block.
fn mentions(text: &str, name: &str, outer: &[&str]) -> bool {
    dollar_words(text).any(|word| {
        let own = [name, "index"]
            .into_iter()
            .filter(|binding| word.starts_with(binding))
            .map(str::len)
            .max();
        let enclosing = outer
            .iter()
            .filter(|binding| word.starts_with(**binding))
            .map(|binding| binding.len())
            .max();
        own.is_some_and(|own| enclosing.is_none_or(|enclosing| own > enclosing))
    })
}

/// The run of identifier characters after each `$` in `text`.
fn dollar_words(text: &str) -> impl Iterator<Item = &str> {
    text.match_indices('$').map(move |(index, _)| {
        let rest = &text[index + 1..];
        let end = rest
            .char_indices()
            .find(|(at, next)| {
                if *at == 0 {
                    !is_identifier_start(*next)
                } else {
                    !is_identifier_char(*next)
                }
            })
            .map_or(rest.len(), |(at, _)| at);
        &rest[..end]
    })
}

/// Replace every `$binding` in the body. Substitution reaches inside quoted
/// text on purpose: the body is a template, and `evidence $r_reading from
/// "$r";` needs the member's name in both places. A `$` word that a binding
/// in `kept`, an enclosing block's, prefixes further than any in `bindings`
/// is left as written, for that block to replace.
fn substitute(body: &str, bindings: &[(&str, &str)], kept: &[&str]) -> Result<String, String> {
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
        let enclosing = kept
            .iter()
            .filter(|name| word.starts_with(**name))
            .map(|name| name.len())
            .max();
        if enclosing > matched.map(|(name, _)| name.len()) {
            out.push('$');
            out.push_str(word);
            while chars.peek().map(|(next, _)| *next < end) == Some(true) {
                chars.next();
            }
            continue;
        }
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
