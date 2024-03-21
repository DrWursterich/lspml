use core::cmp::Ordering;
use core::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Object {
    Anchor {
        name: Word,
        opening_bracket_location: Location,
        closing_bracket_location: Location,
    },
    Function {
        name: Word,
        arguments: Vec<Object>,
        opening_bracket_location: Location,
        closing_bracket_location: Location,
    },
    Name {
        name: Word,
    },
    Null {
        location: Location,
    },
    String {
        content: String,
        location: Location,
    },
    FieldAccess {
        object: Box<Object>,
        field: Word,
        dot_location: Location,
    },
    MethodAccess {
        object: Box<Object>,
        method: Word,
        arguments: Vec<Object>,
        dot_location: Location,
        opening_bracket_location: Location,
        closing_bracket_location: Location,
    },
    ArrayAccess {
        object: Box<Object>,
        index: Expression,
        opening_bracket_location: Location,
        closing_bracket_location: Location,
    },
}

impl Display for Object {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return match self {
            Object::Anchor { name, .. } => name.fmt(formatter),
            Object::Function {
                name, arguments, ..
            } => {
                name.fmt(formatter)?;
                fmt_arguments(formatter, arguments)
            }
            Object::Name { name } => name.fmt(formatter),
            Object::String { content, .. } => formatter.write_str(content),
            Object::Null { .. } => formatter.write_str("null"),
            Object::FieldAccess { object, field, .. } => write!(formatter, "{}.{}", object, field),
            Object::MethodAccess {
                object,
                method,
                arguments,
                ..
            } => {
                write!(formatter, "{}.{}", object, method)?;
                fmt_arguments(formatter, arguments)
            }
            Object::ArrayAccess { object, index, .. } => write!(formatter, "{}[{}]", object, index),
        };
    }
}

