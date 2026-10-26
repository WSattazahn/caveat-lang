//! Where a load error's unknown name is, and what it may have meant.
//!
//! Most load-time checks run on directives that no longer carry their source
//! positions, so their messages name the offending identifier but not where it
//! is. When a message names an unknown or undeclared identifier, this finds
//! its first use in the source and the closest name the source does use, and
//! adds both. It changes only the message text (docs/COMPATIBILITY.md lets
//! diagnostics change); whether the source loads is decided before it runs.

/// Phrases after which a message names the unknown identifier, and the one
/// before which it does (`X must name a declared ...`).
const AFTER: &[&str] = &[
    "unknown numeric identifier ",
    "unknown reactive commitment ",
    "unknown reactive state ",
    "unknown procedure ",
    "unknown history ",
    "unknown expression function ",
    "unknown source function ",
    "unknown citation identifier ",
    "unknown commit reason ",
    "unknown attention ",
    "unknown relation ",
    "unknown graph symbol ",
    "unknown caveat ",
    "unknown event ",
    "undeclared event ",
    "undeclared state ",
    "undeclared cue ",
    "names unknown evidence ",
    "names unknown commitment ",
    "names unknown event ",
    "names unknown reading ",
    "names unknown ",
];
const BEFORE: &str = " must name a declared ";

/// The message with ` (line L, column C)` and, when one is close, `; did you
/// mean NAME?` added. A message that already has a position, or names no
/// identifier the source uses, comes back unchanged.
pub(crate) fn locate(source: &str, message: String) -> String {
    if message.starts_with("line ") || message.contains(", line ") {
        return message;
    }
    let Some(name) = named(&message) else {
        return message;
    };
    let words = identifiers(source);
    let Some(&(_, line, column)) = words.iter().find(|(word, _, _)| *word == name) else {
        return message;
    };
    let mut located = format!("{message} (line {line}, column {column})");
    if let Some(close) = closest(name, &words) {
        located.push_str(&format!("; did you mean {close}?"));
    }
    located
}

/// Words that begin a statement or directive.
const KEYWORDS: &[&str] = &[
    "action",
    "bind",
    "budget",
    "caveat",
    "claim",
    "clock",
    "commit",
    "connect",
    "control",
    "cue",
    "decisions",
    "define",
    "display",
    "entity",
    "event",
    "evidence",
    "examine",
    "fn",
    "identifiers",
    "journal",
    "on",
    "origin",
    "place",
    "proc",
    "readings",
    "renewable",
    "reopen",
    "require",
    "resolve",
    "scene",
    "start_at",
    "state",
];

/// The message for a line no statement form matches: `--` is the most common
/// comment from other languages, and a first word one edit from a keyword is
/// most likely that keyword.
pub(crate) fn unparsed(line: &str) -> String {
    let first = line.split_whitespace().next().unwrap_or("");
    if first.starts_with("--") {
        return format!(
            "-- does not start a comment (comments start with # or //), so this statement runs on to the next semicolon: {line}"
        );
    }
    let mut message = format!("cannot parse statement: {line}");
    if !KEYWORDS.contains(&first) {
        let near = KEYWORDS
            .iter()
            .filter(|keyword| edits(first, keyword) <= if first.len() >= 6 { 2 } else { 1 })
            .min_by_key(|keyword| edits(first, keyword));
        if let Some(keyword) = near {
            message.push_str(&format!("; did you mean {keyword}?"));
        }
    }
    message
}

fn ident(text: &str) -> Option<&str> {
    let end = text
        .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .unwrap_or(text.len());
    (end > 0).then(|| &text[..end])
}

fn named(message: &str) -> Option<&str> {
    if let Some(at) = message.find(BEFORE) {
        let head = &message[..at];
        let start = head
            .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .map_or(0, |index| index + 1);
        let name = &head[start..];
        return (!name.is_empty()).then_some(name);
    }
    AFTER
        .iter()
        .find_map(|phrase| message.find(phrase).map(|at| &message[at + phrase.len()..]))
        .and_then(ident)
}

