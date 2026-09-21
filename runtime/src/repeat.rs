//! Repetition 0.1: `for KIND as $NAME { ... };`
//!
//! A source-to-source pre-pass that expands a block once per member of a
//! declared entity kind. See spec/caveat-repetition-0.1.md. Nothing here adds
//! runtime semantics: the output is ordinary statements the author could have
//! written by hand, which is what keeps order — the thing that decides which
//! `on` rule fires first and which `bind` wins — the author's.

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
    if statement_words(body).contains(&"for") {
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