fn fmt_arguments(formatter: &mut Formatter<'_>, arguments: &Vec<Object>) -> core::fmt::Result {
    match arguments.len() {
        0 => formatter.write_str("()"),
        len => {
            formatter.write_str("(")?;
            for argument in &arguments[1..len] {
                formatter.write_str(",")?;
                argument.fmt(formatter)?;
            }
            formatter.write_str(")")
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Word {
    pub(crate) name: String,
    pub(crate) interpolations: Vec<Interpolation>,
    pub(crate) location: Location,
}

impl Display for Word {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return fmt_interpolations(
            formatter,
            &self.name,
            &self.interpolations,
            self.location.char() as usize,
        );
    }
}

fn fmt_interpolations(
    formatter: &mut Formatter<'_>,
    string: &String,
    interpolations: &Vec<Interpolation>,
    offset: usize,
) -> core::fmt::Result {
    let mut index;
    let mut last_index = 0;
    let indices = &mut string.char_indices();
    for interpolation in interpolations {
        index = interpolation.opening_bracket_location.char() as usize - offset;
        formatter.write_str(&string[last_index..indices.nth(index).unwrap().0])?;
        interpolation.fmt(formatter)?;
        last_index = index;
    }
    formatter.write_str(&string[last_index..])
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Interpolation {
    pub(crate) content: Object,
    pub(crate) opening_bracket_location: Location,
    pub(crate) closing_bracket_location: Location,
}

impl Display for Interpolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("${")?;
        self.content.fmt(formatter)?;
        formatter.write_str("}")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Expression {
    Number {
        content: String,
        location: Location,
    },
    SignedExpression {
        expression: Box<Expression>,
        sign: Sign,
        sign_location: Location,
    },
    BracketedExpression {
        expression: Box<Expression>,
        opening_bracket_location: Location,
        closing_bracket_location: Location,
    },
    BinaryOperation {
        left: Box<Expression>,
        operator: ExpressionOperator,
        right: Box<Expression>,
        operator_location: Location,
    },
}

impl Display for Expression {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return match self {
            Expression::Number { content, .. } => formatter.write_str(content),
            Expression::SignedExpression {
                expression, sign, ..
            } => formatter.write_fmt(format_args!("{}{}", sign, expression)),
            Expression::BracketedExpression { expression, .. } => {
                formatter.write_fmt(format_args!("({})", expression))
            }
            Expression::BinaryOperation {
                left,
                operator: operation,
                right,
                ..
            } => formatter.write_fmt(format_args!("{} {} {}", left, operation, right)),
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
            Sign::Plus { .. } => "+",
            Sign::Minus { .. } => "-",
        });
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ExpressionOperator {
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Modulo,
    Power,
}

impl Display for ExpressionOperator {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return formatter.write_str(match self {
            ExpressionOperator::Addition => "+",
            ExpressionOperator::Subtraction => "-",
            ExpressionOperator::Division => "/",
            ExpressionOperator::Multiplication => "*",
            ExpressionOperator::Modulo => "%",
            ExpressionOperator::Power => "^",
        });
    }
}

impl PartialOrd for ExpressionOperator {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self {
            ExpressionOperator::Addition | ExpressionOperator::Subtraction => match other {
                ExpressionOperator::Addition | ExpressionOperator::Subtraction => Ordering::Equal,
                _ => Ordering::Greater,
            },
            ExpressionOperator::Division
            | ExpressionOperator::Multiplication
            | ExpressionOperator::Modulo => match other {
                ExpressionOperator::Addition | ExpressionOperator::Subtraction => Ordering::Less,
                ExpressionOperator::Division
                | ExpressionOperator::Multiplication
                | ExpressionOperator::Modulo => Ordering::Equal,
                _ => Ordering::Greater,
            },
            ExpressionOperator::Power => match other {
                ExpressionOperator::Power => Ordering::Equal,
                _ => Ordering::Less,
            },
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Condition {
    True {
        location: Location,
    },
    False {
        location: Location,
    },
    BinaryOperation {
        left: Box<Condition>,
        operator: ConditionOperator,
        right: Box<Condition>,
        operator_location: Location,
    },
    BracketedCondition {
        condition: Box<Condition>,
        opening_bracket_location: Location,
        closing_bracket_location: Location,
    },
}

impl Display for ConditionOperator {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return formatter.write_str(match self {
            ConditionOperator::And => "&&",
            ConditionOperator::Or => "||",
        });
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ConditionOperator {
    And,
    Or,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Location {
    SingleCharacter { char: u16, line: u16 },
    DoubleCharacter { char: u16, line: u16 },
    VariableLength { char: u16, line: u16, length: u16 },
}

impl Location {
    pub(crate) fn char(&self) -> u16 {
        return match self {
            Location::SingleCharacter { char, .. } => *char,
            Location::DoubleCharacter { char, .. } => *char,
            Location::VariableLength { char, .. } => *char,
        };
    }

    pub(crate) fn line(&self) -> u16 {
        return match self {
            Location::SingleCharacter { line, .. } => *line,
            Location::DoubleCharacter { line, .. } => *line,
            Location::VariableLength { line, .. } => *line,
        };
    }

    pub(crate) fn len(&self) -> u16 {
        return match self {
            Location::SingleCharacter { .. } => 1 as u16,
            Location::DoubleCharacter { .. } => 2 as u16,
            Location::VariableLength { length, .. } => *length,
        };
    }
}

impl Display for Location {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "({}, {}, {})",
            self.char(),
            self.line(),
            self.len()
        )
    }
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

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ConditionAst {
    pub(crate) root: Condition,
}

impl ConditionAst {
    pub(crate) fn new(condition: Condition) -> Self {
        return Self { root: condition };
    }
}

#[cfg(test)]
mod tests {
    use crate::spel::ast::{Interpolation, Location, Object, Word};

    #[test]
    fn test_format_interpolated_word() {
        assert_eq!(
            format!(
                "{}",
                Word {
                    name: "hello--world".to_string(),
                    interpolations: vec![Interpolation {
                        content: Object::Name {
                            name: Word {
                                name: "nice".to_string(),
                                interpolations: vec![],
                                location: Location::VariableLength {
                                    line: 0,
                                    char: 11,
                                    length: 4,
                                }
                            }
                        },
                        opening_bracket_location: Location::DoubleCharacter { line: 0, char: 9 },
                        closing_bracket_location: Location::SingleCharacter { line: 0, char: 15 }
                    }],
                    location: Location::VariableLength {
                        line: 0,
                        char: 3,
                        length: 21,
                    }
                }
            ),
            "hello-${nice}-world"
        );
    }

    #[test]
    fn test_format_string() {
        assert_eq!(
            format!(
                "{}",
                Object::String {
                    content: "'some  string'".to_string(),
                    location: Location::VariableLength {
                        line: 0,
                        char: 0,
                        length: 15,
                    }
                }
            ),
            "'some  string'"
        );
    }
}
