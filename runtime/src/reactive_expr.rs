//! Bounded, typed expressions for source-authored reactive game rules.
//!
//! Expressions can read numeric state and coordinates, and query the host's
//! epistemic graph. They cannot mutate state or invoke arbitrary host code.

use crate::presentation::Number;
use std::collections::HashSet;

const MAX_TOKENS: usize = 1024;
const MAX_NESTING: usize = 64;
const MAX_SOURCE_BYTES: usize = 65_536;

/// A parsed expression with finite, canonical number literals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    node: Node,
    depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Number(Number),
    Bool(bool),
    Variable(String),
    Unary(Unary, Box<Expr>),
    Binary(Binary, Box<Expr>, Box<Expr>),
    Function(Function, Vec<Expr>),
    Predicate(String, String),
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
        }
    }

    fn arity(self) -> usize {
        match self {
            Self::Abs | Self::Sin | Self::Cos | Self::Sqrt => 1,
            Self::Min | Self::Max => 2,
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
            Node::Unary(_, child) => child.depth + 1,
            Node::Binary(_, left, right) => left.depth.max(right.depth) + 1,
            Node::Function(_, children) => {
                children.iter().map(|child| child.depth).max().unwrap_or(0) + 1
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
            Node::Unary(operator, child) => {
                let expected = match operator {
                    Unary::Not => ValueType::Bool,
                    _ => ValueType::Number,
                };
                child.validate(numeric, predicate)?.require(expected)?;
                Ok(expected)
            }
            Node::Binary(operator, left, right) => {
                let left_type = left.validate(numeric, predicate)?;
                let right_type = right.validate(numeric, predicate)?;
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
            Node::Function(_, arguments) => {
                for argument in arguments {
                    argument
                        .validate(numeric, predicate)?
                        .require(ValueType::Number)?;
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
        match &self.node {
            Node::Number(value) => Ok(Value::Number(value.value())),
            Node::Bool(value) => Ok(Value::Bool(*value)),
            Node::Variable(name) => {
                let value =
                    numbers(name).ok_or_else(|| format!("unknown numeric identifier {name}"))?;
                Ok(Value::Number(finite(value)?))
            }
            Node::Predicate(name, target) => Ok(Value::Bool(predicate(name, target)?)),
            Node::Unary(operator, child) => {
                let value = child.evaluate(numbers, predicate)?;
                match operator {
                    Unary::Positive => Ok(Value::Number(value.number()?)),
                    Unary::Negative => Ok(Value::Number(-value.number()?)),
                    Unary::Not => Ok(Value::Bool(!value.boolean()?)),
                }
            }
            Node::Binary(operator, left, right) => {
                let left = left.evaluate(numbers, predicate)?;
                match operator {
                    Binary::And if !left.boolean()? => return Ok(Value::Bool(false)),
                    Binary::Or if left.boolean()? => return Ok(Value::Bool(true)),
                    _ => {}
                }
                let right = right.evaluate(numbers, predicate)?;
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
                    .map(|argument| argument.evaluate(numbers, predicate)?.number())
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
        let function =
            Function::named(&name).ok_or_else(|| format!("unknown expression function {name}"))?;
        let mut arguments = Vec::new();
        if self.peek() != Some(&TokenKind::RightParen) {
            loop {
                if arguments.len() == function.arity() {
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
        if arguments.len() != function.arity() {
            return Err(format!(
                "{} requires {} arguments",
                function.name(),
                function.arity()
            ));
        }
        Ok(Node::Function(function, arguments))
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
    let mut parser = Parser {
        tokens: tokenize(input)?,
        cursor: 0,
        end: input.len(),
    };
    let expression = parser.expression(0, 0)?;
    if parser.peek().is_some() {
        return Err(format!("unexpected token at byte {}", parser.offset()));
    }
    Ok(expression)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn eval(input: &str) -> Result<Value, String> {
        parse(input)?.evaluate(&|_| None, &|_, _| Err("unexpected predicate".into()))
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
