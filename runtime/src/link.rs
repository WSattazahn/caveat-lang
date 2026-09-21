//! Draft 0.5 modules.
//!
//! Linking is a source-to-source pre-pass. It splits a bundle into modules,
//! checks the static rules in spec/caveat-0.5-draft.md, rewrites every name a
//! module declares or imports into a flat identifier, and concatenates the
//! result into one ordinary program. The evaluator, the transaction model and
//! the epistemic graph are untouched by this file: after linking there is one
//! program, one budget and one graph, exactly as before.

use crate::ast::Statement;
use crate::reactive::Directive;

/// First line of a multi-module source. A source without it is a single
/// program, so every existing `.cav` file and every existing save stays valid.
pub const BUNDLE_MARKER: &str = "#caveat-bundle 1";

/// Separator for flattened names. Source may not declare an identifier
/// containing it, so a rewritten name can never collide with a written one.
const FLAT: &str = "__";

/// One part of a bundle: a module, or the program that imports them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundlePart {
    pub name: String,
    pub source: String,
}

/// Split a bundle into its parts. Header byte lengths are exact, so this needs
/// no parsing and cannot disagree with what the author wrote.
pub fn split_bundle(bundle: &str) -> Result<Vec<BundlePart>, String> {
    let Some(rest) = bundle.strip_prefix(BUNDLE_MARKER) else {
        return Ok(vec![BundlePart {
            name: String::new(),
            source: bundle.to_string(),
        }]);
    };
    let mut rest = rest.strip_prefix('\n').unwrap_or(rest);
    let mut parts: Vec<BundlePart> = Vec::new();
    while !rest.is_empty() {
        let (header, after) = rest.split_once('\n').ok_or_else(|| {
            format!(
                "bundle header is not terminated: {header_hint}",
                header_hint = rest.trim()
            )
        })?;
        let fields: Vec<&str> = header.split_whitespace().collect();
        let ["#module", name, length] = fields.as_slice() else {
            return Err(format!(
                "expected a #module header, found: {}",
                header.trim()
            ));
        };
        let length: usize = length.parse().map_err(|_| {
            format!("bundle header for {name} has an invalid byte length: {length}")
        })?;
        if after.len() < length {
            return Err(format!(
                "bundle header for {name} declares {length} bytes but only {} remain",
                after.len()
            ));
        }
        if !after.is_char_boundary(length) {
            return Err(format!(
                "bundle header for {name} ends inside a multi-byte character"
            ));
        }
        if parts.iter().any(|part| part.name == *name) {
            return Err(format!("duplicate module in bundle: {name}"));
        }
        parts.push(BundlePart {
            name: (*name).to_string(),
            source: after[..length].to_string(),
        });
        rest = &after[length..];
    }
    if parts.is_empty() {
        return Err("bundle contains no parts".into());
    }
    Ok(parts)
}

/// Where each bundle part begins in the linked source: its name and the
/// one-based line the part's first line became.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMap {
    pub parts: Vec<(String, usize)>,
}

impl SourceMap {
    /// Translate a one-based line of linked source back to the part and line
    /// the author wrote. Linking concatenates parts, so a diagnostic from the
    /// evaluator is otherwise pointing at a line no author ever saw.
    pub fn locate(&self, line: usize) -> Option<(&str, usize)> {
        let (name, start) = self.parts.iter().rev().find(|(_, start)| *start <= line)?;
        Some((name.as_str(), line - start + 1))
    }
}

/// Link a bundle into one program source.
pub fn link(bundle: &str) -> Result<String, String> {
    link_with_map(bundle).map(|(source, _)| source)
}

