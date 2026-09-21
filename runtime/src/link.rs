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

/// Link a bundle into one program source.
pub fn link(bundle: &str) -> Result<String, String> {
    let mut parts = split_bundle(bundle)?;
    // Repetition 0.1 expands first, so everything downstream — the declared
    // names, the single-writer check, the rewriter — sees the statements the
    // block generated rather than the block. A part with no `for` block is
    // returned unchanged, so this is a no-op for every existing program.
    for part in &mut parts {
        part.source = crate::repeat::expand(&part.source).map_err(|error| {
            if part.name.is_empty() {
                error.clone()
            } else {
                format!("{}: {error}", part.name)
            }
        })?;
    }
    if parts.len() == 1 && parts[0].name.is_empty() {
        return Ok(parts[0].source.clone());
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
    for (part, is_module) in modules
        .iter()
        .map(|part| (part, true))
        .chain(std::iter::once((root, false)))
    {
        let used = used_modules(&part.source)?;
        for name in &used {
            if !declarations.iter().any(|(module, _)| module == name) {
                return Err(format!(
                    "{} imports a module that is not in the bundle: {name}",
                    describe(part, is_module)
                ));
            }
            if *name == part.name {
                return Err(format!("{} imports itself", describe(part, is_module)));
            }
        }
        imports.push((part.name.clone(), used));
    }
    let order = topological_order(&imports, modules)?;

    let mut linked = String::new();
    let mut writes: Vec<(String, PartWrites)> = Vec::new();
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
        // The record of who declared what belongs in the artifact, not in a
        // side channel: a reader of the linked text sees it too.
        linked.push_str(&format!(
            "origin {name};
"
        ));
        let rewritten = rewrite(part, Some(name), declared, &declarations)?;
        let written = part_writes(&rewritten).map_err(|error| format!("module {name}: {error}"))?;
        writes.push((format!("module {name}"), written));
        linked.push_str(&rewritten);
        if !linked.ends_with('\n') {
            linked.push('\n');
        }
    }
    if !root.name.is_empty() {
        linked.push_str(&format!(
            "origin {};
",
            root.name
        ));
    }
    let rewritten = rewrite(root, None, &[], &declarations)?;
    let label = describe(root, false);
    let written = part_writes(&rewritten).map_err(|error| format!("{label}: {error}"))?;
    writes.push((label, written));
    linked.push_str(&rewritten);
    check_single_writer(&writes)?;
    Ok(linked)
}

