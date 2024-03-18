use core::cmp::Ordering;
use core::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Object {
    Anchor(Word),
    Function(Word, Vec<Object>),
    Name(Word),
    String(String, Vec<Interpolation>),
    Null,
    FieldAccess(Box<Object>, Word),
    MethodAccess(Box<Object>, Word, Vec<Object>),
    ArrayAccess(Box<Object>, Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Expression {
    Number(String),
    SignedExpression(Box<Expression>, Sign),
    BracketedExpression(Box<Expression>),
    BinaryOperation(Box<Expression>, Operation, Box<Expression>),
}

impl Display for Expression {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return match self {
            Expression::Number(string) => formatter.write_str(string),
            Expression::SignedExpression(expression, sign) => {
                formatter.write_fmt(format_args!("{}{}", sign, expression))
            }
            Expression::BracketedExpression(expression) => {
                formatter.write_fmt(format_args!("({})", expression))
            }
            Expression::BinaryOperation(left, operation, right) => {
                formatter.write_fmt(format_args!("{} {} {}", left, operation, right))
            }
        };
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Sign {
    Plus,
    Minus,
}

impl Display for Sign {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return formatter.write_str(match self {
            Sign::Plus => "+",
            Sign::Minus => "-",
        });
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Operation {
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Modulo,
    Power,
}

impl Display for Operation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return formatter.write_str(match self {
            Operation::Addition => "+",
            Operation::Subtraction => "-",
            Operation::Division => "/",
            Operation::Multiplication => "*",
            Operation::Modulo => "%",
            Operation::Power => "^",
        });
    }
}

impl PartialOrd for Operation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self {
            Operation::Addition | Operation::Subtraction => match other {
                Operation::Addition | Operation::Subtraction => Ordering::Equal,
                _ => Ordering::Greater,
            },
            Operation::Division | Operation::Multiplication | Operation::Modulo => match other {
                Operation::Addition | Operation::Subtraction => Ordering::Less,
                Operation::Division | Operation::Multiplication | Operation::Modulo => {
                    Ordering::Equal
                }
                _ => Ordering::Greater,
            },
            Operation::Power => match other {
                Operation::Power => Ordering::Equal,
                _ => Ordering::Less,
            },
        })
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub(crate) struct Word {
    pub(crate) name: String,
    pub(crate) interpolations: Vec<Interpolation>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Interpolation {
    pub(crate) content: Object,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ObjectAst {
    pub(crate) root: Object,
}

impl ObjectAst {
    pub(crate) fn new(object: Object) -> Self {
        return Self { root: object };
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ExpressionAst {
    pub(crate) root: Expression,
}

impl ExpressionAst {
    pub(crate) fn new(expression: Expression) -> Self {
        return Self { root: expression };
    }
}