/// Link a bundle, also returning where each part landed.
pub fn link_with_map(bundle: &str) -> Result<(String, SourceMap), String> {
    let parts = split_bundle(bundle)?;
    if parts.len() == 1 && parts[0].name.is_empty() {
        return Ok((
            parts[0].source.clone(),
            SourceMap {
                parts: vec![(String::new(), 1)],
            },
        ));
    }

    let (root, modules) = parts
        .split_last()
        .ok_or_else(|| "bundle contains no program".to_string())?;
    let mut declarations: Vec<(String, Vec<String>)> = Vec::new();
    for part in modules {
        let declared = module_declarations(part)?;
        declarations.push((part.name.clone(), declared));
    }
    if declared_module(&root.source).is_some() {
        return Err(format!(
            "the last bundle part ({}) is the program and must not declare a module",
            root.name
        ));
    }

    // Import edges, checked against the modules actually present.
    let mut imports: Vec<(String, Vec<String>)> = Vec::new();
    for part in modules.iter().chain(std::iter::once(root)) {
        let used = used_modules(&part.source)?;
        for name in &used {
            if !declarations.iter().any(|(module, _)| module == name) {
                return Err(format!(
                    "{} imports a module that is not in the bundle: {name}",
                    describe(part)
                ));
            }
            if *name == part.name {
                return Err(format!("{} imports itself", describe(part)));
            }
        }
        imports.push((part.name.clone(), used));
    }
    let order = topological_order(&imports, modules)?;

    let mut linked = String::new();
    let mut map = SourceMap { parts: Vec::new() };
    for name in &order {
        let part = modules
            .iter()
            .find(|part| part.name == *name)
            .expect("ordered module is present");
        let declared = declarations
            .iter()
            .find(|(module, _)| module == name)
            .map(|(_, declared)| declared.as_slice())
            .expect("ordered module was checked");
        map.parts.push((name.clone(), lines(&linked)));
        linked.push_str(&rewrite(part, Some(name), declared, &declarations)?);
        if !linked.ends_with('\n') {
            linked.push('\n');
        }
    }
    map.parts.push((root.name.clone(), lines(&linked)));
    linked.push_str(&rewrite(root, None, &[], &declarations)?);
    Ok((linked, map))
}

/// The one-based line that the next appended character starts on.
fn lines(text: &str) -> usize {
    text.chars().filter(|ch| *ch == '\n').count() + 1
}

fn describe(part: &BundlePart) -> String {
    if part.name.is_empty() {
        "the program".into()
    } else {
        format!("module {}", part.name)
    }
}

/// The names a module declares, after checking that it only declares.
fn module_declarations(part: &BundlePart) -> Result<Vec<String>, String> {
    let header = declared_module(&part.source).ok_or_else(|| {
        format!(
            "bundle part {} must begin with `module {};`",
            part.name, part.name
        )
    })?;
    if header != part.name {
        return Err(format!(
            "bundle part {} declares `module {header};`",
            part.name
        ));
    }
    let program = crate::parser::parse(&strip_headers(&part.source))
        .map_err(|message| format!("module {}: {message}", part.name))?;
    let mut declared = Vec::new();
    for statement in &program.statements {
        let name = match statement {
            Statement::Claim { name } => Some(name.clone()),
            Statement::Evidence { name, .. } => Some(name.clone()),
            Statement::Caveat { name, .. } => Some(name.clone()),
            Statement::Rule { name, .. } => Some(name.clone()),
            Statement::Reactive(Directive::Function(function)) => Some(function.name.clone()),
            Statement::Reactive(Directive::Procedure(procedure)) => Some(procedure.name.clone()),
            // Declare nothing, but are allowed to appear.
            Statement::Relate { .. } | Statement::Display { .. } | Statement::Presentation(_) => {
                None
            }
            Statement::Budget { .. } => {
                return Err(format!(
                    "module {} declares a budget; attention is spent by the program, not by a module",
                    part.name
                ));
            }
            other => {
                return Err(format!(
                    "module {} contains {}, which executes; a module may only declare",
                    part.name,
                    statement_kind(other)
                ));
            }
        };
        if let Some(name) = name {
            if name.contains(FLAT) {
                return Err(format!(
                    "module {} declares {name}; `{FLAT}` is reserved for linked names",
                    part.name
                ));
            }
            if declared.contains(&name) {
                return Err(format!("module {} declares {name} twice", part.name));
            }
            declared.push(name);
        }
    }
    Ok(declared)
}

