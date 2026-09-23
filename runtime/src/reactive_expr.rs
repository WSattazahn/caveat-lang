//! Bounded, typed expressions for source-authored reactive game rules.
//!
//! Expressions can read numeric state and coordinates, and query the host's
//! epistemic graph. They cannot mutate state or invoke arbitrary host code.

use crate::presentation::Number;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

const MAX_TOKENS: usize = 1024;
const MAX_NESTING: usize = 64;
const MAX_SOURCE_BYTES: usize = 65_536;
const MAX_FUNCTIONS: usize = 128;
const MAX_EXPANDED_NODES: usize = 4096;
const MAX_EVALUATED_NODES: usize = 65_536;
const MAX_FOLD_RECORDS: usize = 256;
const FOLD_ACC: &str = "$fold_acc";
const FOLD_VALUE: &str = "$fold_value";
type QualificationSink<'a> = dyn Fn(&str, &[String]) -> Result<(), String> + 'a;

/// Read-only requests against one immutable event snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryRead {
    Latest,
    Count,
    At(usize),
}
pub const MAX_PROVENANCE_IDENTIFIERS: usize = 1024;
pub const MAX_PROVENANCE_BYTES: usize = 65_536;
pub const MAX_TEXT_BYTES: usize = 65_536;

/// Dependencies of an evaluated value, not an assertion that evidence is true
/// or that a caveat is discharged. The host resolves and types graph names.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Provenance {
    pub evidence: BTreeSet<String>,
    pub caveats: BTreeSet<String>,
}

impl Provenance {
    /// Construct a sorted, deduplicated trace without truncating dependencies.
    pub fn from_names(
        evidence: impl IntoIterator<Item = String>,
        caveats: impl IntoIterator<Item = String>,
    ) -> Result<Self, String> {
        let mut result = Self::default();
        let mut count = 0;
        let mut bytes = 0;
        for (is_evidence, name) in evidence
            .into_iter()
            .map(|name| (true, name))
            .chain(caveats.into_iter().map(|name| (false, name)))
        {
            let names = if is_evidence {
                &mut result.evidence
            } else {
                &mut result.caveats
            };
            if names.contains(&name) {
                continue;
            }
            check_provenance_size(count + 1, bytes, name.len())?;
            count += 1;
            bytes += name.len();
            names.insert(name);
        }
        Ok(result)
    }

    pub fn is_empty(&self) -> bool {
        self.evidence.is_empty() && self.caveats.is_empty()
    }

    fn size(&self) -> Result<(usize, usize), String> {
        let count = self
            .evidence
            .len()
            .checked_add(self.caveats.len())
            .ok_or_else(provenance_identifier_error)?;
        check_provenance_size(count, 0, 0)?;
        let mut bytes = 0;
        for name in self.evidence.iter().chain(&self.caveats) {
            check_provenance_size(count, bytes, name.len())?;
            bytes += name.len();
        }
        Ok((count, bytes))
    }

    /// Recheck public fields at a trust boundary before accepting host metadata.
    pub fn validate(&self) -> Result<(), String> {
        self.size().map(|_| ())
    }

    /// Union is deterministic and preserves both dependency categories.
    pub fn union(&self, other: &Self) -> Result<Self, String> {
        self.validate()?;
        other.validate()?;
        let mut result = self.clone();
        result.merge(other)?;
        Ok(result)
    }

    /// An overflow leaves `self` unchanged. No identifier is silently dropped.
    pub fn merge(&mut self, other: &Self) -> Result<(), String> {
        let (mut count, mut bytes) = self.size()?;
        other.validate()?;
        for name in other
            .evidence
            .difference(&self.evidence)
            .chain(other.caveats.difference(&self.caveats))
        {
            check_provenance_size(count + 1, bytes, name.len())?;
            count += 1;
            bytes += name.len();
        }
        self.evidence.extend(other.evidence.iter().cloned());
        self.caveats.extend(other.caveats.iter().cloned());
        Ok(())
    }
}

fn provenance_identifier_error() -> String {
    format!("value provenance exceeds limit {MAX_PROVENANCE_IDENTIFIERS} identifiers")
}

