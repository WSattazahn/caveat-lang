//! Bounded, typed expressions for source-authored reactive game rules.
//!
//! Expressions can read numeric state and coordinates, and query the host's
//! epistemic graph. They cannot mutate state or invoke arbitrary host code.

use crate::presentation::Number;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashSet};

const MAX_TOKENS: usize = 1024;
const MAX_NESTING: usize = 64;
const MAX_SOURCE_BYTES: usize = 65_536;
const MAX_FUNCTIONS: usize = 128;
const MAX_EXPANDED_NODES: usize = 4096;
pub const MAX_PROVENANCE_IDENTIFIERS: usize = 1024;
pub const MAX_PROVENANCE_BYTES: usize = 65_536;

/// Dependencies of an evaluated value, not an assertion that evidence is true
/// or that a caveat is discharged. The host resolves and types graph names.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
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

/// A parsed expression with finite, canonical number literals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    node: Node,
    depth: usize,
}

/// A pure numeric source function. Definitions may refer to other functions
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
    Variable(String),
    Unary(Unary, Box<Expr>),
    Binary(Binary, Box<Expr>, Box<Expr>),
    Function(Function, Vec<Expr>),
    UserCall(String, Vec<Expr>),
    // Keep arguments even when the body ignores them: function arguments are
    // numeric and evaluated eagerly, so `constant(1 / 0)` must still fail.
    ExpandedCall(Vec<Expr>, Box<Expr>),
    Predicate(String, String),
    Qualified(Box<Expr>, String, Vec<String>),
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
    Abs,
    Min,
    Max,
    Clamp,
    Sin,
    Cos,
    Sqrt,
    Atan2,
}

impl Function {
    fn named(name: &str) -> Option<Self> {
        match name {
            "abs" => Some(Self::Abs),
            "min" => Some(Self::Min),
            "max" => Some(Self::Max),
            "clamp" => Some(Self::Clamp),
            "sin" => Some(Self::Sin),
            "cos" => Some(Self::Cos),
            "sqrt" => Some(Self::Sqrt),
            "atan2" => Some(Self::Atan2),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Abs => "abs",
            Self::Min => "min",
            Self::Max => "max",
            Self::Clamp => "clamp",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Sqrt => "sqrt",
            Self::Atan2 => "atan2",
        }
    }