/// A short name for a statement, used only in link diagnostics.
fn statement_kind(statement: &Statement) -> &'static str {
    match statement {
        Statement::Attention { .. } => "an attention statement",
        Statement::Examine { .. } => "an examination",
        Statement::Inspect { .. } => "an inspection",
        Statement::Infer { .. } => "an inference",
        Statement::Select { .. } => "a selection",
        Statement::Converge { .. } => "a convergence",
        Statement::Commit { .. } => "a commitment",
        Statement::Reopen { .. } => "a reopening",
        Statement::StartAt { .. } => "a starting place",
        Statement::Budget { .. } => "a budget",
        _ => "a statement that draft 0.5 does not allow in a module",
    }
}

/// Byte ranges of each `;`-terminated statement. Strings, comments and
/// procedure braces follow the same rules as the parser's own scanner, so a
/// semicolon inside quoted text or inside an effect body does not end a
/// statement here either.
fn statement_spans(source: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = None;
    let mut braces = 0usize;
    let mut chars = source.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if ch == '"' {
            if start.is_none() {
                start = Some(index);
            }
            let mut escaped = false;
            for (_, ch) in chars.by_ref() {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    break;
                }
            }
            continue;
        }
        if ch == '#' || (ch == '/' && chars.peek().map(|(_, next)| *next) == Some('/')) {
            for (_, ch) in chars.by_ref() {
                if ch == '\n' {
                    break;
                }
            }
            continue;
        }
        if ch == '{' {
            braces += 1;
        } else if ch == '}' {
            braces = braces.saturating_sub(1);
        }
        if ch == ';' && braces == 0 {
            if let Some(begin) = start.take() {
                spans.push((begin, index + 1));
            }
            continue;
        }
        if start.is_none() && !ch.is_whitespace() {
            start = Some(index);
        }
    }
    spans
}

/// The words of a statement, ignoring quoted text and comments.
fn statement_words(text: &str) -> Vec<&str> {
    text.trim_end_matches(';').split_whitespace().collect()
}

/// The name in a leading `module NAME;` statement, if there is one.
fn declared_module(source: &str) -> Option<String> {
    let (start, end) = *statement_spans(source).first()?;
    match statement_words(&source[start..end]).as_slice() {
        ["module", name] => Some((*name).to_string()),
        _ => None,
    }
}

/// Every module named by a `use NAME;` statement, in source order.
fn used_modules(source: &str) -> Result<Vec<String>, String> {
    let mut used = Vec::new();
    for (start, end) in statement_spans(source) {
        if let ["use", name] = statement_words(&source[start..end]).as_slice() {
            check_module_name(name)?;
            if used.iter().any(|existing| existing == name) {
                return Err(format!("duplicate import of {name}"));
            }
            used.push((*name).to_string());
        }
    }
    Ok(used)
}

/// A module name is a plain identifier. `load` turns one into a file path, so
/// anything that could name a directory, a parent, or a drive is rejected here
/// rather than at the filesystem.
fn check_module_name(name: &str) -> Result<(), String> {
    let mut chars = name.chars();
    let valid = chars.next().is_some_and(is_identifier_start) && chars.all(is_identifier_char);
    if !valid {
        return Err(format!("{name} is not a module name"));
    }
    if name.contains(FLAT) {
        return Err(format!(
            "module {name} contains `{FLAT}`, which is reserved for linked names"
        ));
    }
    Ok(())
}