fn check_provenance_size(count: usize, bytes: usize, additional: usize) -> Result<(), String> {
    if count > MAX_PROVENANCE_IDENTIFIERS {
        return Err(provenance_identifier_error());
    }
    if bytes
        .checked_add(additional)
        .is_none_or(|total| total > MAX_PROVENANCE_BYTES)
    {
        return Err(format!(
            "value provenance exceeds limit {MAX_PROVENANCE_BYTES} name bytes"
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Tracked<T> {
    pub value: T,
    pub provenance: Provenance,
}

impl<T> Tracked<T> {
    pub fn plain(value: T) -> Self {
        Self {
            value,
            provenance: Provenance::default(),
        }
    }

    /// Numeric finiteness is checked by expression evaluation; this generic
    /// constructor checks only the dependency metadata.
    pub fn new(value: T, provenance: Provenance) -> Result<Self, String> {
        provenance.validate()?;
        Ok(Self { value, provenance })
    }
}

/// What an expression may read. See `Expr::collect_reads`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reads {
    /// Numeric names looked up: states, constants, parameters or locals.
    pub names: BTreeSet<String>,
    /// Queries the epistemic graph, a reading stream or a decision series.
    pub graph: bool,
    /// Could read anything; never skip it.
    pub anything: bool,
    /// Evidence named by `observed(...)`.
    pub observed: BTreeSet<String>,
}

/// A parsed expression with finite, canonical number literals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    node: Node,
    depth: usize,
}

/// A pure source function with numeric parameters. Definitions may refer to other functions
/// in any declaration order, but cannot capture state or graph predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Number(Number),
    Bool(bool),
    Text(String),
    Variable(String),
    Unary(Unary, Box<Expr>),
    Binary(Binary, Box<Expr>, Box<Expr>),
    Function(Function, Vec<Expr>),
    UserCall(String, Vec<Expr>),
    // Keep arguments even when the body ignores them: function arguments are
    // numeric and evaluated eagerly, so `constant(1 / 0)` must still fail.
    ExpandedCall(Vec<Expr>, Box<Expr>),
    Predicate(String, String),
    Latest(String),
    HistoryCount(String),
    HistoryAt(String, Box<Expr>),
    Fold(String, Box<Expr>, String),
    ExpandedFold(String, Box<Expr>, Box<Expr>),
    Qualified(Box<Expr>, String, Vec<String>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Require(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unary {
    Positive,
    Negative,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Binary {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Function {
    Sin,
    Cos,
    Sqrt,
    Atan2,
    Floor,
    Ceil,
    Round,
    Text,
}

impl Function {
    fn named(name: &str) -> Option<Self> {
        match name {
            "sin" => Some(Self::Sin),
            "cos" => Some(Self::Cos),
            "sqrt" => Some(Self::Sqrt),
            "atan2" => Some(Self::Atan2),
            "floor" => Some(Self::Floor),
            "ceil" => Some(Self::Ceil),
            "round" => Some(Self::Round),
            "text" => Some(Self::Text),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Sqrt => "sqrt",
            Self::Atan2 => "atan2",
            Self::Floor => "floor",
            Self::Ceil => "ceil",
            Self::Round => "round",
            Self::Text => "text",
        }
    }

    fn arity(self) -> usize {
        match self {
            Self::Atan2 => 2,
            _ => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Number,
    Bool,
    Text,
}

impl ValueType {
    fn name(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::Bool => "boolean",
            Self::Text => "text",
        }
    }

    fn require(self, expected: Self) -> Result<(), String> {
        if self == expected {
            Ok(())
        } else {
            Err(format!(
                "expression requires {}, found {}",
                expected.name(),
                self.name()
            ))
        }
    }
}

impl Value {
    fn number(&self) -> Result<f64, String> {
        match self {
            Self::Number(value) => finite(*value),
            Self::Bool(_) => Err("expression requires number, found boolean".into()),
            Self::Text(_) => Err("expression requires number, found text".into()),
        }
    }

    fn boolean(&self) -> Result<bool, String> {
        match self {
            Self::Bool(value) => Ok(*value),
            Self::Number(_) => Err("expression requires boolean, found number".into()),
            Self::Text(_) => Err("expression requires boolean, found text".into()),
        }
    }
}

fn finite(value: f64) -> Result<f64, String> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err("expression produced a non-finite number".into())
    }
}

fn check_text_size(bytes: usize) -> Result<(), String> {
    if bytes > MAX_TEXT_BYTES {
        Err(format!(
            "expression text exceeds limit {MAX_TEXT_BYTES} bytes"
        ))
    } else {
        Ok(())
    }
}

impl Expr {
    /// Count retained AST nodes and owned string bytes after expansion. The
    /// shared source library uses this in addition to per-expression bounds.
    pub(crate) fn storage_usage(&self) -> (usize, usize) {
        let mut nodes = 1;
        let mut bytes = 0;
        let mut visit = |child: &Expr| {
            let (child_nodes, child_bytes) = child.storage_usage();
            nodes += child_nodes;
            bytes += child_bytes;
        };
        match &self.node {
            Node::Unary(_, child) | Node::HistoryAt(_, child) => visit(child),
            Node::Binary(_, left, right) | Node::Require(left, right) => {
                visit(left);
                visit(right);
            }
            Node::If(condition, yes, no) => {
                visit(condition);
                visit(yes);
                visit(no);
            }
            Node::Function(_, arguments) | Node::UserCall(_, arguments) => {
                for argument in arguments {
                    visit(argument);
                }
            }
            Node::ExpandedCall(arguments, body) => {
                for argument in arguments {
                    visit(argument);
                }
                visit(body);
            }
            Node::Fold(_, initial, _) => visit(initial),
            Node::ExpandedFold(_, initial, body) => {
                visit(initial);
                visit(body);
            }
            Node::Qualified(value, _, _) => visit(value),
            _ => {}
        }
        match &self.node {
            Node::Number(number) => bytes += number.value().to_string().len(),
            Node::Text(text)
            | Node::Variable(text)
            | Node::Latest(text)
            | Node::HistoryCount(text)
            | Node::UserCall(text, _)
            | Node::HistoryAt(text, _)
            | Node::ExpandedFold(text, _, _) => bytes += text.len(),
            Node::Predicate(kind, name) | Node::Fold(kind, _, name) => {
                bytes += kind.len() + name.len()
            }
            Node::Qualified(_, evidence, caveats) => {
                bytes += evidence.len() + caveats.iter().map(String::len).sum::<usize>();
            }
            _ => {}
        }
        (nodes, bytes)
    }

    /// Everything evaluating this expression can read: the numeric names it
    /// looks up, and whether it queries the graph or a history. Every branch
    /// is included, taken or not, so this over-approximates and never misses
    /// a read. The match is exhaustive on purpose: a new kind of node must say
    /// what it reads before it compiles.
    pub fn collect_reads(&self, reads: &mut Reads) {
        match &self.node {
            Node::Number(_) | Node::Bool(_) | Node::Text(_) => {}
            Node::Variable(name) => {
                reads.names.insert(name.clone());
            }
            Node::Predicate(kind, name) => {
                if kind.starts_with("has_caveat:") {
                    // This reads a live state's grounds. An assignment can
                    // replace those grounds without changing the graph.
                    reads.names.insert(name.clone());
                } else {
                    reads.graph = true;
                }
                if kind == "observed" {
                    reads.observed.insert(name.clone());
                }
            }
            Node::Latest(_) | Node::HistoryCount(_) => reads.graph = true,
            Node::HistoryAt(_, index) => {
                reads.graph = true;
                index.collect_reads(reads);
            }
            Node::Fold(_, initial, _) => {
                reads.graph = true;
                initial.collect_reads(reads);
            }
            Node::ExpandedFold(_, initial, body) => {
                reads.graph = true;
                initial.collect_reads(reads);
                body.collect_reads(reads);
            }
            Node::Qualified(value, _, _) => {
                reads.graph = true;
                value.collect_reads(reads);
            }
            Node::Unary(_, child) => child.collect_reads(reads),
            Node::Binary(_, left, right) | Node::Require(left, right) => {
                left.collect_reads(reads);
                right.collect_reads(reads);
            }
            Node::If(condition, yes, no) => {
                condition.collect_reads(reads);
                yes.collect_reads(reads);
                no.collect_reads(reads);
            }
            Node::Function(_, arguments) => {
                for argument in arguments {
                    argument.collect_reads(reads);
                }
            }
            // An unexpanded call's body is unknown here, so it could read
            // anything. Sessions expand every call before evaluating.
            Node::UserCall(_, arguments) => {
                reads.anything = true;
                for argument in arguments {
                    argument.collect_reads(reads);
                }
            }
            // The body has the arguments inlined, so its reads are the call's.
            Node::ExpandedCall(arguments, body) => {
                for argument in arguments {
                    argument.collect_reads(reads);
                }
                body.collect_reads(reads);
            }
        }
    }

    /// The name, if this expression is a bare name.
    pub fn as_name(&self) -> Option<&str> {
        match &self.node {
            Node::Variable(name) => Some(name),
            _ => None,
        }
    }

    /// The same expression with names replaced: graph symbols wherever a
    /// predicate or `qualified` names one, and bare names, so that an argument
    /// can pass a name on. Histories are not renamed. Used to specialize a
    /// procedure for the symbols it is called with.
    pub fn rename_symbols(&self, names: &HashMap<String, String>) -> Expr {
        let rename = |name: &String| names.get(name).cloned().unwrap_or_else(|| name.clone());
        let child = |expression: &Expr| Box::new(expression.rename_symbols(names));
        let node = match &self.node {
            Node::Number(_)
            | Node::Bool(_)
            | Node::Text(_)
            | Node::Latest(_)
            | Node::HistoryCount(_) => self.node.clone(),
            Node::Variable(name) => Node::Variable(rename(name)),
            Node::Predicate(kind, target) => {
                let qualified = ["carries:", "has_caveat:"]
                    .into_iter()
                    .find_map(|prefix| kind.strip_prefix(prefix).map(|caveat| (prefix, caveat)));
                match qualified {
                    Some((prefix, caveat)) => Node::Predicate(
                        format!("{prefix}{}", rename(&caveat.to_string())),
                        rename(target),
                    ),
                    None => Node::Predicate(kind.clone(), rename(target)),
                }
            }
            Node::Qualified(value, evidence, caveats) => Node::Qualified(
                child(value),
                rename(evidence),
                caveats.iter().map(rename).collect(),
            ),
            Node::Unary(operator, operand) => Node::Unary(*operator, child(operand)),
            Node::Binary(operator, left, right) => {
                Node::Binary(*operator, child(left), child(right))
            }
            Node::Require(condition, value) => Node::Require(child(condition), child(value)),
            Node::If(condition, yes, no) => Node::If(child(condition), child(yes), child(no)),
            Node::HistoryAt(history, index) => Node::HistoryAt(history.clone(), child(index)),
            Node::Fold(history, initial, reducer) => {
                Node::Fold(history.clone(), child(initial), reducer.clone())
            }
            Node::ExpandedFold(history, initial, body) => {
                Node::ExpandedFold(history.clone(), child(initial), child(body))
            }
            Node::Function(function, arguments) => Node::Function(
                *function,
                arguments
                    .iter()
                    .map(|argument| argument.rename_symbols(names))
                    .collect(),
            ),
            Node::UserCall(name, arguments) => Node::UserCall(
                name.clone(),
                arguments
                    .iter()
                    .map(|argument| argument.rename_symbols(names))
                    .collect(),
            ),
            Node::ExpandedCall(arguments, body) => Node::ExpandedCall(
                arguments
                    .iter()
                    .map(|argument| argument.rename_symbols(names))
                    .collect(),
                child(body),
            ),
        };
        Expr {
            node,
            depth: self.depth,
        }
    }

    /// The operands of a chain of `and`, left to right; the expression itself
    /// if it is not one.
    pub fn conjuncts(&self) -> Vec<&Expr> {
        match &self.node {
            Node::Binary(Binary::And, left, right) => {
                let mut all = left.conjuncts();
                all.extend(right.conjuncts());
                all
            }
            _ => vec![self],
        }
    }

    /// Evidence that `qualified(...)` names wherever evaluating this
    /// expression is certain to reach it: not inside an `if` branch, the right
    /// side of `and` or `or`, a `require` value or a fold body.
    pub fn qualified_unconditionally(&self, evidence: &mut BTreeSet<String>) {
        match &self.node {
            Node::Qualified(value, name, _) => {
                evidence.insert(name.clone());
                value.qualified_unconditionally(evidence);
            }
            Node::Binary(Binary::And | Binary::Or, left, _)
            | Node::Require(left, _)
            | Node::If(left, _, _) => left.qualified_unconditionally(evidence),
            Node::Binary(_, left, right) => {
                left.qualified_unconditionally(evidence);
                right.qualified_unconditionally(evidence);
            }
            Node::Unary(_, child) | Node::HistoryAt(_, child) => {
                child.qualified_unconditionally(evidence)
            }
            Node::Function(_, arguments) | Node::UserCall(_, arguments) => {
                for argument in arguments {
                    argument.qualified_unconditionally(evidence);
                }
            }
            Node::ExpandedCall(arguments, body) => {
                for argument in arguments {
                    argument.qualified_unconditionally(evidence);
                }
                body.qualified_unconditionally(evidence);
            }
            Node::Fold(_, initial, _) | Node::ExpandedFold(_, initial, _) => {
                initial.qualified_unconditionally(evidence)
            }
            Node::Number(_)
            | Node::Bool(_)
            | Node::Text(_)
            | Node::Variable(_)
            | Node::Predicate(_, _)
            | Node::Latest(_)
            | Node::HistoryCount(_) => {}
        }
    }

    fn new(node: Node) -> Result<Self, String> {
        let depth = match &node {
            Node::Unary(_, child)
            | Node::Qualified(child, _, _)
            | Node::HistoryAt(_, child)
            | Node::Fold(_, child, _) => child.depth + 1,
            Node::ExpandedFold(_, initial, body) => initial.depth.max(body.depth) + 1,
            Node::Binary(_, left, right) => left.depth.max(right.depth) + 1,
            Node::Require(condition, value) => condition.depth.max(value.depth) + 1,
            Node::If(condition, yes, no) => condition.depth.max(yes.depth).max(no.depth) + 1,
            Node::Function(_, children) | Node::UserCall(_, children) => {
                children.iter().map(|child| child.depth).max().unwrap_or(0) + 1
            }
            Node::ExpandedCall(arguments, body) => {
                arguments
                    .iter()
                    .map(|argument| argument.depth)
                    .max()
                    .unwrap_or(0)
                    .max(body.depth)
                    + 1
            }
            _ => 1,
        };
        if depth > MAX_NESTING {
            return Err(format!("expression exceeds nesting limit {MAX_NESTING}"));
        }
        Ok(Self { node, depth })
    }

    /// Check every branch, including branches that may short-circuit at runtime.
    /// The host validates the graph node kind for each predicate target.
    pub fn validate(
        &self,
        numeric: &HashSet<String>,
        predicate: &impl Fn(&str, &str) -> Result<(), String>,
    ) -> Result<ValueType, String> {
        self.validate_calls(numeric, predicate, &|name, _| {
            Err(format!(
                "source function {name} must be expanded before validation"
            ))
        })
    }

    fn validate_calls(
        &self,
        numeric: &HashSet<String>,
        predicate: &dyn Fn(&str, &str) -> Result<(), String>,
        user_call: &dyn Fn(&str, usize) -> Result<ValueType, String>,
    ) -> Result<ValueType, String> {
        match &self.node {
            Node::Number(_) => Ok(ValueType::Number),
            Node::Bool(_) => Ok(ValueType::Bool),
            Node::Text(_) => Ok(ValueType::Text),
            Node::Variable(name) => {
                if numeric.contains(name) {
                    Ok(ValueType::Number)
                } else {
                    Err(format!("unknown numeric identifier {name}"))
                }
            }
            Node::Predicate(name, target) => {
                predicate(name, target)?;
                Ok(ValueType::Bool)
            }
            Node::Latest(name) => {
                predicate("numeric_history", name)?;
                Ok(ValueType::Number)
            }
            Node::HistoryCount(name) => {
                predicate("numeric_history", name)?;
                Ok(ValueType::Number)
            }
            Node::HistoryAt(name, index) => {
                predicate("numeric_history", name)?;
                index
                    .validate_calls(numeric, predicate, user_call)?
                    .require(ValueType::Number)?;
                Ok(ValueType::Number)
            }
            Node::Fold(name, initial, reducer) => {
                predicate("numeric_history", name)?;
                initial
                    .validate_calls(numeric, predicate, user_call)?
                    .require(ValueType::Number)?;
                user_call(reducer, 2)?.require(ValueType::Number)?;
                Ok(ValueType::Number)
            }
            Node::ExpandedFold(name, initial, body) => {
                predicate("numeric_history", name)?;
                initial
                    .validate_calls(numeric, predicate, user_call)?
                    .require(ValueType::Number)?;
                body.validate_calls(
                    &HashSet::from([FOLD_ACC.into(), FOLD_VALUE.into()]),
                    &|_, _| Err("fold reducer must be pure".into()),
                    user_call,
                )?
                .require(ValueType::Number)?;
                Ok(ValueType::Number)
            }
            Node::Qualified(value, evidence, caveats) => {
                value
                    .validate_calls(numeric, predicate, user_call)?
                    .require(ValueType::Number)?;
                predicate("qualification_evidence", evidence)?;
                for caveat in caveats {
                    predicate("qualification_caveat", caveat)?;
                }
                Ok(ValueType::Number)
            }
            Node::If(condition, yes, no) => {
                condition
                    .validate_calls(numeric, predicate, user_call)?
                    .require(ValueType::Bool)?;
                let kind = yes.validate_calls(numeric, predicate, user_call)?;
                no.validate_calls(numeric, predicate, user_call)?
                    .require(kind)?;
                Ok(kind)
            }
            Node::Require(condition, value) => {
                condition
                    .validate_calls(numeric, predicate, user_call)?
                    .require(ValueType::Bool)?;
                value.validate_calls(numeric, predicate, user_call)
            }
            Node::Unary(operator, child) => {
                let expected = match operator {
                    Unary::Not => ValueType::Bool,
                    _ => ValueType::Number,
                };
                child
                    .validate_calls(numeric, predicate, user_call)?
                    .require(expected)?;
                Ok(expected)
            }
            Node::Binary(operator, left, right) => {
                let left_type = left.validate_calls(numeric, predicate, user_call)?;
                let right_type = right.validate_calls(numeric, predicate, user_call)?;
                match operator {
                    Binary::And | Binary::Or => {
                        left_type.require(ValueType::Bool)?;
                        right_type.require(ValueType::Bool)?;
                        Ok(ValueType::Bool)
                    }
                    Binary::Equal | Binary::NotEqual => {
                        right_type.require(left_type)?;
                        Ok(ValueType::Bool)
                    }
                    Binary::Add => {
                        if !matches!(left_type, ValueType::Number | ValueType::Text) {
                            return Err("addition requires two numbers or two texts".into());
                        }
                        right_type.require(left_type)?;
                        Ok(left_type)
                    }
                    _ => {
                        left_type.require(ValueType::Number)?;
                        right_type.require(ValueType::Number)?;
                        match operator {
                            Binary::Subtract | Binary::Multiply | Binary::Divide => {
                                Ok(ValueType::Number)
                            }
                            _ => Ok(ValueType::Bool),
                        }
                    }
                }
            }
            Node::Function(_, arguments)
            | Node::UserCall(_, arguments)
            | Node::ExpandedCall(arguments, _) => {
                for argument in arguments {
                    argument
                        .validate_calls(numeric, predicate, user_call)?
                        .require(ValueType::Number)?;
                }
                match &self.node {
                    Node::UserCall(name, _) => user_call(name, arguments.len()),
                    Node::ExpandedCall(_, body) => {
                        body.validate_calls(numeric, predicate, user_call)
                    }
                    Node::Function(Function::Text, _) => Ok(ValueType::Text),
                    _ => Ok(ValueType::Number),
                }
            }
        }
    }

    /// Evaluate against one host snapshot. Boolean operators short-circuit;
    /// every numeric value, including host reads, must remain finite.
    pub fn evaluate(
        &self,
        numbers: &impl Fn(&str) -> Option<f64>,
        predicate: &impl Fn(&str, &str) -> Result<bool, String>,
    ) -> Result<Value, String> {
        self.evaluate_tracked(
            &|name| Ok(numbers(name).map(Tracked::plain)),
            &|kind, name| predicate(kind, name).map(Tracked::plain),
            &|_, _| Err("qualified values require a tracked evaluation host".into()),
        )
        .map(|tracked| tracked.value)
    }

    /// Preserve the sorted union of every dependency actually read. This
    /// includes ignored eager function arguments, but excludes short-circuited
    /// operands. The host supplies typed graph provenance and authorizes
    /// explicit qualification; expression evaluation never asserts truth.
    pub fn evaluate_tracked(
        &self,
        numbers: &impl Fn(&str) -> Result<Option<Tracked<f64>>, String>,
        predicate: &impl Fn(&str, &str) -> Result<Tracked<bool>, String>,
        qualify: &impl Fn(&str, &[String]) -> Result<Provenance, String>,
    ) -> Result<Tracked<Value>, String> {
        self.evaluate_tracked_with_history(numbers, predicate, qualify, &|name| {
            Err(format!(
                "latest({name}) requires a history-aware evaluation host"
            ))
        })
    }

    /// Read the latest immutable occurrence selected by the host, retaining
    /// its numeric dependencies. Empty histories must be reported by the host
    /// as errors; a stream name never silently becomes a numeric variable.
    pub fn evaluate_tracked_with_history(
        &self,
        numbers: &impl Fn(&str) -> Result<Option<Tracked<f64>>, String>,
        predicate: &impl Fn(&str, &str) -> Result<Tracked<bool>, String>,
        qualify: &impl Fn(&str, &[String]) -> Result<Provenance, String>,
        latest: &impl Fn(&str) -> Result<Tracked<f64>, String>,
    ) -> Result<Tracked<Value>, String> {
        self.evaluate_tracked_with_histories(
            numbers,
            predicate,
            qualify,
            &|name, query| match query {
                HistoryRead::Latest => latest(name),
                _ => Err("history operation requires an indexed history host".into()),
            },
        )
    }

    /// Evaluate bounded history computations without exposing mutable archives.
    pub fn evaluate_tracked_with_histories(
        &self,
        numbers: &impl Fn(&str) -> Result<Option<Tracked<f64>>, String>,
        predicate: &impl Fn(&str, &str) -> Result<Tracked<bool>, String>,
        qualify: &impl Fn(&str, &[String]) -> Result<Provenance, String>,
        history: &impl Fn(&str, HistoryRead) -> Result<Tracked<f64>, String>,
    ) -> Result<Tracked<Value>, String> {
        // The existing evaluator already visits precisely the operands that
        // contribute to this evaluation. Accumulating at those reads gives
        // identical propagation without copying full traces at every AST node.
        let provenance = RefCell::new(Provenance::default());
        let value = self.evaluate_values(
            &|name| {
                let Some(tracked) = numbers(name)? else {
                    return Ok(None);
                };
                let value = finite(tracked.value)?;
                provenance.borrow_mut().merge(&tracked.provenance)?;
                Ok(Some(value))
            },
            &|kind, name| {
                let tracked = predicate(kind, name)?;
                provenance.borrow_mut().merge(&tracked.provenance)?;
                Ok(tracked.value)
            },
            &|evidence, caveats| provenance.borrow_mut().merge(&qualify(evidence, caveats)?),
            &|name, query| {
                let tracked = history(name, query)?;
                let value = finite(tracked.value)?;
                provenance.borrow_mut().merge(&tracked.provenance)?;
                Ok(value)
            },
            &Cell::new(MAX_EVALUATED_NODES),
        )?;
        Ok(Tracked {
            value,
            provenance: provenance.into_inner(),
        })
    }

    fn evaluate_values(
        &self,
        numbers: &dyn Fn(&str) -> Result<Option<f64>, String>,
        predicate: &dyn Fn(&str, &str) -> Result<bool, String>,
        qualify: &QualificationSink<'_>,
        history: &dyn Fn(&str, HistoryRead) -> Result<f64, String>,
        remaining: &Cell<usize>,
    ) -> Result<Value, String> {
        let budget = remaining.get();
        if budget == 0 {
            return Err(format!(
                "expression exceeds evaluation limit {MAX_EVALUATED_NODES} nodes"
            ));
        }
        remaining.set(budget - 1);
        match &self.node {
            Node::Number(value) => Ok(Value::Number(value.value())),
            Node::Bool(value) => Ok(Value::Bool(*value)),
            Node::Text(value) => {
                check_text_size(value.len())?;
                Ok(Value::Text(value.clone()))
            }
            Node::Variable(name) => {
                let value =
                    numbers(name)?.ok_or_else(|| format!("unknown numeric identifier {name}"))?;
                Ok(Value::Number(finite(value)?))
            }
            Node::Predicate(name, target) => Ok(Value::Bool(predicate(name, target)?)),
            Node::Latest(name) => Ok(Value::Number(finite(history(name, HistoryRead::Latest)?)?)),
            Node::HistoryCount(name) => {
                Ok(Value::Number(finite(history(name, HistoryRead::Count)?)?))
            }
            Node::HistoryAt(name, index) => {
                let index = index
                    .evaluate_values(numbers, predicate, qualify, history, remaining)?
                    .number()?;
                if index < 0.0 || index.fract() != 0.0 || index >= MAX_FOLD_RECORDS as f64 {
                    return Err("history index must be an integer in 0..256".into());
                }
                Ok(Value::Number(finite(history(
                    name,
                    HistoryRead::At(index as usize),
                )?)?))
            }
            Node::Fold(_, _, _) => Err("fold reducer must be expanded before evaluation".into()),
            Node::ExpandedFold(name, initial, body) => {
                let mut accumulator = initial
                    .evaluate_values(numbers, predicate, qualify, history, remaining)?
                    .number()?;
                let count = finite(history(name, HistoryRead::Count)?)?;
                if count < 0.0 || count.fract() != 0.0 || count > MAX_FOLD_RECORDS as f64 {
                    return Err("history fold count exceeds bounded history capacity".into());
                }
                for index in 0..count as usize {
                    // Eager record reads retain provenance even if the reducer
                    // ignores an argument or multiplies it by zero.
                    let value = finite(history(name, HistoryRead::At(index))?)?;
                    accumulator = body
                        .evaluate_values(
                            &|variable| {
                                Ok(match variable {
                                    FOLD_ACC => Some(accumulator),
                                    FOLD_VALUE => Some(value),
                                    _ => None,
                                })
                            },
                            predicate,
                            qualify,
                            history,
                            remaining,
                        )?
                        .number()?;
                }
                Ok(Value::Number(accumulator))
            }
            Node::Qualified(value, evidence, caveats) => {
                let value = value
                    .evaluate_values(numbers, predicate, qualify, history, remaining)?
                    .number()?;
                qualify(evidence, caveats)?;
                Ok(Value::Number(value))
            }
            Node::If(condition, yes, no) => {
                if condition
                    .evaluate_values(numbers, predicate, qualify, history, remaining)?
                    .boolean()?
                {
                    yes.evaluate_values(numbers, predicate, qualify, history, remaining)
                } else {
                    no.evaluate_values(numbers, predicate, qualify, history, remaining)
                }
            }
            Node::Require(condition, value) => {
                if !condition
                    .evaluate_values(numbers, predicate, qualify, history, remaining)?
                    .boolean()?
                {
                    return Err("source expression requirement failed".into());
                }
                value.evaluate_values(numbers, predicate, qualify, history, remaining)
            }
            Node::UserCall(name, _) => Err(format!(
                "source function {name} must be expanded before evaluation"
            )),
            Node::ExpandedCall(arguments, body) => {
                for argument in arguments {
                    argument
                        .evaluate_values(numbers, predicate, qualify, history, remaining)?
                        .number()?;
                }
                body.evaluate_values(numbers, predicate, qualify, history, remaining)
            }
            Node::Unary(operator, child) => {
                let value =
                    child.evaluate_values(numbers, predicate, qualify, history, remaining)?;
                match operator {
                    Unary::Positive => Ok(Value::Number(value.number()?)),
                    Unary::Negative => Ok(Value::Number(-value.number()?)),
                    Unary::Not => Ok(Value::Bool(!value.boolean()?)),
                }
            }
            Node::Binary(operator, left, right) => {
                let left = left.evaluate_values(numbers, predicate, qualify, history, remaining)?;
                match operator {
                    Binary::And if !left.boolean()? => return Ok(Value::Bool(false)),
                    Binary::Or if left.boolean()? => return Ok(Value::Bool(true)),
                    _ => {}
                }
                let right =
                    right.evaluate_values(numbers, predicate, qualify, history, remaining)?;
                match operator {
                    Binary::And | Binary::Or => Ok(Value::Bool(right.boolean()?)),
                    Binary::Equal | Binary::NotEqual => {
                        let equal = match (left, right) {
                            (Value::Number(left), Value::Number(right)) => left == right,
                            (Value::Bool(left), Value::Bool(right)) => left == right,
                            (Value::Text(left), Value::Text(right)) => left == right,
                            _ => return Err("equality requires operands of the same type".into()),
                        };
                        Ok(Value::Bool(if *operator == Binary::Equal {
                            equal
                        } else {
                            !equal
                        }))
                    }
                    Binary::Add => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(finite(left + right)?))
                        }
                        (Value::Text(mut left), Value::Text(right)) => {
                            check_text_size(
                                left.len()
                                    .checked_add(right.len())
                                    .ok_or("expression text size overflow")?,
                            )?;
                            left.push_str(&right);
                            Ok(Value::Text(left))
                        }
                        _ => Err("addition requires two numbers or two texts".into()),
                    },
                    _ => {
                        let left = left.number()?;
                        let right = right.number()?;
                        let result = match operator {
                            Binary::Subtract => left - right,
                            Binary::Multiply => left * right,
                            Binary::Divide => {
                                if right == 0.0 {
                                    return Err("division by zero in expression".into());
                                }
                                left / right
                            }
                            Binary::Less => return Ok(Value::Bool(left < right)),
                            Binary::LessEqual => return Ok(Value::Bool(left <= right)),
                            Binary::Greater => return Ok(Value::Bool(left > right)),
                            Binary::GreaterEqual => return Ok(Value::Bool(left >= right)),
                            _ => unreachable!("boolean and equality operators handled above"),
                        };
                        Ok(Value::Number(finite(result)?))
                    }
                }
            }
            Node::Function(function, arguments) => {
                let arguments = arguments
                    .iter()
                    .map(|argument| {
                        argument
                            .evaluate_values(numbers, predicate, qualify, history, remaining)?
                            .number()
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let result = match function {
                    Function::Sin => arguments[0].sin(),
                    Function::Cos => arguments[0].cos(),
                    Function::Sqrt => {
                        if arguments[0] < 0.0 {
                            return Err("sqrt requires a nonnegative number".into());
                        }
                        arguments[0].sqrt()
                    }
                    Function::Atan2 => arguments[0].atan2(arguments[1]),
                    Function::Floor => arguments[0].floor(),
                    Function::Ceil => arguments[0].ceil(),
                    Function::Round => arguments[0].round(),
                    Function::Text => {
                        return Ok(Value::Text(if arguments[0] == 0.0 {
                            "0".into()
                        } else {
                            arguments[0].to_string()
                        }))
                    }
                };
                Ok(Value::Number(finite(result)?))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Number(Number),
    Identifier(String),
    Bool(bool),
    Text(String),
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Not,
    LeftParen,
    RightParen,
    Comma,
}

struct Token {
    kind: TokenKind,
    offset: usize,
}

fn identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn identifier_continue(byte: u8) -> bool {
    identifier_start(byte) || byte.is_ascii_digit()
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    if input.len() > MAX_SOURCE_BYTES {
        return Err(format!(
            "expression exceeds source limit {MAX_SOURCE_BYTES} bytes"
        ));
    }
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let byte = bytes[offset];
        if byte.is_ascii_whitespace() {
            offset += 1;
            continue;
        }
        if tokens.len() == MAX_TOKENS {
            return Err(format!("expression exceeds token limit {MAX_TOKENS}"));
        }
        let start = offset;
        let kind = if byte == b'"' {
            offset += 1;
            let mut escaped = false;
            let mut closed = false;
            while offset < bytes.len() {
                let current = bytes[offset];
                offset += 1;
                if escaped {
                    escaped = false;
                } else if current == b'\\' {
                    escaped = true;
                } else if current == b'"' {
                    closed = true;
                    break;
                }
            }
            if !closed {
                return Err(format!("unterminated expression text at byte {start}"));
            }
            let value = crate::parser::quoted(&input[start..offset])?;
            check_text_size(value.len())?;
            TokenKind::Text(value)
        } else if identifier_start(byte) {
            offset += 1;
            while offset < bytes.len() && identifier_continue(bytes[offset]) {
                offset += 1;
            }
            while bytes.get(offset) == Some(&b'.') {
                offset += 1;
                if !bytes.get(offset).copied().is_some_and(identifier_start) {
                    return Err(format!("expected identifier after '.' at byte {offset}"));
                }
                offset += 1;
                while offset < bytes.len() && identifier_continue(bytes[offset]) {
                    offset += 1;
                }
            }
            match &input[start..offset] {
                "and" => TokenKind::And,
                "or" => TokenKind::Or,
                "not" => TokenKind::Not,
                "true" => TokenKind::Bool(true),
                "false" => TokenKind::Bool(false),
                name => TokenKind::Identifier(name.into()),
            }
        } else if byte.is_ascii_digit()
            || (byte == b'.' && bytes.get(offset + 1).is_some_and(u8::is_ascii_digit))
        {
            while offset < bytes.len() && bytes[offset].is_ascii_digit() {
                offset += 1;
            }
            if bytes.get(offset) == Some(&b'.') {
                offset += 1;
                while offset < bytes.len() && bytes[offset].is_ascii_digit() {
                    offset += 1;
                }
            }
            if matches!(bytes.get(offset), Some(b'e' | b'E')) {
                offset += 1;
                if matches!(bytes.get(offset), Some(b'+' | b'-')) {
                    offset += 1;
                }
                let exponent_start = offset;
                while offset < bytes.len() && bytes[offset].is_ascii_digit() {
                    offset += 1;
                }
                if offset == exponent_start {
                    return Err(format!("expected exponent digits at byte {offset}"));
                }
            }
            let literal = &input[start..offset];
            TokenKind::Number(Number::parse(literal).map_err(|_| {
                format!("expression literal must be a finite number at byte {start}: {literal}")
            })?)
        } else {
            offset += 1;
            match byte {
                b'+' => TokenKind::Plus,
                b'-' => TokenKind::Minus,
                b'*' => TokenKind::Star,
                b'/' => TokenKind::Slash,
                b'(' => TokenKind::LeftParen,
                b')' => TokenKind::RightParen,
                b',' => TokenKind::Comma,
                b'=' | b'!' => {
                    if bytes.get(offset) != Some(&b'=') {
                        return Err(format!("expected '=' at byte {offset}"));
                    }
                    offset += 1;
                    if byte == b'=' {
                        TokenKind::Equal
                    } else {
                        TokenKind::NotEqual
                    }
                }
                b'<' | b'>' => {
                    let equal = bytes.get(offset) == Some(&b'=');
                    offset += usize::from(equal);
                    match (byte, equal) {
                        (b'<', false) => TokenKind::Less,
                        (b'<', true) => TokenKind::LessEqual,
                        (_, false) => TokenKind::Greater,
                        (_, true) => TokenKind::GreaterEqual,
                    }
                }
                _ => {
                    return Err(format!(
                        "unexpected character {:?} at byte {start}",
                        input[start..].chars().next().unwrap()
                    ));
                }
            }
        };
        tokens.push(Token {
            kind,
            offset: start,
        });
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    end: usize,
    allow_user_functions: bool,
}

impl Parser {
    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.cursor).map(|token| &token.kind)
    }

    fn take(&mut self) -> Option<TokenKind> {
        let token = self.peek()?.clone();
        self.cursor += 1;
        Some(token)
    }

    fn offset(&self) -> usize {
        self.tokens
            .get(self.cursor)
            .map_or(self.end, |token| token.offset)
    }

    fn expect(&mut self, expected: TokenKind, description: &str) -> Result<(), String> {
        if self.peek() == Some(&expected) {
            self.cursor += 1;
            Ok(())
        } else {
            Err(format!("expected {description} at byte {}", self.offset()))
        }
    }

    fn expression(&mut self, minimum: u8, nesting: usize) -> Result<Expr, String> {
        if nesting > MAX_NESTING {
            return Err(format!("expression exceeds nesting limit {MAX_NESTING}"));
        }
        let offset = self.offset();
        let node = match self.take() {
            Some(TokenKind::Number(number)) => Node::Number(number),
            Some(TokenKind::Bool(value)) => Node::Bool(value),
            Some(TokenKind::Text(value)) => Node::Text(value),
            Some(TokenKind::Identifier(name)) => {
                if self.peek() == Some(&TokenKind::LeftParen) {
                    self.cursor += 1;
                    self.call(name, nesting + 1)?
                } else {
                    Node::Variable(name)
                }
            }
            Some(TokenKind::Plus) => {
                Node::Unary(Unary::Positive, Box::new(self.expression(13, nesting + 1)?))
            }
            Some(TokenKind::Minus) => {
                Node::Unary(Unary::Negative, Box::new(self.expression(13, nesting + 1)?))
            }
            Some(TokenKind::Not) => {
                // `not heat > 10` means `not (heat > 10)`; `and` and `or`
                // retain lower precedence than the negation.
                Node::Unary(Unary::Not, Box::new(self.expression(5, nesting + 1)?))
            }
            Some(TokenKind::LeftParen) => {
                let nested = self.expression(0, nesting + 1)?;
                self.expect(TokenKind::RightParen, "')'")?;
                nested.node
            }
            _ => return Err(format!("expected expression at byte {offset}")),
        };
        let mut left = Expr::new(node)?;
        while let Some((operator, binding)) = self.peek().and_then(binary) {
            if binding < minimum {
                break;
            }
            self.cursor += 1;
            let right = self.expression(binding + 1, nesting)?;
            left = Expr::new(Node::Binary(operator, Box::new(left), Box::new(right)))?;
        }
        Ok(left)
    }

    fn call(&mut self, name: String, nesting: usize) -> Result<Node, String> {
        if nesting > MAX_NESTING {
            return Err(format!("expression exceeds nesting limit {MAX_NESTING}"));
        }
        if matches!(name.as_str(), "if" | "require") {
            let condition = self.expression(0, nesting)?;
            self.expect(TokenKind::Comma, "',' after condition")?;
            let value = self.expression(0, nesting)?;
            let node = if name == "if" {
                self.expect(TokenKind::Comma, "',' before alternative")?;
                Node::If(
                    Box::new(condition),
                    Box::new(value),
                    Box::new(self.expression(0, nesting)?),
                )
            } else {
                Node::Require(Box::new(condition), Box::new(value))
            };
            self.expect(TokenKind::RightParen, "')' after conditional expression")?;
            return Ok(node);
        }
        if name == "qualified" {
            let value = self.expression(0, nesting)?;
            self.expect(TokenKind::Comma, "',' before qualification evidence")?;
            let evidence = self.graph_identifier("qualified evidence")?;
            let mut caveats = Vec::new();
            while self.peek() == Some(&TokenKind::Comma) {
                self.cursor += 1;
                caveats.push(self.graph_identifier("qualified caveat")?);
            }
            self.expect(TokenKind::RightParen, "')' after qualified value")?;
            return Ok(Node::Qualified(Box::new(value), evidence, caveats));
        }
        if matches!(
            name.as_str(),
            "history_count" | "history_at" | "fold_history"
        ) {
            let history = self.graph_identifier("history")?;
            let node = if name == "history_count" {
                Node::HistoryCount(history)
            } else {
                self.expect(TokenKind::Comma, "',' after history")?;
                let value = self.expression(0, nesting)?;
                if name == "history_at" {
                    Node::HistoryAt(history, Box::new(value))
                } else {
                    self.expect(TokenKind::Comma, "',' before source reducer")?;
                    let reducer = self.graph_identifier("source reducer")?;
                    Node::Fold(history, Box::new(value), reducer)
                }
            };
            self.expect(TokenKind::RightParen, "')' after history operation")?;
            return Ok(node);
        }
        // carries(EVIDENCE, CAVEAT): a predicate on the evidence. The caveat
        // travels in the predicate's name. See spec/caveat-renewal-0.1.md.
        if name == "carries" {
            let evidence = self.graph_identifier("carries evidence")?;
            self.expect(TokenKind::Comma, "',' before the carried caveat")?;
            let caveat = self.graph_identifier("carries caveat")?;
            self.expect(TokenKind::RightParen, "')' after the carried caveat")?;
            return Ok(Node::Predicate(format!("carries:{caveat}"), evidence));
        }
        if name == "has_caveat" {
            let state = self.graph_identifier("has_caveat state")?;
            self.expect(TokenKind::Comma, "',' before the state's caveat")?;
            let caveat = self.graph_identifier("has_caveat caveat")?;
            self.expect(TokenKind::RightParen, "')' after the state's caveat")?;
            return Ok(Node::Predicate(format!("has_caveat:{caveat}"), state));
        }
        if name == "latest" {
            let history = self.graph_identifier("latest history")?;
            self.expect(TokenKind::RightParen, "')' after history identifier")?;
            return Ok(Node::Latest(history));
        }
        if predicate_name(&name) {
            let offset = self.offset();
            let Some(TokenKind::Identifier(target)) = self.take() else {
                return Err(format!(
                    "{name} requires a graph identifier at byte {offset}"
                ));
            };
            if target.contains('.') {
                return Err(format!(
                    "{name} requires a plain graph identifier at byte {offset}"
                ));
            }
            self.expect(TokenKind::RightParen, "')' after predicate target")?;
            return Ok(Node::Predicate(name, target));
        }
        let function = Function::named(&name);
        if function.is_none() && (!self.allow_user_functions || !plain_identifier(&name)) {
            return Err(format!("unknown expression function {name}"));
        }
        let mut arguments = Vec::new();
        if self.peek() != Some(&TokenKind::RightParen) {
            loop {
                if function.is_some_and(|function| arguments.len() == function.arity()) {
                    let function = function.unwrap();
                    return Err(format!(
                        "{} requires {} arguments",
                        function.name(),
                        function.arity()
                    ));
                }
                arguments.push(self.expression(0, nesting)?);
                if self.peek() != Some(&TokenKind::Comma) {
                    break;
                }
                self.cursor += 1;
            }
        }
        self.expect(TokenKind::RightParen, "')' after function arguments")?;
        if let Some(function) = function {
            if arguments.len() != function.arity() {
                return Err(format!(
                    "{} requires {} arguments",
                    function.name(),
                    function.arity()
                ));
            }
            Ok(Node::Function(function, arguments))
        } else {
            Ok(Node::UserCall(name, arguments))
        }
    }

    fn graph_identifier(&mut self, context: &str) -> Result<String, String> {
        let offset = self.offset();
        match self.take() {
            Some(TokenKind::Identifier(name)) if plain_identifier(&name) => Ok(name),
            _ => Err(format!(
                "{context} requires a plain graph identifier at byte {offset}"
            )),
        }
    }
}

fn binary(token: &TokenKind) -> Option<(Binary, u8)> {
    Some(match token {
        TokenKind::Or => (Binary::Or, 1),
        TokenKind::And => (Binary::And, 3),
        TokenKind::Equal => (Binary::Equal, 5),
        TokenKind::NotEqual => (Binary::NotEqual, 5),
        TokenKind::Less => (Binary::Less, 7),
        TokenKind::LessEqual => (Binary::LessEqual, 7),
        TokenKind::Greater => (Binary::Greater, 7),
        TokenKind::GreaterEqual => (Binary::GreaterEqual, 7),
        TokenKind::Plus => (Binary::Add, 9),
        TokenKind::Minus => (Binary::Subtract, 9),
        TokenKind::Star => (Binary::Multiply, 11),
        TokenKind::Slash => (Binary::Divide, 11),
        _ => return None,
    })
}

/// Parse at most 1024 tokens and 64 levels of nesting or AST depth.
/// Numeric names may contain dotted ASCII identifier segments, such as
/// `reef_one.x`; predicate targets are plain graph identifiers.
pub fn parse(input: &str) -> Result<Expr, String> {
    parse_mode(input, false)
}

/// Parse an expression before all source function declarations are known.
/// Call `expand` before ordinary validation or evaluation.
pub fn parse_unresolved(input: &str) -> Result<Expr, String> {
    parse_mode(input, true)
}

/// Parse a procedure call with the expression tokenizer, keeping nested commas,
/// quoted text and delimiter errors consistent with ordinary expressions.
pub(crate) fn parse_procedure_call(input: &str) -> Result<(String, Vec<Expr>), String> {
    let mut parser = Parser {
        tokens: tokenize(input)?,
        cursor: 0,
        end: input.len(),
        allow_user_functions: true,
    };
    let name = parser.graph_identifier("procedure call")?;
    parser.expect(TokenKind::LeftParen, "'(' after procedure name")?;
    let mut arguments = Vec::new();
    if parser.peek() != Some(&TokenKind::RightParen) {
        loop {
            arguments.push(parser.expression(0, 0)?);
            if parser.peek() != Some(&TokenKind::Comma) {
                break;
            }
            parser.cursor += 1;
        }
    }
    parser.expect(TokenKind::RightParen, "')' after procedure arguments")?;
    if parser.peek().is_some() {
        return Err(format!("unexpected token at byte {}", parser.offset()));
    }
    Ok((name, arguments))
}

fn parse_mode(input: &str, allow_user_functions: bool) -> Result<Expr, String> {
    let mut parser = Parser {
        tokens: tokenize(input)?,
        cursor: 0,
        end: input.len(),
        allow_user_functions,
    };
    let expression = parser.expression(0, 0)?;
    if parser.peek().is_some() {
        return Err(format!("unexpected token at byte {}", parser.offset()));
    }
    Ok(expression)
}

fn plain_identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(identifier_start)
        && bytes.all(identifier_continue)
        && !matches!(name, "true" | "false" | "and" | "or" | "not")
}

fn predicate_name(name: &str) -> bool {
    matches!(
        name,
        "observed" | "examined" | "committed" | "reopened" | "has_sample"
    )
}

fn check_arity(definition: &FunctionDef, supplied: usize) -> Result<(), String> {
    if definition.parameters.len() == supplied {
        Ok(())
    } else {
        Err(format!(
            "source function {} requires {} arguments, found {supplied}",
            definition.name,
            definition.parameters.len()
        ))
    }
}

/// Validate every definition, including unused functions. Functions accept
/// numeric arguments and infer a number, boolean, or text result. They capture
/// no state or graph data and may not recurse. Constants are literals; callers
/// pass named coordinates explicitly as arguments.
pub fn validate_functions(functions: &BTreeMap<String, FunctionDef>) -> Result<(), String> {
    if functions.len() > MAX_FUNCTIONS {
        return Err(format!("source exceeds function limit {MAX_FUNCTIONS}"));
    }
    for (name, definition) in functions {
        if name != &definition.name || !plain_identifier(name) {
            return Err(format!("invalid source function name {}", definition.name));
        }
        if Function::named(name).is_some()
            || predicate_name(name)
            || matches!(
                name.as_str(),
                "qualified"
                    | "has_caveat"
                    | "if"
                    | "require"
                    | "latest"
                    | "history_count"
                    | "history_at"
                    | "fold_history"
            )
        {
            return Err(format!(
                "source function {name} shadows an intrinsic or predicate"
            ));
        }
        let mut parameters = HashSet::new();
        for parameter in &definition.parameters {
            if !plain_identifier(parameter) {
                return Err(format!(
                    "invalid parameter {parameter} in source function {name}"
                ));
            }
            if !parameters.insert(parameter.clone()) {
                return Err(format!(
                    "duplicate parameter {parameter} in source function {name}"
                ));
            }
        }
    }
    let mut result_types = BTreeMap::new();
    let mut active = HashSet::new();
    for name in functions.keys() {
        infer_function(name, functions, &mut result_types, &mut active)?;
    }
    // Check expansion limits for unused definitions as well. Each body is
    // checked separately, so a large unused macro cannot escape validation.
    for (name, definition) in functions {
        expand(&definition.body, functions)
            .map_err(|error| format!("source function {name}: {error}"))?;
    }
    Ok(())
}

fn collect_calls<'a>(expression: &'a Expr, calls: &mut Vec<&'a str>) {
    match &expression.node {
        Node::Unary(_, child) | Node::Qualified(child, _, _) | Node::HistoryAt(_, child) => {
            collect_calls(child, calls)
        }
        Node::Fold(_, initial, reducer) => {
            calls.push(reducer);
            collect_calls(initial, calls);
        }
        Node::ExpandedFold(_, initial, body) => {
            collect_calls(initial, calls);
            collect_calls(body, calls);
        }
        Node::Binary(_, left, right) => {
            collect_calls(left, calls);
            collect_calls(right, calls);
        }
        Node::Require(condition, value) => {
            collect_calls(condition, calls);
            collect_calls(value, calls);
        }
        Node::If(condition, yes, no) => {
            collect_calls(condition, calls);
            collect_calls(yes, calls);
            collect_calls(no, calls);
        }
        Node::Function(_, arguments) | Node::UserCall(_, arguments) => {
            if let Node::UserCall(name, _) = &expression.node {
                calls.push(name);
            }
            for argument in arguments {
                collect_calls(argument, calls);
            }
        }
        Node::ExpandedCall(arguments, body) => {
            for argument in arguments {
                collect_calls(argument, calls);
            }
            collect_calls(body, calls);
        }
        _ => {}
    }
}

fn infer_function(
    name: &str,
    functions: &BTreeMap<String, FunctionDef>,
    result_types: &mut BTreeMap<String, ValueType>,
    active: &mut HashSet<String>,
) -> Result<ValueType, String> {
    if active.contains(name) {
        return Err(format!("recursive source function cycle involving {name}"));
    }
    if let Some(kind) = result_types.get(name) {
        return Ok(*kind);
    }
    let definition = functions
        .get(name)
        .ok_or_else(|| format!("unknown source function {name}"))?;
    active.insert(name.into());
    let mut calls = Vec::new();
    collect_calls(&definition.body, &mut calls);
    for callee in calls {
        infer_function(callee, functions, result_types, active)?;
    }
    let parameters = definition.parameters.iter().cloned().collect();
    let kind = definition
        .body
        .validate_calls(
            &parameters,
            &|_, _| Err("pure source functions cannot query graph predicates".into()),
            &|called, arity| {
                let callee = functions
                    .get(called)
                    .ok_or_else(|| format!("unknown source function {called}"))?;
                check_arity(callee, arity)?;
                result_types
                    .get(called)
                    .copied()
                    .ok_or_else(|| format!("source function {called} has no inferred result type"))
            },
        )
        .map_err(|error| format!("source function {name}: {error}"))?;
    active.remove(name);
    result_types.insert(name.into(), kind);
    Ok(kind)
}

struct Expander<'a> {
    functions: &'a BTreeMap<String, FunctionDef>,
    /// Named expressions over state and the graph, inlined where their name
    /// is read. See spec/caveat-define-0.1.md.
    defines: &'a BTreeMap<String, Expr>,
    active: Vec<String>,
    remaining: usize,
}

impl Expander<'_> {
    fn build(&mut self, node: Node) -> Result<Expr, String> {
        if self.remaining == 0 {
            return Err(format!(
                "expression exceeds expansion limit {MAX_EXPANDED_NODES} nodes"
            ));
        }
        self.remaining -= 1;
        Expr::new(node)
    }

    fn walk(
        &mut self,
        expression: &Expr,
        parameters: Option<&BTreeMap<String, Expr>>,
    ) -> Result<Expr, String> {
        let node = match &expression.node {
            Node::Number(number) => Node::Number(number.clone()),
            Node::Bool(value) => Node::Bool(*value),
            Node::Text(value) => Node::Text(value.clone()),
            Node::Variable(name) => {
                if let Some(parameters) = parameters {
                    let argument = parameters
                        .get(name)
                        .ok_or_else(|| format!("pure source function cannot capture {name}"))?;
                    // The argument is already expanded in its caller's scope.
                    // Do not substitute again if a caller name matches a formal.
                    return self.walk(argument, None);
                }
                if let Some(body) = self.defines.get(name) {
                    if self.active.contains(name) {
                        return Err(format!("define {name} refers to itself"));
                    }
                    self.active.push(name.clone());
                    let expanded = self.walk(body, None);
                    self.active.pop();
                    return expanded.map_err(|error| format!("define {name}: {error}"));
                }
                Node::Variable(name.clone())
            }
            Node::Predicate(name, target) => {
                if parameters.is_some() {
                    return Err("pure source functions cannot query graph predicates".into());
                }
                Node::Predicate(name.clone(), target.clone())
            }
            Node::HistoryCount(name) => {
                if parameters.is_some() {
                    return Err("pure source functions cannot capture numeric history".into());
                }
                Node::HistoryCount(name.clone())
            }
            Node::HistoryAt(name, index) => {
                if parameters.is_some() {
                    return Err("pure source functions cannot capture numeric history".into());
                }
                Node::HistoryAt(name.clone(), Box::new(self.walk(index, None)?))
            }
            Node::Fold(name, initial, reducer) => {
                if parameters.is_some() {
                    return Err("pure source functions cannot capture numeric history".into());
                }
                let initial = self.walk(initial, None)?;
                let call = Expr::new(Node::UserCall(
                    reducer.clone(),
                    vec![
                        Expr::new(Node::Variable(FOLD_ACC.into()))?,
                        Expr::new(Node::Variable(FOLD_VALUE.into()))?,
                    ],
                ))?;
                Node::ExpandedFold(
                    name.clone(),
                    Box::new(initial),
                    Box::new(self.walk(&call, None)?),
                )
            }
            Node::ExpandedFold(name, initial, body) => {
                if parameters.is_some() {
                    return Err("pure source functions cannot capture numeric history".into());
                }
                Node::ExpandedFold(
                    name.clone(),
                    Box::new(self.walk(initial, None)?),
                    Box::new(self.walk(body, None)?),
                )
            }
            Node::Latest(name) => {
                if parameters.is_some() {
                    return Err("pure source functions cannot capture numeric history".into());
                }
                Node::Latest(name.clone())
            }
            Node::Qualified(value, evidence, caveats) => {
                if parameters.is_some() {
                    return Err(
                        "pure source functions cannot capture evidence in qualified values".into(),
                    );
                }
                Node::Qualified(
                    Box::new(self.walk(value, parameters)?),
                    evidence.clone(),
                    caveats.clone(),
                )
            }
            Node::Unary(operator, child) => {
                Node::Unary(*operator, Box::new(self.walk(child, parameters)?))
            }
            Node::Binary(operator, left, right) => Node::Binary(
                *operator,
                Box::new(self.walk(left, parameters)?),
                Box::new(self.walk(right, parameters)?),
            ),
            Node::Require(condition, value) => Node::Require(
                Box::new(self.walk(condition, parameters)?),
                Box::new(self.walk(value, parameters)?),
            ),
            Node::If(condition, yes, no) => Node::If(
                Box::new(self.walk(condition, parameters)?),
                Box::new(self.walk(yes, parameters)?),
                Box::new(self.walk(no, parameters)?),
            ),
            Node::Function(function, arguments) => Node::Function(
                *function,
                arguments
                    .iter()
                    .map(|argument| self.walk(argument, parameters))
                    .collect::<Result<_, _>>()?,
            ),
            Node::ExpandedCall(arguments, body) => Node::ExpandedCall(
                arguments
                    .iter()
                    .map(|argument| self.walk(argument, parameters))
                    .collect::<Result<_, _>>()?,
                Box::new(self.walk(body, parameters)?),
            ),
            Node::UserCall(name, arguments) => {
                let functions = self.functions;
                let definition = functions
                    .get(name)
                    .ok_or_else(|| format!("unknown source function {name}"))?;
                check_arity(definition, arguments.len())?;
                if self.active.contains(name) {
                    return Err(format!("recursive source function cycle involving {name}"));
                }
                if self.active.len() >= MAX_NESTING {
                    return Err(format!(
                        "source function expansion exceeds nesting limit {MAX_NESTING}"
                    ));
                }
                // Expand call arguments before entering the callee: f(f(x))
                // is ordinary composition, not recursive function definition.
                let arguments = arguments
                    .iter()
                    .map(|argument| self.walk(argument, parameters))
                    .collect::<Result<Vec<_>, _>>()?;
                let bindings = definition
                    .parameters
                    .iter()
                    .cloned()
                    .zip(arguments.iter().cloned())
                    .collect();
                self.active.push(name.clone());
                let body = self.walk(&definition.body, Some(&bindings));
                self.active.pop();
                Node::ExpandedCall(arguments, Box::new(body?))
            }
        };
        self.build(node)
    }
}

/// Expand pure source calls into a bounded, closed expression before the host
/// supplies state types. Call `validate_functions` once for the registry first;
/// expansion also checks used calls for arity, captures, cycles and limits.
pub fn expand(
    expression: &Expr,
    functions: &BTreeMap<String, FunctionDef>,
) -> Result<Expr, String> {
    expand_with(expression, functions, &BTreeMap::new())
}

/// `expand`, also inlining named `define` expressions wherever they are read.
pub fn expand_with(
    expression: &Expr,
    functions: &BTreeMap<String, FunctionDef>,
    defines: &BTreeMap<String, Expr>,
) -> Result<Expr, String> {
    if functions.len() > MAX_FUNCTIONS {
        return Err(format!("source exceeds function limit {MAX_FUNCTIONS}"));
    }
    Expander {
        functions,
        defines,
        active: Vec::new(),
        remaining: MAX_EXPANDED_NODES,
    }
    .walk(expression, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    // Tests exercising the standard library expand its actual Caveat source.
    // The public strict parser intentionally recognizes primitives only.
    fn parse(input: &str) -> Result<Expr, String> {
        expand(
            &parse_unresolved(input)?,
            &crate::reactive::prelude_functions()?,
        )
    }

    fn eval(input: &str) -> Result<Value, String> {
        parse(input)?.evaluate(&|_| None, &|_, _| Err("unexpected predicate".into()))
    }

    fn definitions(items: &[(&str, &[&str], &str)]) -> BTreeMap<String, FunctionDef> {
        items
            .iter()
            .map(|(name, parameters, body)| {
                (
                    (*name).into(),
                    FunctionDef {
                        name: (*name).into(),
                        parameters: parameters
                            .iter()
                            .map(|parameter| (*parameter).into())
                            .collect(),
                        body: parse_unresolved(body).unwrap(),
                    },
                )
            })
            .collect()
    }

    fn provenance(evidence: &[&str], caveats: &[&str]) -> Provenance {
        Provenance::from_names(
            evidence.iter().map(|name| (*name).to_string()),
            caveats.iter().map(|name| (*name).to_string()),
        )
        .unwrap()
    }

    #[test]
    fn history_reads_use_explicit_typed_hooks_and_exact_identifier_arity() {
        let expression = parse("has_sample(readings) and latest(course) > 0").unwrap();
        let calls = RefCell::new(Vec::new());
        assert_eq!(
            expression.validate(&HashSet::new(), &|kind, name| {
                calls
                    .borrow_mut()
                    .push((kind.to_string(), name.to_string()));
                Ok(())
            }),
            Ok(ValueType::Bool)
        );
        assert_eq!(
            *calls.borrow(),
            vec![
                ("has_sample".into(), "readings".into()),
                ("numeric_history".into(), "course".into()),
            ]
        );
        assert!(expression
            .validate(&HashSet::new(), &|kind, _| {
                if kind == "has_sample" {
                    Err("expected reading stream".into())
                } else {
                    Ok(())
                }
            })
            .is_err());
        assert!(parse("latest(readings)")
            .unwrap()
            .validate(&HashSet::new(), &|_, _| Err(
                "unknown numeric history".into()
            ))
            .is_err());
        assert!(parse("readings")
            .unwrap()
            .validate(&HashSet::new(), &|_, _| Ok(()))
            .is_err());
        for source in [
            "latest()",
            "latest(1)",
            "latest(a, b)",
            "latest(a.x)",
            "latest(a + b)",
            "has_sample()",
            "has_sample(a, b)",
            "has_sample(a.x)",
        ] {
            assert!(super::parse(source).is_err(), "{source}");
        }
    }

    #[test]
    fn latest_reads_merge_immutable_occurrence_dependencies_and_never_read_numeric_aliases() {
        let expression = parse("latest(readings) + latest(course)").unwrap();
        let current = Cell::new(1);
        let read = |name: &str| {
            let occurrence = format!("{name}#{}", current.get());
            Tracked::new(
                2.0,
                Provenance::from_names([occurrence], ["uncertainty".into()])?,
            )
        };
        let first = expression
            .evaluate_tracked_with_history(
                &|_| Err("latest must not use numeric variable callbacks".into()),
                &|_, _| Ok(Tracked::plain(false)),
                &|_, _| Ok(Provenance::default()),
                &read,
            )
            .unwrap();
        current.set(2);
        let second = expression
            .evaluate_tracked_with_history(
                &|_| Err("latest must not use numeric variable callbacks".into()),
                &|_, _| Ok(Tracked::plain(false)),
                &|_, _| Ok(Provenance::default()),
                &read,
            )
            .unwrap();
        assert_eq!(first.value, Value::Number(4.0));
        assert_eq!(
            first.provenance,
            provenance(&["course#1", "readings#1"], &["uncertainty"])
        );
        assert_eq!(
            second.provenance,
            provenance(&["course#2", "readings#2"], &["uncertainty"])
        );
        let qualified = parse("qualified(latest(readings), template)")
            .unwrap()
            .evaluate_tracked_with_history(
                &|_| Ok(None),
                &|_, _| Ok(Tracked::plain(false)),
                &|evidence, _| Ok(provenance(&[evidence], &["explicit"])),
                &read,
            )
            .unwrap();
        assert_eq!(
            qualified.provenance,
            provenance(&["readings#2", "template"], &["uncertainty", "explicit"])
        );
    }

    #[test]
    fn guarded_absent_history_reads_short_circuit_without_fabricating_samples() {
        let history_reads = Cell::new(0);
        let expression = parse("if(has_sample(readings), latest(readings), 0)").unwrap();
        let result = expression
            .evaluate_tracked_with_history(
                &|_| Ok(None),
                &|kind, name| {
                    assert_eq!((kind, name), ("has_sample", "readings"));
                    Ok(Tracked::plain(false))
                },
                &|_, _| Ok(Provenance::default()),
                &|_| {
                    history_reads.set(history_reads.get() + 1);
                    Err("history has no reached occurrence".into())
                },
            )
            .unwrap();
        assert_eq!(result.value, Value::Number(0.0));
        assert!(result.provenance.is_empty());
        assert_eq!(history_reads.get(), 0);
        let unguarded = parse("latest(readings)").unwrap();
        assert_eq!(
            unguarded.evaluate_tracked_with_history(
                &|_| Ok(None),
                &|_, _| Ok(Tracked::plain(false)),
                &|_, _| Ok(Provenance::default()),
                &|_| Err("history has no reached occurrence".into()),
            ),
            Err("history has no reached occurrence".into())
        );
        assert!(unguarded
            .evaluate_tracked(
                &|_| Ok(None),
                &|_, _| Ok(Tracked::plain(false)),
                &|_, _| Ok(Provenance::default())
            )
            .unwrap_err()
            .contains("history-aware evaluation host"));
        assert!(unguarded
            .evaluate(&|_| None, &|_, _| Ok(false))
            .unwrap_err()
            .contains("history-aware evaluation host"));
        assert_eq!(
            eval("if(false, latest(readings), 1)"),
            Ok(Value::Number(1.0))
        );
    }

    #[test]
    fn history_values_are_eager_function_arguments_but_cannot_be_captured() {
        let functions = definitions(&[("ignore", &["x"], "0")]);
        let expression = expand(
            &parse_unresolved("ignore(latest(readings))").unwrap(),
            &functions,
        )
        .unwrap();
        let result = expression
            .evaluate_tracked_with_history(
                &|_| Ok(None),
                &|_, _| Ok(Tracked::plain(false)),
                &|_, _| Ok(Provenance::default()),
                &|_| Tracked::new(3.0, provenance(&["readings#2"], &["risk"])),
            )
            .unwrap();
        assert_eq!(result.value, Value::Number(0.0));
        assert_eq!(result.provenance, provenance(&["readings#2"], &["risk"]));
        assert!(expression
            .evaluate_tracked_with_history(
                &|_| Ok(None),
                &|_, _| Ok(Tracked::plain(false)),
                &|_, _| Ok(Provenance::default()),
                &|_| Err("history has no reached occurrence".into()),
            )
            .is_err());
        for body in ["latest(readings) + x", "has_sample(readings)"] {
            assert!(
                validate_functions(&definitions(&[("capture", &["x"], body)])).is_err(),
                "{body}"
            );
        }
        let capture = definitions(&[("capture", &["x"], "latest(readings)")]);
        assert!(expand(&parse_unresolved("capture(1)").unwrap(), &capture)
            .unwrap_err()
            .contains("cannot capture numeric history"));
        for name in ["latest", "has_sample"] {
            assert!(validate_functions(&definitions(&[(name, &["x"], "x")]))
                .unwrap_err()
                .contains("shadows"));
        }
    }

    #[test]
    fn history_callback_values_obey_finite_and_provenance_bounds() {
        let expression = parse("latest(readings)").unwrap();
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(expression
                .evaluate_tracked_with_history(
                    &|_| Ok(None),
                    &|_, _| Ok(Tracked::plain(false)),
                    &|_, _| Ok(Provenance::default()),
                    &|_| Ok(Tracked::plain(value)),
                )
                .unwrap_err()
                .contains("non-finite"));
        }
        let invalid = Provenance {
            evidence: (0..=MAX_PROVENANCE_IDENTIFIERS)
                .map(|index| format!("e{index}"))
                .collect(),
            caveats: BTreeSet::new(),
        };
        assert!(expression
            .evaluate_tracked_with_history(
                &|_| Ok(None),
                &|_, _| Ok(Tracked::plain(false)),
                &|_, _| Ok(Provenance::default()),
                &|_| Ok(Tracked {
                    value: 1.0,
                    provenance: invalid.clone()
                }),
            )
            .unwrap_err()
            .contains("identifiers"));
    }

    #[test]
    fn text_literals_concatenation_and_numeric_conversion_are_typed() {
        assert_eq!(
            eval(r#""a  b" + "\n\t\"\\é""#),
            Ok(Value::Text("a  b\n\t\"\\é".into()))
        );
        assert_eq!(eval(r#""same" == "same""#), Ok(Value::Bool(true)));
        assert_eq!(eval(r#""left" != "right""#), Ok(Value::Bool(true)));
        assert_eq!(eval("text(-0)"), Ok(Value::Text("0".into())));
        assert_eq!(eval("text(1.25)"), Ok(Value::Text("1.25".into())));
        assert_eq!(eval("text(1e12)"), Ok(Value::Text("1000000000000".into())));
        assert_eq!(eval("floor(-1.2)"), Ok(Value::Number(-2.0)));
        assert_eq!(eval("ceil(-1.2)"), Ok(Value::Number(-1.0)));
        assert_eq!(eval("round(2.5)"), Ok(Value::Number(3.0)));
        assert_eq!(eval("round(-2.5)"), Ok(Value::Number(-3.0)));
        for source in [
            r#""1" + 2"#,
            r#"1 + "2""#,
            r#""1" - "2""#,
            r#""a" < "b""#,
            "text(true)",
            r#"floor("1")"#,
            r#""a" == 1"#,
        ] {
            let expression = parse(source).unwrap();
            assert!(
                expression
                    .validate(&HashSet::new(), &|_, _| Ok(()))
                    .is_err(),
                "{source}"
            );
            assert!(
                expression.evaluate(&|_| None, &|_, _| Ok(false)).is_err(),
                "{source}"
            );
        }
        for source in [r#""unterminated"#, r#""bad\q""#, r#""trailing\"#] {
            assert!(super::parse(source).is_err(), "{source}");
        }
    }

    #[test]
    fn if_and_require_are_lazy_with_static_branch_types() {
        assert_eq!(eval("if(true, 7, 1 / 0)"), Ok(Value::Number(7.0)));
        assert_eq!(eval("if(false, 1 / 0, 9)"), Ok(Value::Number(9.0)));
        assert_eq!(
            eval(r#"if(false, "first", "second")"#),
            Ok(Value::Text("second".into()))
        );
        assert_eq!(eval("if(true, false, true)"), Ok(Value::Bool(false)));
        assert_eq!(
            eval(r#"require(true, "ready")"#),
            Ok(Value::Text("ready".into()))
        );
        assert_eq!(eval("require(true, false)"), Ok(Value::Bool(false)));
        let calls = Cell::new(0);
        let result = parse("require(false, missing)").unwrap().evaluate(
            &|_| {
                calls.set(calls.get() + 1);
                None
            },
            &|_, _| Ok(false),
        );
        assert!(result.unwrap_err().contains("requirement failed"));
        assert_eq!(calls.get(), 0);
        for source in ["if(1, 2, 3)", r#"if(true, 1, "two")"#, "require(1, 2)"] {
            assert!(
                parse(source)
                    .unwrap()
                    .validate(&HashSet::new(), &|_, _| Ok(()))
                    .is_err(),
                "{source}"
            );
        }
        for source in [
            "if(true, 1)",
            "if(true, 1, 2, 3)",
            "require(true)",
            "require(true, 1, 2)",
        ] {
            assert!(super::parse(source).is_err(), "{source}");
        }
    }

    #[test]
    fn lazy_text_results_keep_condition_and_selected_branch_provenance() {
        for source in [
            "if(qualified(flag, condition, caution) > 0, text(qualified(value, selected, risk)), text(qualified(missing, skipped)))",
            "require(qualified(flag, condition, caution) > 0, text(qualified(value, selected, risk)))",
        ] {
            let result = parse(source).unwrap().evaluate_tracked(&|name| match name {
                "flag" => Ok(Some(Tracked::plain(1.0))),
                "value" => Ok(Some(Tracked::plain(7.0))),
                _ => Err("unselected branch was evaluated".into()),
            }, &|_, _| Ok(Tracked::plain(false)), &|evidence, caveats| {
                Provenance::from_names([evidence.to_string()], caveats.iter().cloned())
            }).unwrap();
            assert_eq!(result.value, Value::Text("7".into()));
            assert_eq!(result.provenance, provenance(&["condition", "selected"], &["caution", "risk"]));
        }
        let result = parse(r#""Speed " + text(qualified(2, reading, risk))"#)
            .unwrap()
            .evaluate_tracked(
                &|_| Ok(None),
                &|_, _| Ok(Tracked::plain(false)),
                &|evidence, caveats| {
                    Provenance::from_names([evidence.to_string()], caveats.iter().cloned())
                },
            )
            .unwrap();
        assert_eq!(result.value, Value::Text("Speed 2".into()));
        assert_eq!(result.provenance, provenance(&["reading"], &["risk"]));
    }

    #[test]
    fn source_functions_infer_forward_boolean_and_text_results() {
        let functions = definitions(&[
            (
                "report",
                &["x"],
                r#"if(positive(x), "positive", "nonpositive") + ": " + text(x)"#,
            ),
            ("positive", &["x"], "x > 0"),
            ("forward", &["x"], "report(x)"),
            ("constant_text", &["ignored"], r#""constant""#),
        ]);
        validate_functions(&functions).unwrap();
        let expression = expand(&parse_unresolved("forward(3)").unwrap(), &functions).unwrap();
        assert_eq!(
            expression.validate(&HashSet::new(), &|_, _| Ok(())),
            Ok(ValueType::Text)
        );
        assert_eq!(
            expression.evaluate(&|_| None, &|_, _| Ok(false)),
            Ok(Value::Text("positive: 3".into()))
        );
        let expression = expand(&parse_unresolved("positive(3)").unwrap(), &functions).unwrap();
        assert_eq!(
            expression.validate(&HashSet::new(), &|_, _| Ok(())),
            Ok(ValueType::Bool)
        );
        assert_eq!(
            expression.evaluate(&|_| None, &|_, _| Ok(false)),
            Ok(Value::Bool(true))
        );
        let ignored = expand(
            &parse_unresolved("constant_text(qualified(3, reading, risk))").unwrap(),
            &functions,
        )
        .unwrap()
        .evaluate_tracked(
            &|_| Ok(None),
            &|_, _| Ok(Tracked::plain(false)),
            &|evidence, caveats| {
                Provenance::from_names([evidence.to_string()], caveats.iter().cloned())
            },
        )
        .unwrap();
        assert_eq!(ignored.value, Value::Text("constant".into()));
        assert_eq!(ignored.provenance, provenance(&["reading"], &["risk"]));
        for source in [
            r#"forward("three")"#,
            "forward(positive(3))",
            "forward(3) + 1",
        ] {
            assert!(
                expand(&parse_unresolved(source).unwrap(), &functions)
                    .unwrap()
                    .validate(&HashSet::new(), &|_, _| Ok(()))
                    .is_err(),
                "{source}"
            );
        }
        let invalid = definitions(&[("mixed", &["x"], r#"if(x > 0, "yes", 0)"#)]);
        assert!(validate_functions(&invalid).is_err());
    }

    #[test]
    fn source_prelude_owns_standard_math_and_formatting_policy() {
        let mut functions = crate::reactive::prelude_functions().unwrap();
        validate_functions(&functions).unwrap();
        for name in [
            "abs",
            "min",
            "max",
            "clamp",
            "number_text",
            "percent_text",
            "time_text",
        ] {
            assert!(
                Function::named(name).is_none(),
                "{name} must be source-defined"
            );
        }
        assert!(super::parse("max(1, 2)").is_err());
        assert_eq!(eval("number_text(1.6)"), Ok(Value::Text("2".into())));
        assert_eq!(eval("percent_text(0.875)"), Ok(Value::Text("88%".into())));
        assert_eq!(eval("time_text(61.2)"), Ok(Value::Text("1:02".into())));
        assert_eq!(eval("time_text(-1)"), Ok(Value::Text("0:00".into())));
        assert_eq!(
            eval("time_text(floor(61.9))"),
            Ok(Value::Text("1:01".into()))
        );
        assert!(eval("clamp(1, 2, 0)")
            .unwrap_err()
            .contains("requirement failed"));
        assert!(eval("min(1, sqrt(-1))").is_err());
        assert_eq!(eval("max(2, 9)"), Ok(Value::Number(9.0)));
        // Editing an ordinary source function changes the result without any
        // primitive implementation switch or special session override.
        functions.get_mut("max").unwrap().body = parse_unresolved("left").unwrap();
        validate_functions(&functions).unwrap();
        let modified = expand(&parse_unresolved("max(2, 9)").unwrap(), &functions).unwrap();
        assert_eq!(
            modified.evaluate(&|_| None, &|_, _| Ok(false)),
            Ok(Value::Number(2.0))
        );
    }

    #[test]
    fn text_result_limits_reject_overflow_without_truncation() {
        let chunk = format!("\"{}\"", "é".repeat(MAX_TEXT_BYTES / 4));
        let functions = definitions(&[
            ("chunk", &[], &chunk),
            ("pair", &[], "chunk() + chunk()"),
            ("too_large", &[], r#"pair() + "x""#),
        ]);
        validate_functions(&functions).unwrap();
        let expression = expand(&parse_unresolved("pair()").unwrap(), &functions).unwrap();
        let Value::Text(value) = expression.evaluate(&|_| None, &|_, _| Ok(false)).unwrap() else {
            panic!("expected source text");
        };
        assert_eq!(value.len(), MAX_TEXT_BYTES);
        let overflow = expand(&parse_unresolved("too_large()").unwrap(), &functions).unwrap();
        assert!(overflow
            .evaluate(&|_| None, &|_, _| Ok(false))
            .unwrap_err()
            .contains("text exceeds limit"));
        let oversized_literal = Expr::new(Node::Text("x".repeat(MAX_TEXT_BYTES + 1))).unwrap();
        assert!(oversized_literal
            .evaluate(&|_| None, &|_, _| Ok(false))
            .is_err());
    }

    #[test]
    fn tracked_arithmetic_comparisons_and_intrinsics_cannot_launder_dependencies() {
        let expected = provenance(&["reading"], &["uncertainty"]);
        for source in [
            "x * 0",
            "x - x",
            "-x",
            "+x",
            "min(0, x)",
            "max(x, 100)",
            "clamp(x, 0, 1)",
            "clamp(0, 0, x)",
            "abs(x)",
            "sqrt(x)",
            "sin(x)",
            "cos(x)",
            "atan2(x, 1)",
            "x == x",
            "x != 0",
            "x < 100",
            "x <= 2",
            "x > 1",
            "x >= 2",
            "not x > 10",
        ] {
            let expression = parse(source).unwrap();
            let old_value = expression
                .evaluate(&|_| Some(2.0), &|_, _| Ok(false))
                .unwrap();
            let tracked = expression
                .evaluate_tracked(
                    &|_| Ok(Some(Tracked::new(2.0, expected.clone())?)),
                    &|_, _| Ok(Tracked::plain(false)),
                    &|_, _| Err("unexpected qualification".into()),
                )
                .unwrap();
            assert_eq!(tracked.value, old_value, "{source}");
            assert_eq!(tracked.provenance, expected, "{source}");
        }
        let literal = parse("1 + 2")
            .unwrap()
            .evaluate_tracked(
                &|_| Err("unexpected read".into()),
                &|_, _| Err("unexpected predicate".into()),
                &|_, _| Err("unexpected qualification".into()),
            )
            .unwrap();
        assert!(literal.provenance.is_empty());
    }

    #[test]
    fn tracked_boolean_short_circuit_only_retains_evaluated_dependencies() {
        for (source, left, right, visits, names) in [
            (
                "observed(left) or examined(right)",
                true,
                false,
                1,
                vec!["left"],
            ),
            (
                "observed(left) and examined(right)",
                false,
                true,
                1,
                vec!["left"],
            ),
            (
                "observed(left) and examined(right)",
                true,
                false,
                2,
                vec!["left", "right"],
            ),
            (
                "observed(left) or examined(right)",
                false,
                true,
                2,
                vec!["left", "right"],
            ),
            ("not observed(left)", true, false, 1, vec!["left"]),
        ] {
            let calls = Cell::new(0);
            let tracked = parse(source)
                .unwrap()
                .evaluate_tracked(
                    &|_| Ok(None),
                    &|_, name| {
                        calls.set(calls.get() + 1);
                        Tracked::new(
                            if name == "left" { left } else { right },
                            provenance(&[name], &[]),
                        )
                    },
                    &|_, _| Err("unexpected qualification".into()),
                )
                .unwrap();
            assert_eq!(calls.get(), visits, "{source}");
            assert_eq!(tracked.provenance, provenance(&names, &[]), "{source}");
        }
        for source in [
            "true or qualified(1, missing) > 0",
            "false and qualified(1, missing) > 0",
        ] {
            let tracked = parse(source)
                .unwrap()
                .evaluate_tracked(
                    &|_| Ok(None),
                    &|_, _| Err("unexpected predicate".into()),
                    &|_, _| Err("evidence has not been observed".into()),
                )
                .unwrap();
            assert!(tracked.provenance.is_empty());
        }
    }

    #[test]
    fn qualified_constructor_checks_graph_types_and_unions_numeric_and_graph_traces() {
        let expression = parse("qualified(raw + 2, reading, uncertainty, omission)").unwrap();
        let validations = RefCell::new(Vec::new());
        assert_eq!(
            expression.validate(&["raw".into()].into_iter().collect(), &|kind, name| {
                validations
                    .borrow_mut()
                    .push((kind.to_string(), name.to_string()));
                Ok(())
            }),
            Ok(ValueType::Number)
        );
        assert_eq!(
            *validations.borrow(),
            vec![
                ("qualification_evidence".into(), "reading".into()),
                ("qualification_caveat".into(), "uncertainty".into()),
                ("qualification_caveat".into(), "omission".into()),
            ]
        );
        let tracked = expression
            .evaluate_tracked(
                &|_| {
                    Ok(Some(Tracked::new(
                        3.0,
                        provenance(&["earlier"], &["retained"]),
                    )?))
                },
                &|_, _| Err("constructor must use the dedicated callback".into()),
                &|evidence, caveats| {
                    assert_eq!(evidence, "reading");
                    assert_eq!(
                        caveats,
                        &["uncertainty".to_string(), "omission".to_string()]
                    );
                    Ok(provenance(
                        &[evidence],
                        &["uncertainty", "omission", "transitive"],
                    ))
                },
            )
            .unwrap();
        assert_eq!(tracked.value, Value::Number(5.0));
        assert_eq!(
            tracked.provenance,
            provenance(
                &["earlier", "reading"],
                &["retained", "uncertainty", "omission", "transitive"]
            )
        );
        assert!(expression
            .validate(&["raw".into()].into_iter().collect(), &|_, _| Err(
                "wrong graph kind".into()
            ))
            .is_err());
        assert!(expression
            .evaluate(&|_| Some(3.0), &|_, _| Ok(true))
            .unwrap_err()
            .contains("tracked evaluation host"));
    }

    #[test]
    fn qualified_constructor_is_numeric_explicit_and_fails_on_host_rejection() {
        for source in [
            "qualified()",
            "qualified(1)",
            "qualified(1, 2)",
            "qualified(1, reading,)",
            "qualified(1, reading.x)",
            "qualified(1, reading, risk + 1)",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
        let expression = parse("qualified(true, reading)").unwrap();
        assert!(expression
            .validate(&HashSet::new(), &|_, _| Ok(()))
            .is_err());
        let calls = Cell::new(0);
        assert!(expression
            .evaluate_tracked(&|_| Ok(None), &|_, _| Ok(Tracked::plain(false)), &|_, _| {
                calls.set(calls.get() + 1);
                Ok(Provenance::default())
            })
            .is_err());
        assert_eq!(calls.get(), 0);
        let expression = parse("qualified(1, unread)").unwrap();
        assert_eq!(
            expression.evaluate_tracked(
                &|_| Ok(None),
                &|_, _| Ok(Tracked::plain(false)),
                &|_, _| Err("evidence unread has not been observed".into()),
            ),
            Err("evidence unread has not been observed".into())
        );
        let nested = parse("qualified(qualified(1, first, a), second, b)")
            .unwrap()
            .evaluate_tracked(
                &|_| Ok(None),
                &|_, _| Ok(Tracked::plain(false)),
                &|evidence, caveats| {
                    Provenance::from_names([evidence.to_string()], caveats.iter().cloned())
                },
            )
            .unwrap();
        assert_eq!(
            nested.provenance,
            provenance(&["first", "second"], &["a", "b"])
        );
    }

    #[test]
    fn expanded_functions_preserve_ignored_qualified_arguments_without_global_capture() {
        let functions = definitions(&[("ignore", &["x"], "0"), ("cancel", &["x"], "x-x")]);
        validate_functions(&functions).unwrap();
        for source in [
            "ignore(qualified(3, reading, risk))",
            "cancel(qualified(3, reading, risk))",
            "ignore(cancel(qualified(3, reading, risk)))",
        ] {
            let expression = expand(&parse_unresolved(source).unwrap(), &functions).unwrap();
            let tracked = expression
                .evaluate_tracked(
                    &|_| Ok(None),
                    &|_, _| Ok(Tracked::plain(false)),
                    &|evidence, caveats| {
                        Provenance::from_names([evidence.to_string()], caveats.iter().cloned())
                    },
                )
                .unwrap();
            assert_eq!(tracked.value, Value::Number(0.0));
            assert_eq!(tracked.provenance, provenance(&["reading"], &["risk"]));
        }
        let capture = definitions(&[("capture", &["x"], "qualified(x, reading)")]);
        assert!(validate_functions(&capture).is_err());
        assert!(expand(&parse_unresolved("capture(1)").unwrap(), &capture).is_err());
        let shadow = definitions(&[("qualified", &["x"], "x")]);
        assert!(validate_functions(&shadow).unwrap_err().contains("shadows"));
    }

    #[test]
    fn provenance_is_sorted_deduplicated_and_bounded_with_atomic_merge() {
        let left = provenance(&["z", "a", "z"], &["risk"]);
        let right = provenance(&["b", "a"], &["risk", "omission"]);
        let joined = left.union(&right).unwrap();
        assert_eq!(joined, right.union(&left).unwrap());
        assert_eq!(
            serde_json::to_string(&joined).unwrap(),
            "{\"evidence\":[\"a\",\"b\",\"z\"],\"caveats\":[\"omission\",\"risk\"]}"
        );
        let mut full = Provenance::from_names(
            (0..MAX_PROVENANCE_IDENTIFIERS).map(|index| format!("e{index}")),
            [],
        )
        .unwrap();
        assert_eq!(full.union(&full).unwrap(), full);
        let before = full.clone();
        assert!(full
            .merge(&provenance(&[], &["additional"]))
            .unwrap_err()
            .contains("identifiers"));
        assert_eq!(full, before);
        assert!(Provenance::from_names(
            (0..=MAX_PROVENANCE_IDENTIFIERS).map(|index| format!("e{index}")),
            []
        )
        .is_err());
        let full_bytes =
            Provenance::from_names(["é".repeat(MAX_PROVENANCE_BYTES / 2)], []).unwrap();
        let mut oversized = full_bytes.clone();
        assert!(oversized
            .merge(&provenance(&[], &["x"]))
            .unwrap_err()
            .contains("name bytes"));
        assert_eq!(oversized, full_bytes);
        assert!(Provenance::from_names(["a".repeat(MAX_PROVENANCE_BYTES + 1)], []).is_err());
    }

    #[test]
    fn tracked_evaluation_rejects_oversized_host_metadata_and_combined_traces() {
        let half = MAX_PROVENANCE_IDENTIFIERS / 2;
        let expression = parse("left + right").unwrap();
        let result = expression.evaluate_tracked(
            &|name| {
                let (start, end) = if name == "left" {
                    (0, half)
                } else {
                    (half, MAX_PROVENANCE_IDENTIFIERS + 1)
                };
                Ok(Some(Tracked::new(
                    1.0,
                    Provenance::from_names((start..end).map(|index| format!("e{index}")), [])?,
                )?))
            },
            &|_, _| Ok(Tracked::plain(false)),
            &|_, _| Ok(Provenance::default()),
        );
        assert!(result.unwrap_err().contains("identifiers"));
        let invalid = Provenance {
            evidence: (0..=MAX_PROVENANCE_IDENTIFIERS)
                .map(|index| format!("e{index}"))
                .collect(),
            caveats: BTreeSet::new(),
        };
        assert!(parse("x")
            .unwrap()
            .evaluate_tracked(
                &|_| Ok(Some(Tracked {
                    value: 1.0,
                    provenance: invalid.clone()
                })),
                &|_, _| Ok(Tracked::plain(false)),
                &|_, _| Ok(Provenance::default()),
            )
            .is_err());
        assert!(parse("observed(e)")
            .unwrap()
            .evaluate_tracked(
                &|_| Ok(None),
                &|_, _| Ok(Tracked {
                    value: true,
                    provenance: invalid.clone()
                }),
                &|_, _| Ok(Provenance::default()),
            )
            .is_err());
        assert!(parse("qualified(1, e)")
            .unwrap()
            .evaluate_tracked(
                &|_| Ok(None),
                &|_, _| Ok(Tracked::plain(false)),
                &|_, _| Ok(invalid.clone()),
            )
            .is_err());
    }

    #[test]
    fn tracked_evaluation_keeps_arithmetic_and_callback_errors() {
        for source in ["x / 0", "x * 1e308", "sqrt(-x)", "clamp(x, 2, 1)"] {
            let expression = parse(source).unwrap();
            let expected = expression
                .evaluate(&|_| Some(2.0), &|_, _| Ok(false))
                .unwrap_err();
            let actual = expression
                .evaluate_tracked(
                    &|_| {
                        Ok(Some(Tracked::new(
                            2.0,
                            provenance(&["reading"], &["risk"]),
                        )?))
                    },
                    &|_, _| Ok(Tracked::plain(false)),
                    &|_, _| Ok(Provenance::default()),
                )
                .unwrap_err();
            assert_eq!(actual, expected, "{source}");
        }
        let expression = parse("x").unwrap();
        assert_eq!(
            expression.evaluate_tracked(
                &|_| Err("host lookup failed".into()),
                &|_, _| Ok(Tracked::plain(false)),
                &|_, _| Ok(Provenance::default())
            ),
            Err("host lookup failed".into())
        );
        for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            assert!(expression
                .evaluate_tracked(
                    &|_| Ok(Some(Tracked::plain(value))),
                    &|_, _| Ok(Tracked::plain(false)),
                    &|_, _| Ok(Provenance::default())
                )
                .is_err());
        }
    }

    #[test]
    fn atan2_uses_y_x_order_and_retains_finite_argument_checks() {
        assert_eq!(
            eval("atan2(1, 0)"),
            Ok(Value::Number(std::f64::consts::FRAC_PI_2))
        );
        assert_eq!(
            eval("atan2(0, -1)"),
            Ok(Value::Number(std::f64::consts::PI))
        );
        assert_eq!(
            eval("atan2(-1, 0)"),
            Ok(Value::Number(-std::f64::consts::FRAC_PI_2))
        );
        assert_eq!(
            eval("atan2(1e308, 1e308)"),
            Ok(Value::Number(std::f64::consts::FRAC_PI_4))
        );
        assert_eq!(eval("atan2(0, 0)"), Ok(Value::Number(0.0)));
        assert!(parse("atan2(1)").is_err());
        assert!(parse("atan2(1, 2, 3)").is_err());
        assert!(eval("atan2(1 / 0, 1)").is_err());
        assert!(parse("atan2(x, 1)")
            .unwrap()
            .evaluate(&|_| Some(f64::INFINITY), &|_, _| Ok(false))
            .is_err());
    }

    #[test]
    fn source_functions_expand_forward_calls_and_explicit_world_coordinates() {
        let functions = definitions(&[
            (
                "distance",
                &["ax", "az", "bx", "bz"],
                "sqrt(square(ax-bx) + square(az-bz))",
            ),
            ("square", &["x"], "x*x"),
        ]);
        validate_functions(&functions).unwrap();
        assert!(parse("distance(1, 2, 3, 4)").is_err());
        let expression =
            parse_unresolved("distance(ship_x, ship_z, reef_one.x, reef_one.z)").unwrap();
        assert!(expression
            .validate(&HashSet::new(), &|_, _| Ok(()))
            .is_err());
        assert!(expression
            .evaluate(&|_| Some(1.0), &|_, _| Ok(false))
            .is_err());
        let expression = expand(&expression, &functions).unwrap();
        let names = ["ship_x", "ship_z", "reef_one.x", "reef_one.z"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(
            expression.validate(&names, &|_, _| Ok(())),
            Ok(ValueType::Number)
        );
        assert_eq!(
            expression.evaluate(
                &|name| match name {
                    "ship_x" => Some(7.0),
                    "ship_z" => Some(9.0),
                    "reef_one.x" => Some(4.0),
                    "reef_one.z" => Some(5.0),
                    _ => None,
                },
                &|_, _| Ok(false)
            ),
            Ok(Value::Number(5.0))
        );
    }

    #[test]
    fn function_substitution_preserves_caller_scope_and_allows_composition() {
        let functions = definitions(&[
            ("subtract", &["x", "y"], "x-y"),
            ("shift", &["x"], "x+1"),
            ("quarter_turn", &[], "atan2(1, 0)"),
        ]);
        validate_functions(&functions).unwrap();
        let expression = expand(&parse_unresolved("subtract(y, x)").unwrap(), &functions).unwrap();
        assert_eq!(
            expression.evaluate(
                &|name| match name {
                    "x" => Some(2.0),
                    "y" => Some(7.0),
                    _ => None,
                },
                &|_, _| Ok(false)
            ),
            Ok(Value::Number(5.0))
        );
        let expression = expand(&parse_unresolved("shift(shift(2))").unwrap(), &functions).unwrap();
        assert_eq!(
            expression.evaluate(&|_| None, &|_, _| Ok(false)),
            Ok(Value::Number(4.0))
        );
        let expression = expand(&parse_unresolved("quarter_turn()").unwrap(), &functions).unwrap();
        assert_eq!(
            expression.evaluate(&|_| None, &|_, _| Ok(false)),
            Ok(Value::Number(std::f64::consts::FRAC_PI_2))
        );
    }

    #[test]
    fn functions_reject_duplicate_parameters_shadowing_bad_names_and_arity() {
        let duplicate = definitions(&[("duplicate", &["x", "x"], "x")]);
        assert!(validate_functions(&duplicate)
            .unwrap_err()
            .contains("duplicate parameter"));
        for name in [
            "sin",
            "cos",
            "sqrt",
            "atan2",
            "floor",
            "ceil",
            "round",
            "text",
            "if",
            "require",
            "observed",
            "examined",
            "committed",
            "reopened",
        ] {
            let functions = definitions(&[(name, &[], "1")]);
            assert!(
                validate_functions(&functions)
                    .unwrap_err()
                    .contains("shadows"),
                "{name}"
            );
        }
        for name in ["a.b", "1name", "and", "true", ""] {
            assert!(validate_functions(&definitions(&[(name, &[], "1")])).is_err());
            assert!(validate_functions(&definitions(&[("valid", &[name], "1")])).is_err());
        }
        let mut mismatch = definitions(&[("valid", &[], "1")]);
        mismatch.get_mut("valid").unwrap().name = "different".into();
        assert!(validate_functions(&mismatch).is_err());
        let functions = definitions(&[("square", &["x"], "x*x")]);
        for source in ["square()", "square(1, 2)", "unknown(1)"] {
            assert!(
                expand(&parse_unresolved(source).unwrap(), &functions).is_err(),
                "{source}"
            );
        }
        for body in ["square()", "square(1, 2)", "unknown(1)"] {
            let functions = definitions(&[("square", &["x"], "x*x"), ("unused", &[], body)]);
            assert!(validate_functions(&functions).is_err(), "{body}");
        }
        assert!(parse_unresolved("source.function(1)").is_err());
    }

    #[test]
    fn pure_functions_cannot_capture_state_coordinates_or_graph() {
        for body in [
            "heat + x",
            "reef_one.x + x",
            "observed(signal)",
            "examined(risk)",
            "max(x, false)",
        ] {
            let functions = definitions(&[("impure", &["x"], body)]);
            assert!(validate_functions(&functions).is_err(), "{body}");
        }
        let functions = definitions(&[("ignore", &["x"], "1"), ("capture", &[], "ignore(heat)")]);
        assert!(validate_functions(&functions).is_err());
        let functions = definitions(&[("capture", &["x"], "x + heat")]);
        assert!(expand(&parse_unresolved("capture(1)").unwrap(), &functions)
            .unwrap_err()
            .contains("cannot capture"));
    }

    #[test]
    fn unused_function_arguments_are_still_numeric_and_eagerly_checked() {
        let functions = definitions(&[("constant", &["ignored"], "2")]);
        validate_functions(&functions).unwrap();
        let boolean = expand(&parse_unresolved("constant(true)").unwrap(), &functions).unwrap();
        assert!(boolean.validate(&HashSet::new(), &|_, _| Ok(())).is_err());
        assert!(boolean.evaluate(&|_| None, &|_, _| Ok(false)).is_err());
        let unknown = expand(&parse_unresolved("constant(missing)").unwrap(), &functions).unwrap();
        assert!(unknown.validate(&HashSet::new(), &|_, _| Ok(())).is_err());
        let division = expand(&parse_unresolved("constant(1 / 0)").unwrap(), &functions).unwrap();
        assert!(division
            .evaluate(&|_| None, &|_, _| Ok(false))
            .unwrap_err()
            .contains("division by zero"));
    }

    #[test]
    fn unused_direct_and_indirect_recursive_functions_are_rejected() {
        for functions in [
            definitions(&[("valid", &[], "1"), ("recursive", &["x"], "recursive(x)")]),
            definitions(&[
                ("valid", &[], "1"),
                ("first", &["x"], "second(x)"),
                ("second", &["x"], "first(x)"),
            ]),
        ] {
            assert!(validate_functions(&functions)
                .unwrap_err()
                .contains("cycle"));
        }
        let functions = definitions(&[("recursive", &["x"], "recursive(x)")]);
        assert!(
            expand(&parse_unresolved("recursive(1)").unwrap(), &functions)
                .unwrap_err()
                .contains("cycle")
        );
    }

    #[test]
    fn function_count_expanded_nodes_and_depth_are_bounded() {
        let mut functions = BTreeMap::new();
        for index in 0..MAX_FUNCTIONS {
            let name = format!("constant_{index}");
            functions.insert(
                name.clone(),
                FunctionDef {
                    name,
                    parameters: Vec::new(),
                    body: parse("1").unwrap(),
                },
            );
        }
        validate_functions(&functions).unwrap();
        functions.insert(
            "excess".into(),
            FunctionDef {
                name: "excess".into(),
                parameters: Vec::new(),
                body: parse("1").unwrap(),
            },
        );
        assert!(validate_functions(&functions)
            .unwrap_err()
            .contains("function limit"));
        assert!(expand(&parse("1").unwrap(), &functions)
            .unwrap_err()
            .contains("function limit"));

        let mut expanding = definitions(&[("f0", &["x"], "x+x")]);
        for index in 1..=12 {
            let name = format!("f{index}");
            expanding.insert(
                name.clone(),
                FunctionDef {
                    name,
                    parameters: vec!["x".into()],
                    body: parse_unresolved(&format!("f{}(x)+f{}(x)", index - 1, index - 1))
                        .unwrap(),
                },
            );
        }
        assert!(validate_functions(&expanding)
            .unwrap_err()
            .contains("expansion limit"));
        assert!(expand(&parse_unresolved("f12(1)").unwrap(), &expanding)
            .unwrap_err()
            .contains("expansion limit"));

        let mut deep = definitions(&[("f0", &["x"], "x")]);
        for index in 1..=MAX_NESTING {
            let name = format!("f{index}");
            deep.insert(
                name.clone(),
                FunctionDef {
                    name,
                    parameters: vec!["x".into()],
                    body: parse_unresolved(&format!("f{}(x)", index - 1)).unwrap(),
                },
            );
        }
        assert!(validate_functions(&deep)
            .unwrap_err()
            .contains("nesting limit"));
    }

    #[test]
    fn arithmetic_has_standard_precedence_and_left_associativity() {
        assert_eq!(eval("1 + 2 * 3 - 8 / 4"), Ok(Value::Number(5.0)));
        assert_eq!(eval("20 / 2 / 5"), Ok(Value::Number(2.0)));
        assert_eq!(eval("10 - 3 - 2"), Ok(Value::Number(5.0)));
        assert_eq!(eval("-(1 + 2) * +4"), Ok(Value::Number(-12.0)));
        assert_eq!(eval(".5 + 1. + 2.5e-1 + 1E+1"), Ok(Value::Number(11.75)));
        assert_eq!(parse("1.00"), parse("1e0"));
    }

    #[test]
    fn boolean_comparisons_and_not_have_defined_precedence() {
        assert_eq!(eval("1 + 2 == 3 and 4 != 5"), Ok(Value::Bool(true)));
        assert_eq!(
            eval("1 < 2 and 2 <= 2 and 3 > 2 and 3 >= 3"),
            Ok(Value::Bool(true))
        );
        assert_eq!(eval("true or false and false"), Ok(Value::Bool(true)));
        assert_eq!(eval("not 3 < 2 and true"), Ok(Value::Bool(true)));
        assert_eq!(eval("not true == false"), Ok(Value::Bool(true)));
        assert_eq!(eval("(1 < 2) == true"), Ok(Value::Bool(true)));
    }

    #[test]
    fn functions_support_source_owned_motion_and_collision_math() {
        assert_eq!(eval("max(abs(-4), min(10, 2))"), Ok(Value::Number(4.0)));
        assert_eq!(eval("clamp(20, 0, 7)"), Ok(Value::Number(7.0)));
        assert_eq!(eval("clamp(-2, 0, 7)"), Ok(Value::Number(0.0)));
        assert_eq!(eval("clamp(2, 2, 2)"), Ok(Value::Number(2.0)));
        assert_eq!(eval("sin(0) + cos(0) + sqrt(9)"), Ok(Value::Number(4.0)));
        let expression =
            parse("sqrt((ship_x - reef_one.x) * (ship_x - reef_one.x) + reef_one.z * reef_one.z)")
                .unwrap();
        let identifiers = ["ship_x", "reef_one.x", "reef_one.z"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(
            expression.validate(&identifiers, &|_, _| Ok(())),
            Ok(ValueType::Number)
        );
        assert_eq!(
            expression.evaluate(
                &|name| match name {
                    "ship_x" => Some(7.0),
                    "reef_one.x" => Some(4.0),
                    "reef_one.z" => Some(4.0),
                    _ => None,
                },
                &|_, _| Ok(false)
            ),
            Ok(Value::Number(5.0))
        );
    }

    #[test]
    fn graph_queries_delegate_target_validation_and_evaluation_to_host() {
        for name in ["observed", "examined", "committed", "reopened"] {
            let expression = parse(&format!("{name}(signal_known)")).unwrap();
            assert_eq!(
                expression.validate(&HashSet::new(), &|kind, target| {
                    assert_eq!(kind, name);
                    assert_eq!(target, "signal_known");
                    Ok(())
                }),
                Ok(ValueType::Bool)
            );
            assert_eq!(
                expression.evaluate(&|_| None, &|kind, target| {
                    assert_eq!(kind, name);
                    assert_eq!(target, "signal_known");
                    Ok(true)
                }),
                Ok(Value::Bool(true))
            );
            assert_eq!(
                expression.validate(&HashSet::new(), &|_, _| Err("wrong node kind".into())),
                Err("wrong node kind".into())
            );
            assert_eq!(
                expression.evaluate(&|_| None, &|_, _| Err("missing graph node".into())),
                Err("missing graph node".into())
            );
        }
    }

    #[test]
    fn static_validation_rejects_unknown_names_and_type_mismatches() {
        let identifiers = ["heat".to_string()].into_iter().collect();
        assert_eq!(
            parse("heat + 2")
                .unwrap()
                .validate(&identifiers, &|_, _| Ok(())),
            Ok(ValueType::Number)
        );
        for source in [
            "unknown + 2",
            "true + 1",
            "not heat",
            "heat and true",
            "1 == true",
            "max(true, 1)",
            "true < false",
        ] {
            assert!(
                parse(source)
                    .unwrap()
                    .validate(&identifiers, &|_, _| Ok(()))
                    .is_err(),
                "{source}"
            );
        }
        assert!(parse("true or observed(invalid)")
            .unwrap()
            .validate(&identifiers, &|_, _| Err("unknown evidence".into()))
            .is_err());
    }

    #[test]
    fn runtime_short_circuit_does_not_read_unneeded_graph_or_numeric_state() {
        let reads = Cell::new(0);
        let numbers = |_: &str| {
            reads.set(reads.get() + 1);
            None
        };
        let predicates = |_: &str, _: &str| {
            reads.set(reads.get() + 1);
            Err("not available".into())
        };
        assert_eq!(
            parse("false and observed(missing)")
                .unwrap()
                .evaluate(&numbers, &predicates),
            Ok(Value::Bool(false))
        );
        assert_eq!(
            parse("true or missing > 0")
                .unwrap()
                .evaluate(&numbers, &predicates),
            Ok(Value::Bool(true))
        );
        assert_eq!(reads.get(), 0);
        assert_eq!(eval("true or 1 / 0 > 0"), Ok(Value::Bool(true)));
        assert_eq!(eval("false and 1 / 0 > 0"), Ok(Value::Bool(false)));
        assert!(parse("true and observed(missing)")
            .unwrap()
            .evaluate(&numbers, &predicates)
            .is_err());
        assert_eq!(reads.get(), 1);
    }

    #[test]
    fn invalid_arithmetic_and_host_values_return_errors() {
        for source in [
            "1 / 0",
            "1 / -0",
            "1e308 * 1e308",
            "1e308 + 1e308",
            "clamp(2, 5, 1)",
            "sqrt(-1)",
            "true + 1",
            "1 == false",
            "1 and true",
            "false or 1",
        ] {
            assert!(eval(source).is_err(), "{source}");
        }
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(parse("heat")
                .unwrap()
                .evaluate(&|_| Some(value), &|_, _| Ok(false))
                .is_err());
        }
        assert!(eval("missing").unwrap_err().contains("missing"));
        assert!(parse("1e309").is_err());
    }

    #[test]
    fn malformed_source_and_wrong_function_arities_are_rejected() {
        for source in [
            "",
            "1 2",
            "1 +",
            "(1 + 2",
            "1)",
            "1e",
            "1e+",
            ".",
            "1..2",
            "x.",
            "x..y",
            "x.1",
            "é",
            "2 & 1",
            "1 = 2",
            "!true",
            "unknown(2)",
            "abs()",
            "min(1)",
            "max(1, 2, 3)",
            "clamp(1, 2)",
            "sin(1,)",
            "sqrt(1, 2)",
            "observed()",
            "examined(1)",
            "committed(a, b)",
            "reopened(a + b)",
            "observed(a.x)",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
        assert!(parse("observed_value + _Heat2 + reef_one.x").is_ok());
    }

    #[test]
    fn resource_limits_bound_parser_and_evaluator_depth() {
        let valid = format!("{}1{}", "(".repeat(MAX_NESTING), ")".repeat(MAX_NESTING));
        assert_eq!(eval(&valid), Ok(Value::Number(1.0)));
        let deep = format!("({valid})");
        assert!(parse(&deep).unwrap_err().contains("nesting limit"));
        assert!(parse(&"-".repeat(MAX_NESTING)).is_err());
        let unary = format!("{}1", "-".repeat(MAX_NESTING));
        assert!(parse(&unary).unwrap_err().contains("nesting limit"));
        let chain = vec!["1"; MAX_NESTING + 1].join(" + ");
        assert!(parse(&chain).unwrap_err().contains("nesting limit"));
        let excessive = vec!["1"; MAX_TOKENS].join(" + ");
        assert!(parse(&excessive).unwrap_err().contains("token limit"));
        assert!(parse(&"a".repeat(MAX_SOURCE_BYTES + 1))
            .unwrap_err()
            .contains("source limit"));
    }
}
