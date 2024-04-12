use core::cmp::Ordering;
use core::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Identifier {
    Name(Word),
    FieldAccess {
        identifier: Box<Identifier>,
        field: Word,
        dot_location: Location,
    },
}

impl Display for Identifier {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return match self {
            Identifier::Name(name) => name.fmt(formatter),
            Identifier::FieldAccess {
                identifier, field, ..
            } => write!(formatter, "{}.{}", identifier, field),
        };
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Object {
    Anchor(Anchor),
    Function(Function),
    Name(Word),
    Null(Null),
    String(StringLiteral),
    FieldAccess {
        object: Box<Object>,
        field: Word,
        dot_location: Location,
    },
    MethodAccess {
        object: Box<Object>,
        function: Function,
        dot_location: Location,
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
            Object::Anchor(anchor) => anchor.fmt(formatter),
            Object::Function(function) => function.fmt(formatter),
            Object::Name(name) => name.fmt(formatter),
            Object::String(inner) => inner.fmt(formatter),
            Object::Null(inner) => inner.fmt(formatter),
            Object::FieldAccess { object, field, .. } => write!(formatter, "{}.{}", object, field),
            Object::MethodAccess {
                object, function, ..
            } => {
                object.fmt(formatter)?;
                formatter.write_str(".")?;
                function.fmt(formatter)
            }
            Object::ArrayAccess { object, index, .. } => write!(formatter, "{}[{}]", object, index),
        };
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) arguments: Vec<FunctionArgument>,
    pub(crate) name_location: Location,
    pub(crate) opening_bracket_location: Location,
    pub(crate) closing_bracket_location: Location,
}