/// Blank out `module NAME;` and `use NAME;` statements. They are link-time
/// declarations, not program statements, so the evaluator must never see them.
/// Their bytes become spaces rather than disappearing, which keeps every later
/// statement at the line and column the author wrote *within its own part*.
/// Concatenation then shifts whole parts, which `SourceMap` undoes.
fn strip_headers(source: &str) -> String {
    let mut out: Vec<char> = source.chars().collect();
    let offsets: Vec<usize> = source.char_indices().map(|(index, _)| index).collect();
    for (start, end) in statement_spans(source) {
        let words = statement_words(&source[start..end]);
        if !matches!(words.as_slice(), ["module", _] | ["use", _]) {
            continue;
        }
        for (position, offset) in offsets.iter().enumerate() {
            if *offset >= start && *offset < end && out[position] != '\n' {
                out[position] = ' ';
            }
        }
    }
    out.into_iter().collect()
}

/// Modules in dependency order. Rejects cycles by name.
fn topological_order(
    imports: &[(String, Vec<String>)],
    modules: &[BundlePart],
) -> Result<Vec<String>, String> {
    let mut order: Vec<String> = Vec::new();
    let mut visiting: Vec<String> = Vec::new();
    for part in modules {
        visit(&part.name, imports, &mut order, &mut visiting)?;
    }
    Ok(order)
}

fn visit(
    name: &str,
    imports: &[(String, Vec<String>)],
    order: &mut Vec<String>,
    visiting: &mut Vec<String>,
) -> Result<(), String> {
    if order.iter().any(|done| done == name) {
        return Ok(());
    }
    if visiting.iter().any(|open| open == name) {
        visiting.push(name.to_string());
        return Err(format!("import cycle: {}", visiting.join(" -> ")));
    }
    visiting.push(name.to_string());
    if let Some((_, used)) = imports.iter().find(|(module, _)| module == name) {
        for dependency in used {
            visit(dependency, imports, order, visiting)?;
        }
    }
    visiting.pop();
    order.push(name.to_string());
    Ok(())
}

/// Rewrite one part's identifiers. `module` is None for the program, whose own
/// names stay bare.
fn rewrite(
    part: &BundlePart,
    module: Option<&str>,
    declared: &[String],
    declarations: &[(String, Vec<String>)],
) -> Result<String, String> {
    let imported = used_modules(&part.source)?;
    let stripped = strip_headers(&part.source);
    let mut out = String::with_capacity(stripped.len());
    let mut chars = stripped.char_indices().peekable();
    let source = stripped.as_str();

    while let Some((index, ch)) = chars.next() {
        if ch == '"' {
            out.push(ch);
            let mut escaped = false;
            for (_, ch) in chars.by_ref() {
                out.push(ch);
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    break;
                }
            }
            continue;
        }
        if ch == '#' || (ch == '/' && chars.peek().map(|(_, next)| *next) == Some('/')) {
            out.push(ch);
            for (_, ch) in chars.by_ref() {
                out.push(ch);
                if ch == '\n' {
                    break;
                }
            }
            continue;
        }
        if !is_identifier_start(ch) {
            out.push(ch);
            continue;
        }

        let mut end = index + ch.len_utf8();
        while let Some((next, ch)) = chars.peek() {
            if is_identifier_char(*ch) {
                end = next + ch.len_utf8();
                chars.next();
            } else {
                break;
            }
        }
        let word = &source[index..end];

        // A qualified reference: WORD :: SYMBOL.
        if source[end..].starts_with("::") {
            let symbol_start = end + 2;
            let mut symbol_end = symbol_start;
            while let Some(ch) = source[symbol_end..].chars().next() {
                if (symbol_end == symbol_start && is_identifier_start(ch))
                    || (symbol_end > symbol_start && is_identifier_char(ch))
                {
                    symbol_end += ch.len_utf8();
                } else {
                    break;
                }
            }
            if symbol_end == symbol_start {
                return Err(format!("{}: `{word}::` names no symbol", describe(part)));
            }
            let symbol = &source[symbol_start..symbol_end];
            if !imported.iter().any(|used| used == word) {
                return Err(format!(
                    "{} names {word}::{symbol} without `use {word};`",
                    describe(part)
                ));
            }
            let Some((_, names)) = declarations.iter().find(|(module, _)| module == word) else {
                return Err(format!("{}: unknown module {word}", describe(part)));
            };
            if !names.iter().any(|declared| declared == symbol) {
                return Err(format!("module {word} does not declare {symbol}"));
            }
            out.push_str(&flat(word, symbol));
            // Skip the consumed `::SYMBOL`.
            while chars.peek().map(|(next, _)| *next < symbol_end) == Some(true) {
                chars.next();
            }
            continue;
        }

        match module {
            Some(module) if declared.iter().any(|name| name == word) => {
                out.push_str(&flat(module, word));
            }
            _ => out.push_str(word),
        }
    }
    Ok(out)
}