/// How a part is named in diagnostics. The program carries a name too (its
/// file stem), so rootness has to be passed in rather than guessed from it.
fn describe(part: &BundlePart, module: bool) -> String {
    match (module, part.name.is_empty()) {
        (true, _) => format!("module {}", part.name),
        (false, true) => "the program".into(),
        (false, false) => format!("the program {}", part.name),
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
    let mut parameters: Vec<(String, String)> = Vec::new();
    for statement in &program.statements {
        let name = match statement {
            Statement::Claim { name } => Some(name.clone()),
            Statement::Evidence { name, .. } => Some(name.clone()),
            Statement::Caveat { name, .. } => Some(name.clone()),
            Statement::Rule { name, .. } => Some(name.clone()),
            Statement::Reactive(directive) => match directive {
                Directive::Function(function) => {
                    for parameter in &function.parameters {
                        parameters.push((parameter.clone(), function.name.clone()));
                    }
                    Some(function.name.clone())
                }
                Directive::Procedure(procedure) => {
                    for parameter in &procedure.parameters {
                        parameters.push((parameter.clone(), procedure.name.clone()));
                    }
                    Some(procedure.name.clone())
                }
                Directive::Event {
                    name,
                    parameters: declared_parameters,
                } => {
                    for parameter in declared_parameters {
                        parameters.push((parameter.name.clone(), name.clone()));
                    }
                    Some(name.clone())
                }
                Directive::State { name, .. }
                | Directive::Readings { name, .. }
                | Directive::Decisions { name, .. }
                | Directive::Control { name, .. } => Some(name.clone()),
                // Respond or project, but introduce no name of their own.
                Directive::Rule(_)
                | Directive::Binding(_)
                | Directive::Cue(_)
                | Directive::Clock(_) => None,
            },
            // The world's nouns. A module may declare them so that a
            // `for KIND` block has a kind to iterate in its own part; the
            // kind itself is a type tag like `consequence material`, not a
            // symbol, so it is not namespaced.
            Statement::Place { name, .. } => Some(name.clone()),
            Statement::Entity { name, .. } => Some(name.clone()),
            // Declare nothing, but are allowed to appear.
            Statement::Connect { .. }
            | Statement::Relate { .. }
            | Statement::Display { .. }
            | Statement::Presentation(_) => None,
            Statement::Budget { .. } => {
                return Err(format!(
                    "module {} declares a budget; attention is spent by the program, not by a module",
                    part.name
                ));
            }
            // Not an executing statement, so it needs its own reason: the
            // linker records where a part's declarations came from, and a
            // module that set its own could name a part that did not write it.
            Statement::Origin { part: claimed } => {
                return Err(format!(
                    "module {} declares `origin {claimed};`; the linker records a part's origin",
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
    // Rewriting is lexical, so a declared name that is also a parameter would
    // be rewritten inside the body and fuse the parameter with the
    // declaration. Refuse rather than silently rename one into the other.
    for (parameter, owner) in &parameters {
        if declared.contains(parameter) {
            return Err(format!(
                "module {} declares {parameter} and also takes it as a parameter of {owner}; rename one",
                part.name
            ));
        }
    }
    Ok(declared)
}

/// What a part writes: state cells it assigns, and host bindings it projects.
/// Collected from the rewritten text, so the names are already flat and can be
/// compared across parts.
#[derive(Default)]
struct PartWrites {
    assigned: Vec<String>,
    bound: Vec<(String, String)>,
}

fn part_writes(linked: &str) -> Result<PartWrites, String> {
    let program = crate::parser::parse(linked)?;
    let mut writes = PartWrites::default();
    for statement in &program.statements {
        let Statement::Reactive(directive) = statement else {
            continue;
        };
        match directive {
            Directive::Rule(rule) => collect_assignment(&rule.effect, &mut writes.assigned),
            Directive::Procedure(procedure) => {
                for step in &procedure.body {
                    collect_assignment(&step.effect, &mut writes.assigned);
                }
            }
            Directive::Binding(binding) => {
                // A part may cascade over one property as many times as it
                // likes; that order is the one the author wrote. Record it
                // once so only a second *part* counts as a conflict.
                let bound = (binding.target.clone(), binding.property.clone());
                if !writes.bound.contains(&bound) {
                    writes.bound.push(bound);
                }
            }
            _ => {}
        }
    }
    Ok(writes)
}

fn collect_assignment(effect: &crate::reactive::Effect, assigned: &mut Vec<String>) {
    if let crate::reactive::Effect::Set { name, .. } = effect {
        if !assigned.contains(name) {
            assigned.push(name.clone());
        }
    }
}

/// Refuse a state cell or a host binding written by more than one part.
///
/// Within a part, order is what the author wrote: `on` rules fire in source
/// order and the last matching `bind` wins. Across parts the order is the
/// linker's topological one, which nobody authored, so letting two parts write
/// the same name would hand the outcome to an ordering decision the author
/// never made. See spec/caveat-0.5-draft.md section 8.
fn check_single_writer(writes: &[(String, PartWrites)]) -> Result<(), String> {
    let mut cells: Vec<(&str, &str)> = Vec::new();
    let mut bindings: Vec<((&str, &str), &str)> = Vec::new();
    for (part, written) in writes {
        for cell in &written.assigned {
            if let Some((_, owner)) = cells.iter().find(|(name, _)| *name == cell) {
                return Err(format!(
                    "{cell} is assigned by both {owner} and {part}; a state cell is written by one part"
                ));
            }
            cells.push((cell, part));
        }
        for (target, property) in &written.bound {
            let key = (target.as_str(), property.as_str());
            if let Some((_, owner)) = bindings.iter().find(|(bound, _)| *bound == key) {
                return Err(format!(
                    "{target}.{property} is bound by both {owner} and {part}; a binding is written by one part"
                ));
            }
            bindings.push((key, part));
        }
    }
    Ok(())
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
pub(crate) fn statement_spans(source: &str) -> Vec<(usize, usize)> {
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
pub(crate) fn statement_words(text: &str) -> Vec<&str> {
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

        // An identifier directly after `.` is a member name, not a symbol: the
        // property in `bind ability.learned`, the axis in `ferry.x`. Both are
        // part of the host's contract and must survive linking unchanged.
        let member = out.ends_with('.');

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
                return Err(format!(
                    "{}: `{word}::` names no symbol",
                    describe(part, module.is_some())
                ));
            }
            let symbol = &source[symbol_start..symbol_end];
            if !imported.iter().any(|used| used == word) {
                return Err(format!(
                    "{} names {word}::{symbol} without `use {word};`",
                    describe(part, module.is_some())
                ));
            }
            let Some((_, names)) = declarations.iter().find(|(module, _)| module == word) else {
                return Err(format!(
                    "{}: unknown module {word}",
                    describe(part, module.is_some())
                ));
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
            Some(module) if !member && declared.iter().any(|name| name == word) => {
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

pub(crate) fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

pub(crate) fn is_identifier_char(ch: char) -> bool {
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