/// Every identifier outside comments and strings, with its 1-based line and
/// column (in characters), in source order.
fn identifiers(source: &str) -> Vec<(&str, usize, usize)> {
    let mut words = Vec::new();
    for (index, text) in source.lines().enumerate() {
        let mut chars = text.char_indices().peekable();
        let mut column = 0;
        let mut quoted = false;
        while let Some((at, ch)) = chars.next() {
            column += 1;
            if quoted {
                if ch == '\\' {
                    chars.next();
                    column += 1;
                } else if ch == '"' {
                    quoted = false;
                }
            } else if ch == '"' {
                quoted = true;
            } else if ch == '#' || (ch == '/' && text[at + 1..].starts_with('/')) {
                break;
            } else if ch.is_ascii_alphabetic() || ch == '_' {
                let start = column;
                let mut end = at + ch.len_utf8();
                while let Some(&(next, more)) = chars.peek() {
                    if !(more.is_ascii_alphanumeric() || more == '_') {
                        break;
                    }
                    end = next + more.len_utf8();
                    chars.next();
                    column += 1;
                }
                words.push((&text[at..end], index + 1, start));
            }
        }
    }
    words
}

/// "; did you mean NAME?" for the candidate nearest `name`, or nothing.
pub(crate) fn suggestion<'a>(name: &str, candidates: impl Iterator<Item = &'a str>) -> String {
    let words: Vec<(&str, usize, usize)> = candidates.map(|word| (word, 0, 0)).collect();
    closest(name, &words).map_or_else(String::new, |word| format!("; did you mean {word}?"))
}

/// The source identifier nearest `name` by edit distance, within 2 edits and
/// a third of its length; the first in source order wins a tie.
fn closest<'a>(name: &str, words: &[(&'a str, usize, usize)]) -> Option<&'a str> {
    let limit = (name.chars().count() / 3).clamp(1, 2);
    let mut best: Option<(usize, &str)> = None;
    for &(word, _, _) in words {
        if word == name || best.is_some_and(|(_, seen)| seen == word) {
            continue;
        }
        let distance = edits(name, word);
        if distance <= limit && best.is_none_or(|(least, _)| distance < least) {
            best = Some((distance, word));
        }
    }
    best.map(|(_, word)| word)
}

/// Optimal string alignment distance: insertions, deletions, substitutions
/// and adjacent transpositions.
fn edits(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > 2 {
        return usize::MAX;
    }
    let mut rows = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in rows.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in rows[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut value = (rows[i - 1][j] + 1)
                .min(rows[i][j - 1] + 1)
                .min(rows[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                value = value.min(rows[i - 2][j - 2] + 1);
            }
            rows[i][j] = value;
        }
    }
    rows[a.len()][b.len()]
}

#[cfg(test)]
mod tests {
    use super::locate;

    const SOURCE: &str = "readings rain_chance from forecast limit 4;\n\
        event read_forecast chance min 0 max 100;\n\
        # rain_chanse in a comment is skipped\n\
        on read_forecast sample rain_chanse = chanse supports rain_likely;\n";

    #[test]
    fn a_misspelled_name_gets_its_position_and_the_name_it_was_near() {
        assert_eq!(
            locate(SOURCE, "rain_chanse must name a declared reading stream".into()),
            "rain_chanse must name a declared reading stream (line 4, column 25); did you mean rain_chance?"
        );
        assert_eq!(
            locate(SOURCE, "unknown numeric identifier chanse".into()),
            "unknown numeric identifier chanse (line 4, column 39); did you mean chance?"
        );
    }

    #[test]
    fn a_message_with_a_position_or_no_name_from_the_source_is_unchanged() {
        for message in [
            "line 3, column 1: cannot parse statement: clam x",
            "unknown numeric identifier nowhere",
            "reactive program must declare an event",
        ] {
            assert_eq!(locate(SOURCE, message.into()), message);
        }
    }

    #[test]
    fn an_unparsed_line_names_the_comment_syntax_or_the_keyword_it_was_near() {
        assert_eq!(
            super::unparsed("-- hello"),
            "-- does not start a comment (comments start with # or //), so this statement runs on to the next semicolon: -- hello"
        );
        assert_eq!(
            super::unparsed("clam rain_likely"),
            "cannot parse statement: clam rain_likely; did you mean claim?"
        );
        assert_eq!(
            super::unparsed("evidense sky from \"x\""),
            "cannot parse statement: evidense sky from \"x\"; did you mean evidence?"
        );
        assert_eq!(super::unparsed("x y z"), "cannot parse statement: x y z");
    }

    #[test]
    fn no_suggestion_when_nothing_is_close() {
        let source = "event go;\non go set zzz = 1;\n";
        assert_eq!(
            locate(source, "set references undeclared state zzz".into()),
            "set references undeclared state zzz (line 2, column 11)"
        );
    }
}