fn flat(module: &str, symbol: &str) -> String {
    format!("{module}{FLAT}{symbol}")
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

/// Build a bundle from ordered parts. The program is last.
pub fn bundle(parts: &[BundlePart]) -> String {
    let mut out = String::from(BUNDLE_MARKER);
    out.push('\n');
    for part in parts {
        out.push_str(&format!("#module {} {}\n", part.name, part.source.len()));
        out.push_str(&part.source);
    }
    out
}

/// Reading a program and its modules from disk.
///
/// `use weather;` resolves to `weather.cav` beside the entry file. Nothing
/// searches a wider path, and a module name is checked to be a plain
/// identifier before it becomes a filename, so source cannot reach outside the
/// directory it was loaded from.
#[cfg(not(target_arch = "wasm32"))]
pub mod disk {
    use super::{check_module_name, declared_module, used_modules, BundlePart};
    use std::path::{Path, PathBuf};

    /// Read an entry program and every module it reaches, returning a bundle.
    ///
    /// A program with no imports is returned byte-for-byte, with no bundle
    /// header, so single-file programs keep exactly the identity they have now.
    pub fn load(entry: &Path) -> Result<String, String> {
        let source = read(entry)?;
        let directory = entry.parent().unwrap_or_else(|| Path::new("."));
        let mut parts: Vec<BundlePart> = Vec::new();
        let mut pending = used_modules(&source)?;
        if pending.is_empty() {
            return Ok(source);
        }
        if let Some(name) = declared_module(&source) {
            return Err(format!(
                "{} is the program and must not declare `module {name};`",
                entry.display()
            ));
        }
        while let Some(name) = pending.pop() {
            if parts.iter().any(|part| part.name == name) {
                continue;
            }
            check_module_name(&name)?;
            let path = module_path(directory, &name);
            let module = read(&path)?;
            match declared_module(&module) {
                Some(declared) if declared == name => {}
                Some(declared) => {
                    return Err(format!(
                        "{} declares `module {declared};` but is imported as {name}",
                        path.display()
                    ))
                }
                None => {
                    return Err(format!(
                        "{} must begin with `module {name};`",
                        path.display()
                    ))
                }
            }
            pending.extend(used_modules(&module)?);
            parts.push(BundlePart {
                name,
                source: module,
            });
        }
        // Deterministic order regardless of filesystem or discovery order; the
        // linker sorts by dependency afterwards.
        parts.sort_by(|left, right| left.name.cmp(&right.name));
        parts.push(BundlePart {
            name: program_name(entry),
            source,
        });
        Ok(super::bundle(&parts))
    }

    fn module_path(directory: &Path, name: &str) -> PathBuf {
        directory.join(format!("{name}.cav"))
    }

    /// The program's part name is its file stem, which is what diagnostics and
    /// `SourceMap` report. It is a label, not an importable module name.
    fn program_name(entry: &Path) -> String {
        entry
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .filter(|stem| !stem.is_empty())
            .unwrap_or_else(|| "program".into())
    }

    fn read(path: &Path) -> Result<String, String> {
        std::fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))
    }
}
