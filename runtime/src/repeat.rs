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
//! Section 10 of Routed Repetition 0.1 lets a routed rule name a second
//! member, the one its event names by another `kind` parameter Q, as `$Q`.
//! Each member's copy of such a rule is written once for each member of Q's
//! kind, with `Q == N` after the route. It is a text expansion too: each copy
//! is the rule an author would write by hand for that pair, and at most one
//! of them runs on an event.

use crate::link::{
    blank_comments, block_braces, is_identifier_char, is_identifier_start, statement_words,
    statements_of,
};
use crate::parser::position_of;
use crate::reactive::MAX_EVENT_STEPS;

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

/// `instantiate`, with each `$Q` also replaced by the text `markers` gives
/// for Q: how check reads a rule that names a member by another `kind`
/// parameter of its event (spec/caveat-routed-repetition-0.1.md section 10).
/// A marker named as the binding or `index` binds nothing, so the block's own
/// bindings keep their words.
pub(crate) fn instantiate_with(
    template: &str,
    (binding, member, index): (&str, &str, &str),
    markers: &[(&str, &str)],
) -> Result<String, String> {
    let mut bindings = vec![(binding, member), ("index", index)];
    bindings.extend(
        markers
            .iter()
            .filter(|(name, _)| *name != binding && *name != "index"),
    );
    substitute(template, &bindings)
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
    let header = block.header.join(" ");
    // The part's events, read for a routed block, and for a plain block only
    // when one of its rules has a `$` word that no binding of its own begins,
    // which does not load. A plain block that loads expands as it always did.
    let mut events = None;
    // Which rules are routed is read from the body as written, once.
    let routed = match block.route {
        Some(parameter) => Routing {
            header: header.clone(),
            kind: block.kind,
            binding: block.binding,
            parameter,
        }
        .rules(
            block.body,
            part,
            events.get_or_insert_with(|| part.events()),
        )?,
        None => Vec::new(),
    };
    // Section 10: the routed rules that name a member by another `kind`
    // parameter of their event, and the checks of every `$` word in a rule.
    let named = parameter_rules(&block, &header, &routed, part, &mut events)?;
    event_work(&block, &header, &named)?;

    let mut out = String::new();
    for (position, member) in block.members.iter().enumerate() {
        let index = (position + 1).to_string();
        let bindings = [(block.name, member.as_str()), ("index", index.as_str())];
        match block.route {
            None => out.push_str(&substitute(block.body, &bindings)?),
            Some(parameter) => out.push_str(&routed_copy(
                block.body,
                (&routed, &named),
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

/// An event a part declares: its name, and each parameter's name with the
/// words declared after it, such as `kind pr` or `in failed passed`.
type Event = (String, Vec<(String, String)>);

/// A routed rule that names a member by `$Q`, where Q is another `kind`
/// parameter of its event (spec/caveat-routed-repetition-0.1.md section 10).
struct Named<'k> {
    /// The rule's span in the body.
    span: (usize, usize),
    event: String,
    /// Q.
    parameter: String,
    /// The part's top-level members of Q's kind, which Q numbers from 1.
    members: &'k [String],
}

/// Section 10's checks, rule by rule in the order written, and the routed
/// rules that name a member by `$Q`.
///
/// In a routed rule, each `$` word, quoted text included and comments not,
/// is read by section 1's rule. `$NAME` and `$index` keep a word that one of
/// them begins, unless a `kind` parameter of the rule's event begins it too
/// and is longer, or as long and not P: then either could be meant, and the
/// rule is refused. A word neither begins is bound by the longest `kind`
/// parameter that begins it, which may not be P. A word no parameter begins
/// is read as before. Any other `on` rule, in a plain block too, is refused
/// only for a word that no binding of the block's own begins and a `kind`
/// parameter of its event does, which is not bound there. Its other words
/// are read as before.
fn parameter_rules<'p>(
    block: &Block,
    header: &str,
    routed: &[(usize, usize)],
    part: &Part<'p>,
    events: &mut Option<Vec<Event>>,
) -> Result<Vec<Named<'p>>, String> {
    // The longest binding of the block's own that begins a word, as
    // `substitute` binds it.
    let own = |word: &str| {
        [block.name, "index"]
            .into_iter()
            .filter(|name| word.starts_with(name))
            .max_by_key(|name| name.len())
    };
    let mut out = Vec::new();
    for (start, end) in statements_of(block.body) {
        let is_routed = routed.contains(&(start, end));
        let code = blank_comments(without_terminator(&block.body[start..end]));
        let dollars = dollar_words(&code);
        // Any other statement with only the block's own words reads as
        // before, without its event read.
        if !is_routed && dollars.iter().all(|word| own(word).is_some()) {
            continue;
        }
        let ["on", event, ..] = words(&code)[..] else {
            continue;
        };
        // An event written with `$NAME` or `$index` is read as before.
        if event.contains('$') {
            continue;
        }
        let events = events.get_or_insert_with(|| part.events());
        let Some((_, parameters)) = events.iter().find(|(declared, _)| declared == event) else {
            continue;
        };
        let kinds = parameters.iter().filter_map(|(name, form)| {
            match form.split_whitespace().collect::<Vec<_>>()[..] {
                ["kind", kind] => Some((name.as_str(), kind)),
                _ => None,
            }
        });
        if !is_routed {
            let unbound = dollars.iter().find(|word| {
                own(word).is_none() && kinds.clone().any(|(name, _)| word.starts_with(name))
            });
            if let Some(word) = unbound {
                return Err(format!(
                    "`${word}` in `on {event}` in `{header}`: a `$` names the member a parameter names only in a routed rule (spec/caveat-routed-repetition-0.1.md section 10), and this rule is not routed"
                ));
            }
            continue;
        }
        // Only a routed block has routed rules.
        let route = block.route.unwrap_or_default();
        let mut named: Vec<(&str, &str)> = Vec::new();
        let mut mentions_own = false;
        for word in &dollars {
            let longest = kinds
                .clone()
                .filter(|(name, _)| word.starts_with(name))
                .max_by_key(|(name, _)| name.len());
            match (own(word), longest) {
                (Some(mine), Some((name, kind)))
                    if name.len() > mine.len() || (name.len() == mine.len() && name != route) =>
                {
                    return Err(either_binding(
                        block,
                        header,
                        (word, mine),
                        (event, name, kind),
                    ));
                }
                (Some(_), _) => mentions_own = true,
                (None, Some((name, kind))) if name == route => {
                    return Err(format!(
                        "`${word}` in `on {event}` in `{header}`: the block is routed by `{name}`, so the {kind} it names is `{}`; write `{}` for `${name}`",
                        block.binding, block.binding
                    ));
                }
                (None, Some(parameter)) => {
                    if !named.contains(&parameter) {
                        named.push(parameter);
                    }
                }
                (None, None) => {
                    let other = parameters
                        .iter()
                        .filter(|(name, _)| word.starts_with(name.as_str()))
                        .max_by_key(|(name, _)| name.len());
                    if let Some((name, form)) = other {
                        let declared = format!("{name} {form}");
                        return Err(format!(
                            "`${word}` in `on {event}` in `{header}`: `{event}` declares `{}`, and a `$` names a member only by a `kind` parameter",
                            declared.trim_end()
                        ));
                    }
                    // No binding begins it: refused as not bound when the
                    // copies are written, as before.
                }
            }
        }
        let (parameter, kind) = match named[..] {
            [] => continue,
            [one] => one,
            [.., last] => {
                let names = named[..named.len() - 1]
                    .iter()
                    .map(|(name, _)| format!("`${name}`"))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(format!(
                    "`on {event}` in `{header}` names members by {names} and `${}`; a rule names a member by one parameter at most, so write it in a block routed by one of them",
                    last.0
                ));
            }
        };
        if !mentions_own {
            return Err(format!(
                "`on {event}` in `{header}` names members only by `${parameter}`, and neither `{}` nor `$index`; each {}'s copy of it would be the same rule, so write it in a block routed by `{parameter}`",
                block.binding, block.kind
            ));
        }
        let Some((_, members)) = part.kinds.iter().find(|(declared, _)| declared == kind) else {
            return Err(format!(
                "`${parameter}` in `on {event}` in `{header}`: no entity is declared `kind {kind}` at the top level of this part, so `${parameter}` names no member"
            ));
        };
        if let Some(entity) = part.entity_in_block(kind) {
            return Err(format!(
                "`${parameter}` in `on {event}` in `{header}`: `{parameter}` counts entity `{entity}` of kind {kind}, declared in a for block, and `${parameter}` does not; declare it at the top level of this part"
            ));
        }
        out.push(Named {
            span: (start, end),
            event: event.to_string(),
            parameter: parameter.to_string(),
            members,
        });
    }
    Ok(out)
}

/// The refusal of `word`, which `own`, `$NAME` or `$index`, begins, and the
/// `kind` parameter `name` of `event` begins too, longer, or as long and not
/// P. The remedies are the ones that apply: renaming the binding, which
/// `$index` cannot be, and routing by the parameter, which P already is.
fn either_binding(
    block: &Block,
    header: &str,
    (word, own): (&str, &str),
    (event, name, kind): (&str, &str, &str),
) -> String {
    let written = if own == "index" {
        "$index"
    } else {
        block.binding
    };
    let mut remedies = Vec::new();
    if own != "index" {
        remedies.push(format!(
            "rename `{}` so that only one of them begins it",
            block.binding
        ));
    }
    if block.route != Some(name) {
        remedies.push(format!("write the rule in a block routed by `{name}`"));
    }
    if remedies.is_empty() {
        remedies.push(format!(
            "write `{}` for the {kind} that `{name}` names",
            block.binding
        ));
    }
    format!(
        "`${word}` in `on {event}` in `{header}` could be `{written}` or the {kind} `{event}` names by `{name}`; {}",
        remedies.join(", or ")
    )
}

/// Section 10's count of the work an event does, for each event that a rule
/// naming a member by `$Q` is on: W×|K| copies of each such rule, and W of
/// each other rule of the block on the event, W the block's members and K
/// Q's kind. Every copy spends a step of the event's work (reactive.rs
/// `MAX_EVENT_STEPS`), whether its route holds or not. The count is a floor:
/// rules outside the block, and the steps of a procedure a copy calls, are
/// counted only when the event runs.
fn event_work(block: &Block, header: &str, named: &[Named]) -> Result<(), String> {
    if named.is_empty() {
        return Ok(());
    }
    let rules = statements_of(block.body)
        .into_iter()
        .filter_map(|(start, end)| {
            let code = blank_comments(without_terminator(&block.body[start..end]));
            match words(&code)[..] {
                ["on", event, ..] => Some(((start, end), event.to_string())),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    let mut counted: Vec<&str> = Vec::new();
    for rule in named {
        if counted.contains(&rule.event.as_str()) {
            continue;
        }
        counted.push(&rule.event);
        let steps = rules
            .iter()
            .filter(|(_, event)| *event == rule.event)
            .map(|(span, _)| {
                let copies = named
                    .iter()
                    .find(|named| named.span == *span)
                    .map_or(1, |named| named.members.len());
                block.members.len().saturating_mul(copies)
            })
            .fold(0, usize::saturating_add);
        if steps > MAX_EVENT_STEPS {
            return Err(format!(
                "`{header}` writes {steps} rules on `{}`, and one event does at most {MAX_EVENT_STEPS} steps of work; each copy spends a step whether its route holds or not",
                rule.event
            ));
        }
    }
    Ok(())
}

/// Each `$` word in a statement: the name after the `$`, as `substitute`
/// reads it. A `$` that no name follows has none.
fn dollar_words(code: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = code;
    while let Some(at) = rest.find('$') {
        let after = &rest[at + 1..];
        let mut end = 0;
        for (offset, ch) in after.char_indices() {
            let acceptable = if offset == 0 {
                is_identifier_start(ch)
            } else {
                is_identifier_char(ch)
            };
            if !acceptable {
                break;
            }
            end = offset + ch.len_utf8();
        }
        if end > 0 {
            out.push(&after[..end]);
        }
        rest = &after[end..];
    }
    out
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
    parameter: &'a str,
}

impl Routing<'_> {
    /// The routed rules of the body, by span, once every check of section 7
    /// has passed: an entity of KIND in a `for` block, which `$index` does
    /// not count, then each rule in the order written, then whether any rule
    /// is routed. A rule is routed from its event alone; what the rule does
    /// is not read. `events` are the part's.
    fn rules(
        &self,
        body: &str,
        part: &Part,
        events: &[Event],
    ) -> Result<Vec<(usize, usize)>, String> {
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
    /// expanded: only a block whose statements other than its `on` rules
    /// Repetition 0.1 expands for every member counts. A block that does not
    /// expand is left out here and reports its own error when it is
    /// expanded. A body whose last statement has no `;` is read as its copies
    /// read, and its block is refused for that when it is expanded.
    fn events(&self) -> Vec<Event> {
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

    /// Each member's copy of a `for` block's body without its `on` rules, as
    /// Repetition 0.1 expands it. None when it does not expand.
    fn copies(&self, span: (usize, usize)) -> Option<Vec<String>> {
        let block = read_block(self.source, span, self.kinds).ok()?;
        // An `on` rule declares no event, and one that names a member by `$Q`
        // is bound only once its event is read (section 10), so the rules are
        // left out: such a rule does not hide the block's events from a
        // routed block that reads them.
        let declarations = statements_of(block.body)
            .into_iter()
            .map(|(start, end)| &block.body[start..end])
            .filter(|statement| {
                words(&blank_comments(without_terminator(statement))).first() != Some(&"on")
            })
            .collect::<Vec<_>>()
            .join("\n");
        block
            .members
            .iter()
            .enumerate()
            .map(|(position, member)| {
                let index = (position + 1).to_string();
                substitute(
                    &declarations,
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
/// with the route written into each copy of a routed rule. A rule that names
/// a member by `$Q` is written once for each member of Q's kind, in
/// declaration order, the copies one after another where the rule is, with
/// one space between them, and each copy's route `P == I and Q == N`
/// (section 10).
fn routed_copy(
    body: &str,
    (routed, named): (&[(usize, usize)], &[Named]),
    bindings: &[(&str, &str); 2],
    route: &str,
) -> Result<String, String> {
    let mut out = String::with_capacity(body.len());
    let mut cursor = 0;
    for &(start, end) in routed {
        out.push_str(&substitute(&body[cursor..start], bindings)?);
        let rule = &body[start..end];
        match named.iter().find(|named| named.span == (start, end)) {
            None => out.push_str(&with_route(&substitute(rule, bindings)?, route)),
            Some(named) => {
                for (position, member) in named.members.iter().enumerate() {
                    if position > 0 {
                        out.push(' ');
                    }
                    let [own, index] = *bindings;
                    let copy = substitute(
                        rule,
                        &[own, index, (named.parameter.as_str(), member.as_str())],
                    )?;
                    out.push_str(&with_route(
                        &copy,
                        &format!("{route} and {} == {}", named.parameter, position + 1),
                    ));
                }
            }
        }
        cursor = end;
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
fn substitute(body: &str, bindings: &[(&str, &str)]) -> Result<String, String> {
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