    fn arity(self) -> usize {
        match self {
            Self::Abs | Self::Sin | Self::Cos | Self::Sqrt => 1,
            Self::Min | Self::Max | Self::Atan2 => 2,
            Self::Clamp => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Number,
    Bool,
}

impl ValueType {
    fn name(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::Bool => "boolean",
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
    fn number(self) -> Result<f64, String> {
        match self {
            Self::Number(value) => finite(value),
            Self::Bool(_) => Err("expression requires number, found boolean".into()),
        }
    }

    fn boolean(self) -> Result<bool, String> {
        match self {
            Self::Bool(value) => Ok(value),
            Self::Number(_) => Err("expression requires boolean, found number".into()),
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

impl Expr {
    fn new(node: Node) -> Result<Self, String> {
        let depth = match &node {
            Node::Unary(_, child) | Node::Qualified(child, _, _) => child.depth + 1,
            Node::Binary(_, left, right) => left.depth.max(right.depth) + 1,
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
        predicate: &impl Fn(&str, &str) -> Result<(), String>,
        user_call: &impl Fn(&str, usize) -> Result<(), String>,
    ) -> Result<ValueType, String> {
        match &self.node {
            Node::Number(_) => Ok(ValueType::Number),
            Node::Bool(_) => Ok(ValueType::Bool),
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
                    _ => {
                        left_type.require(ValueType::Number)?;
                        right_type.require(ValueType::Number)?;
                        match operator {
                            Binary::Add | Binary::Subtract | Binary::Multiply | Binary::Divide => {
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
                    Node::UserCall(name, _) => user_call(name, arguments.len())?,
                    Node::ExpandedCall(_, body) => body
                        .validate_calls(numeric, predicate, user_call)?
                        .require(ValueType::Number)?,
                    _ => {}
                }
                Ok(ValueType::Number)
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
        )?;
        Ok(Tracked {
            value,
            provenance: provenance.into_inner(),
        })
    }

    fn evaluate_values(
        &self,
        numbers: &impl Fn(&str) -> Result<Option<f64>, String>,
        predicate: &impl Fn(&str, &str) -> Result<bool, String>,
        qualify: &impl Fn(&str, &[String]) -> Result<(), String>,
    ) -> Result<Value, String> {
        match &self.node {
            Node::Number(value) => Ok(Value::Number(value.value())),
            Node::Bool(value) => Ok(Value::Bool(*value)),
            Node::Variable(name) => {
                let value =
                    numbers(name)?.ok_or_else(|| format!("unknown numeric identifier {name}"))?;
                Ok(Value::Number(finite(value)?))
            }
            Node::Predicate(name, target) => Ok(Value::Bool(predicate(name, target)?)),
            Node::Qualified(value, evidence, caveats) => {
                let value = value
                    .evaluate_values(numbers, predicate, qualify)?
                    .number()?;
                qualify(evidence, caveats)?;
                Ok(Value::Number(value))
            }
            Node::UserCall(name, _) => Err(format!(
                "source function {name} must be expanded before evaluation"
            )),
            Node::ExpandedCall(arguments, body) => {
                for argument in arguments {
                    argument
                        .evaluate_values(numbers, predicate, qualify)?
                        .number()?;
                }
                Ok(Value::Number(
                    body.evaluate_values(numbers, predicate, qualify)?
                        .number()?,
                ))
            }
            Node::Unary(operator, child) => {
                let value = child.evaluate_values(numbers, predicate, qualify)?;
                match operator {
                    Unary::Positive => Ok(Value::Number(value.number()?)),
                    Unary::Negative => Ok(Value::Number(-value.number()?)),
                    Unary::Not => Ok(Value::Bool(!value.boolean()?)),
                }
            }
            Node::Binary(operator, left, right) => {
                let left = left.evaluate_values(numbers, predicate, qualify)?;
                match operator {
                    Binary::And if !left.boolean()? => return Ok(Value::Bool(false)),
                    Binary::Or if left.boolean()? => return Ok(Value::Bool(true)),
                    _ => {}
                }
                let right = right.evaluate_values(numbers, predicate, qualify)?;
                match operator {
                    Binary::And | Binary::Or => Ok(Value::Bool(right.boolean()?)),
                    Binary::Equal | Binary::NotEqual => {
                        let equal = match (left, right) {
                            (Value::Number(left), Value::Number(right)) => left == right,
                            (Value::Bool(left), Value::Bool(right)) => left == right,
                            _ => return Err("equality requires operands of the same type".into()),
                        };
                        Ok(Value::Bool(if *operator == Binary::Equal {
                            equal
                        } else {
                            !equal
                        }))
                    }
                    _ => {
                        let left = left.number()?;
                        let right = right.number()?;
                        let result = match operator {
                            Binary::Add => left + right,
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
                            .evaluate_values(numbers, predicate, qualify)?
                            .number()
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let result = match function {
                    Function::Abs => arguments[0].abs(),
                    Function::Min => arguments[0].min(arguments[1]),
                    Function::Max => arguments[0].max(arguments[1]),
                    Function::Clamp => {
                        if arguments[1] > arguments[2] {
                            return Err("clamp minimum must not exceed maximum".into());
                        }
                        arguments[0].max(arguments[1]).min(arguments[2])
                    }
                    Function::Sin => arguments[0].sin(),
                    Function::Cos => arguments[0].cos(),
                    Function::Sqrt => {
                        if arguments[0] < 0.0 {
                            return Err("sqrt requires a nonnegative number".into());
                        }
                        arguments[0].sqrt()
                    }
                    Function::Atan2 => arguments[0].atan2(arguments[1]),
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
        let kind = if identifier_start(byte) {
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
            "observed" | "examined" | "committed" | "reopened"
        ) {
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
    matches!(name, "observed" | "examined" | "committed" | "reopened")
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

/// Validate every definition, including unused functions. Functions return
/// numbers, accept numeric arguments, capture no state or graph data, and may
/// not recurse. Numeric constants in bodies are literal constants; callers
/// pass named coordinates explicitly as arguments.
pub fn validate_functions(functions: &BTreeMap<String, FunctionDef>) -> Result<(), String> {
    if functions.len() > MAX_FUNCTIONS {
        return Err(format!("source exceeds function limit {MAX_FUNCTIONS}"));
    }
    for (name, definition) in functions {
        if name != &definition.name || !plain_identifier(name) {
            return Err(format!("invalid source function name {}", definition.name));
        }
        if Function::named(name).is_some() || predicate_name(name) || name == "qualified" {
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
        definition
            .body
            .validate_calls(
                &parameters,
                &|_, _| Err("pure source functions cannot query graph predicates".into()),
                &|called, arity| {
                    let callee = functions
                        .get(called)
                        .ok_or_else(|| format!("unknown source function {called}"))?;
                    check_arity(callee, arity)
                },
            )
            .and_then(|kind| kind.require(ValueType::Number))
            .map_err(|error| format!("source function {name}: {error}"))?;
    }
    let mut visited = HashSet::new();
    let mut active = HashSet::new();
    for name in functions.keys() {
        visit_function(name, functions, &mut visited, &mut active)?;
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
        Node::Unary(_, child) | Node::Qualified(child, _, _) => collect_calls(child, calls),
        Node::Binary(_, left, right) => {
            collect_calls(left, calls);
            collect_calls(right, calls);
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

fn visit_function(
    name: &str,
    functions: &BTreeMap<String, FunctionDef>,
    visited: &mut HashSet<String>,
    active: &mut HashSet<String>,
) -> Result<(), String> {
    if active.contains(name) {
        return Err(format!("recursive source function cycle involving {name}"));
    }
    if visited.contains(name) {
        return Ok(());
    }
    active.insert(name.into());
    let mut calls = Vec::new();
    collect_calls(&functions[name].body, &mut calls);
    for callee in calls {
        visit_function(callee, functions, visited, active)?;
    }
    active.remove(name);
    visited.insert(name.into());
    Ok(())
}

struct Expander<'a> {
    functions: &'a BTreeMap<String, FunctionDef>,
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
            Node::Variable(name) => {
                if let Some(parameters) = parameters {
                    let argument = parameters
                        .get(name)
                        .ok_or_else(|| format!("pure source function cannot capture {name}"))?;
                    // The argument is already expanded in its caller's scope.
                    // Do not substitute again if a caller name matches a formal.
                    return self.walk(argument, None);
                }
                Node::Variable(name.clone())
            }
            Node::Predicate(name, target) => {
                if parameters.is_some() {
                    return Err("pure source functions cannot query graph predicates".into());
                }
                Node::Predicate(name.clone(), target.clone())
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
    if functions.len() > MAX_FUNCTIONS {
        return Err(format!("source exceeds function limit {MAX_FUNCTIONS}"));
    }
    Expander {
        functions,
        active: Vec::new(),
        remaining: MAX_EXPANDED_NODES,
    }
    .walk(expression, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

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
            "abs",
            "min",
            "max",
            "clamp",
            "sin",
            "cos",
            "sqrt",
            "atan2",
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
    fn pure_functions_cannot_capture_state_coordinates_graph_or_boolean_values() {
        for body in [
            "heat + x",
            "reef_one.x + x",
            "observed(signal)",
            "examined(risk)",
            "true",
            "x > 0",
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