impl Display for Function {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(&self.name)?;
        match self.arguments.len() {
            0 => formatter.write_str("()"),
            len => {
                formatter.write_str("(")?;
                for argument in &self.arguments[1..len] {
                    formatter.write_str(", ")?;
                    argument.fmt(formatter)?;
                }
                formatter.write_str(")")
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Anchor {
    pub(crate) name: Word,
    pub(crate) opening_bracket_location: Location,
    pub(crate) closing_bracket_location: Location,
}

impl Display for Anchor {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("!{")?;
        self.name.fmt(formatter)?;
        formatter.write_str("}")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct FunctionArgument {
    pub(crate) argument: Argument,
    pub(crate) comma_location: Option<Location>,
}

impl Display for FunctionArgument {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        self.argument.fmt(formatter)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Argument {
    Anchor(Anchor),
    Function(Function),
    Null(Null),
    Number(Number),
    Object(Interpolation),
    SignedNumber(SignedNumber),
    String(StringLiteral),
}

impl Display for Argument {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Argument::Anchor(anchor) => anchor.fmt(formatter),
            Argument::Function(function) => function.fmt(formatter),
            Argument::Null(null) => null.fmt(formatter),
            Argument::Number(number) => number.fmt(formatter),
            Argument::Object(object) => object.fmt(formatter),
            Argument::SignedNumber(number) => number.fmt(formatter),
            Argument::String(string) => string.fmt(formatter),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Word {
    pub(crate) fragments: Vec<WordFragment>,
}

impl Display for Word {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        for fragment in &self.fragments {
            fragment.fmt(formatter)?;
        }
        return Ok(());
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum WordFragment {
    String(StringLiteral),
    Interpolation(Interpolation),
}

impl Display for WordFragment {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            WordFragment::String(string) => string.fmt(formatter),
            WordFragment::Interpolation(interpolation) => interpolation.fmt(formatter),
        }
    }
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
pub(crate) struct StringLiteral {
    pub(crate) content: String,
    pub(crate) location: Location,
}

impl Display for StringLiteral {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return formatter.write_str(&self.content);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Null {
    pub(crate) location: Location,
}

impl Display for Null {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return formatter.write_str("null");
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Expression {
    Number(Number),
    Object(Box<Interpolation>),
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
    Ternary {
        condition: Box<Condition>,
        left: Box<Expression>,
        right: Box<Expression>,
        question_mark_location: Location,
        colon_location: Location,
    },
}

impl Display for Expression {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return match self {
            Expression::Number(number) => number.fmt(formatter),
            Expression::Object(interpolation) => interpolation.fmt(formatter),
            Expression::SignedExpression {
                expression, sign, ..
            } => write!(formatter, "{}{}", sign, expression),
            Expression::BracketedExpression { expression, .. } => {
                write!(formatter, "({})", expression)
            }
            Expression::BinaryOperation {
                left,
                operator,
                right,
                ..
            } => write!(formatter, "{} {} {}", left, operator, right),
            Expression::Ternary {
                condition,
                left,
                right,
                ..
            } => write!(formatter, "{} ? {} : {}", condition, left, right),
        };
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Number {
    pub(crate) content: String,
    pub(crate) location: Location,
}

impl Display for Number {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(&self.content)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SignedNumber {
    pub(crate) sign: Sign,
    pub(crate) sign_location: Location,
    pub(crate) number: Number,
}

impl Display for SignedNumber {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        self.sign.fmt(formatter)?;
        self.number.fmt(formatter)
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
    Object(Interpolation),
    Function(Function),
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
    NegatedCondition {
        condition: Box<Condition>,
        exclamation_mark_location: Location,
    },
    Comparisson {
        left: Box<Comparable>,
        operator: ComparissonOperator,
        right: Box<Comparable>,
        operator_location: Location,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Comparable {
    Condition(Condition),
    Expression(Expression),
    Function(Function),
    Object(Interpolation),
    String(StringLiteral),
    Null(Null),
}

impl Display for Comparable {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Comparable::Condition(inner) => inner.fmt(formatter),
            Comparable::Expression(inner) => inner.fmt(formatter),
            Comparable::Function(inner) => inner.fmt(formatter),
            Comparable::Object(inner) => inner.fmt(formatter),
            Comparable::String(inner) => inner.fmt(formatter),
            Comparable::Null(inner) => inner.fmt(formatter),
        }
    }
}

impl Into<Comparable> for UndecidedExpressionContent {
    fn into(self) -> Comparable {
        return match self {
            UndecidedExpressionContent::Expression(expression) => {
                Comparable::Expression(expression)
            }
            UndecidedExpressionContent::Condition(condition) => Comparable::Condition(condition),
            UndecidedExpressionContent::Name(interpolation) => Comparable::Object(interpolation),
            UndecidedExpressionContent::String(string) => Comparable::String(string),
            UndecidedExpressionContent::Null(null) => Comparable::Null(null),
        };
    }
}

impl Comparable {
    pub(crate) fn r#type(&self) -> &str {
        match self {
            Comparable::Condition(_) => "condition",
            Comparable::Expression(_) => "expression",
            Comparable::Function(_) => "function",
            Comparable::Object(_) => "object",
            Comparable::String(_) => "string",
            Comparable::Null(_) => "null",
        }
    }
}

impl Display for Condition {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return match self {
            Condition::True { .. } => formatter.write_str("true"),
            Condition::False { .. } => formatter.write_str("false"),
            Condition::Object(interpolation) => interpolation.fmt(formatter),
            Condition::Function(function) => function.fmt(formatter),
            Condition::BracketedCondition { condition, .. } => write!(formatter, "({})", condition),
            Condition::BinaryOperation {
                left,
                operator,
                right,
                ..
            } => write!(formatter, "{} {} {}", left, operator, right),
            Condition::NegatedCondition { condition, .. } => write!(formatter, "!{}", condition),
            Condition::Comparisson {
                left,
                operator,
                right,
                ..
            } => write!(formatter, "{} {} {}", left, operator, right),
        };
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ConditionOperator {
    And,
    Or,
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
pub(crate) enum ComparissonOperator {
    Equal,
    Unequal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl Display for ComparissonOperator {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return formatter.write_str(match self {
            ComparissonOperator::Equal => "==",
            ComparissonOperator::Unequal => "!=",
            ComparissonOperator::GreaterThan => ">",
            ComparissonOperator::GreaterThanOrEqual => ">=",
            ComparissonOperator::LessThan => "<",
            ComparissonOperator::LessThanOrEqual => "<=",
        });
    }
}

// TODO: better name!
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum UndecidedExpressionContent {
    Expression(Expression),
    Condition(Condition),
    Name(Interpolation),
    String(StringLiteral),
    Null(Null),
}

impl Display for UndecidedExpressionContent {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return match self {
            UndecidedExpressionContent::Expression(expression) => expression.fmt(formatter),
            UndecidedExpressionContent::Condition(condition) => condition.fmt(formatter),
            UndecidedExpressionContent::Name(interpolation) => interpolation.fmt(formatter),
            UndecidedExpressionContent::String(string) => string.fmt(formatter),
            UndecidedExpressionContent::Null(null) => null.fmt(formatter),
        };
    }
}

impl UndecidedExpressionContent {
    pub(crate) fn r#type(&self) -> &str {
        return match self {
            UndecidedExpressionContent::Expression(_) => "expression",
            UndecidedExpressionContent::Condition(_) => "condition",
            UndecidedExpressionContent::Name(_) => "object",
            UndecidedExpressionContent::String(_) => "string",
            UndecidedExpressionContent::Null(_) => "null",
        };
    }
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

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Uri {
    Literal(UriLiteral),
    Object(Interpolation),
}

impl Display for Uri {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        return match self {
            Uri::Literal(literal) => literal.fmt(formatter),
            Uri::Object(object) => object.fmt(formatter),
        };
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct UriLiteral {
    pub(crate) fragments: Vec<UriFragment>,
    pub(crate) file_extension: Option<UriFileExtension>,
}

impl Display for UriLiteral {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        for fragment in &self.fragments {
            fragment.fmt(formatter)?;
        }
        if let Some(extension) = &self.file_extension {
            extension.fmt(formatter)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct UriFragment {
    pub(crate) slash_location: Location,
    pub(crate) content: Word,
}

impl Display for UriFragment {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("/")?;
        return self.content.fmt(formatter);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct UriFileExtension {
    pub(crate) dot_location: Location,
    pub(crate) content: Word,
}

impl Display for UriFileExtension {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(".")?;
        return self.content.fmt(formatter);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Regex {
    pub(crate) location: Location,
}

#[cfg(test)]
mod tests {
    use crate::spel::ast::{Interpolation, Location, Object, StringLiteral, Word, WordFragment};

    #[test]
    fn test_format_interpolated_word() {
        assert_eq!(
            format!(
                "{}",
                Word {
                    fragments: vec![
                        WordFragment::String(StringLiteral {
                            content: "hello-".to_string(),
                            location: Location::VariableLength {
                                line: 0,
                                char: 3,
                                length: 6,
                            }
                        }),
                        WordFragment::Interpolation(Interpolation {
                            content: Object::Name(Word {
                                fragments: vec![WordFragment::String(StringLiteral {
                                    content: "nice".to_string(),
                                    location: Location::VariableLength {
                                        line: 0,
                                        char: 11,
                                        length: 4,
                                    }
                                })]
                            }),
                            opening_bracket_location: Location::DoubleCharacter {
                                line: 0,
                                char: 9
                            },
                            closing_bracket_location: Location::SingleCharacter {
                                line: 0,
                                char: 15
                            }
                        }),
                        WordFragment::String(StringLiteral {
                            content: "-world".to_string(),
                            location: Location::VariableLength {
                                line: 0,
                                char: 16,
                                length: 6,
                            }
                        }),
                    ],
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
                Object::String(StringLiteral {
                    content: "'some  string'".to_string(),
                    location: Location::VariableLength {
                        line: 0,
                        char: 0,
                        length: 15,
                    }
                })
            ),
            "'some  string'"
        );
    }
}
