use std::fmt::Display;

use anyhow::Result;
use core::fmt;
use lsp_types::{Position, Range, TextEdit};

use super::{
    ast::{
        Anchor, Argument, Comparable, ComparissonOperator, Condition, ConditionAst,
        ConditionOperator, Expression, ExpressionAst, ExpressionOperator, Function,
        FunctionArgument, Identifier, Interpolation, Location, Null, Number, Object, ObjectAst,
        Query, Regex, Sign, SignedNumber, StringLiteral, UndecidedExpressionContent, Uri,
        UriFileExtension, UriFragment, UriLiteral, Word, WordFragment,
    },
    Scanner,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SyntaxFix {
    Insert(Position, String),
    Delete(Location),
    Replace(Location, String),
}

const ADDITIONAL_CHARS_IN_ANCHOR: [char; 1] = ['.'];

impl SyntaxFix {
    pub(crate) fn to_text_edit(&self, offset: &Position) -> TextEdit {
        return match self {
            SyntaxFix::Insert(position, text) => {
                let position = Position {
                    line: offset.line + position.line,
                    character: offset.character + position.character,
                };
                TextEdit {
                    range: Range {
                        start: position,
                        end: position,
                    },
                    new_text: text.clone(),
                }
            }
            SyntaxFix::Delete(location) => {
                let line = offset.line + location.line() as u32;
                let start = offset.character + location.char() as u32;
                TextEdit {
                    range: Range {
                        start: Position {
                            line,
                            character: start,
                        },
                        end: Position {
                            line,
                            character: start + location.len() as u32,
                        },
                    },
                    new_text: "".to_string(),
                }
            }
            SyntaxFix::Replace(location, text) => {
                let line = offset.line + location.line() as u32;
                let start = offset.character + location.char() as u32;
                TextEdit {
                    range: Range {
                        start: Position {
                            line,
                            character: start,
                        },
                        end: Position {
                            line,
                            character: start + location.len() as u32,
                        },
                    },
                    new_text: text.clone(),
                }
            }
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SyntaxError {
    pub(crate) message: Box<str>,
    pub(crate) proposed_fixes: Vec<SyntaxFix>,
}

impl Display for SyntaxError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

macro_rules! syntax_error {
    ($msg:literal $(,)?) => {
        SyntaxError {
            message: $msg.into(),
            proposed_fixes: vec![],
        }
    };
    ($fmt:expr, $($arg:tt)* $(,)?) => {
        SyntaxError {
            message: format!($fmt, $($arg)*).into(),
            proposed_fixes: vec![],
        }
    };
    (msg: $msg:literal, fix: [$($fix:expr),+ $(,)?] $(,)?) => {
        SyntaxError {
            message: $msg.into(),
            proposed_fixes: vec![$($fix)*],
        }
    };
    (fmt: ($fmt:expr, $($arg:tt)*), fix: [$($fix:expr),+ $(,)?] $(,)?) => {
        SyntaxError {
            message: format!($fmt, $($arg)*).into(),
            proposed_fixes: vec![$($fix)*],
        }
    };
}

pub(crate) struct Parser {
    scanner: Scanner,
}

impl Parser {
    pub(crate) fn new(string: &str) -> Self {
        return Self {
            scanner: Scanner::new(string),
        };
    }

    pub(crate) fn parse_object_ast(&mut self) -> Result<ObjectAst, SyntaxError> {
        self.scanner.skip_whitespace();
        if self.scanner.is_done() {
            return Err(syntax_error!("string is empty"));
        }
        let root = self.parse_object()?;
        self.scanner.skip_whitespace();
        match self.scanner.is_done() {
            true => Ok(ObjectAst::new(root)),
            false => Err(self.trailing_characters_error()),
        }
    }

    pub(crate) fn parse_expression_ast(&mut self) -> Result<ExpressionAst, SyntaxError> {
        self.scanner.skip_whitespace();
        if self.scanner.is_done() {
            return Err(syntax_error!("string is empty"));
        }
        let root = self.parse_expression()?;
        self.scanner.skip_whitespace();
        match self.scanner.is_done() {
            true => Ok(ExpressionAst::new(root)),
            false => Err(self.trailing_characters_error()),
        }
    }

    pub(crate) fn parse_condition_ast(&mut self) -> Result<ConditionAst, SyntaxError> {
        self.scanner.skip_whitespace();
        if self.scanner.is_done() {
            return Err(syntax_error!("string is empty"));
        }
        let root = self.parse_condition()?;
        self.scanner.skip_whitespace();
        match self.scanner.is_done() {
            true => Ok(ConditionAst::new(root)),
            false => Err(self.trailing_characters_error()),
        }
    }

    pub(crate) fn parse_text(&mut self) -> Result<Word, SyntaxError> {
        let mut string = String::new();
        let mut fragments = Vec::new();
        let mut start = self.scanner.cursor as u16;
        loop {
            match self.scanner.peek() {
                Some('$') => {
                    let interpolation = self.parse_interpolation()?;
                    let length = string.len() as u16;
                    if length > 0 {
                        fragments.push(WordFragment::String(StringLiteral {
                            content: string.into(),
                            location: Location::VariableLength {
                                char: start,
                                line: 0,
                                length,
                            },
                        }));
                        string = String::new();
                        start = self.scanner.cursor as u16;
                    }
                    fragments.push(WordFragment::Interpolation(interpolation))
                }
                Some(char) => {
                    string.push(*char);
                    self.scanner.pop();
                }
                None => break,
            }
        }
        let length = string.len() as u16;
        if length > 0 || fragments.len() == 0 {
            fragments.push(WordFragment::String(StringLiteral {
                content: string.into(),
                location: Location::VariableLength {
                    char: start,
                    line: 0,
                    length,
                },
            }));
        }
        return Ok(Word { fragments });
    }

    pub(crate) fn parse_uri(&mut self) -> Result<Uri, SyntaxError> {
        let mut fragments = Vec::new();
        self.scanner.skip_whitespace();
        loop {
            match self.scanner.peek() {
                Some('/') => {
                    let slash_location = Location::SingleCharacter {
                        char: self.scanner.cursor as u16,
                        line: 0,
                    };
                    self.scanner.pop();
                    let content = self.parse_word()?;
                    fragments.push(UriFragment {
                        content,
                        slash_location,
                    });
                }
                Some('.') if fragments.len() != 0 => {
                    let dot_location = Location::SingleCharacter {
                        char: self.scanner.cursor as u16,
                        line: 0,
                    };
                    self.scanner.pop();
                    let content = self.parse_word()?;
                    return Ok(Uri::Literal(UriLiteral {
                        fragments,
                        file_extension: Some(UriFileExtension {
                            content,
                            dot_location,
                        }),
                    }));
                }
                Some('$') if fragments.len() == 0 => {
                    let interpolation = self.parse_interpolation()?;
                    self.scanner.skip_whitespace();
                    return match self.scanner.is_done() {
                        true => Ok(Uri::Object(interpolation)),
                        false => Err(self.trailing_characters_error()),
                    };
                }
                Some(char) => return Err(syntax_error!("unexpected char \"{}\"", char)),
                None => {
                    return Ok(Uri::Literal(UriLiteral {
                        fragments,
                        file_extension: None,
                    }))
                }
            }
        }
    }

    pub(crate) fn parse_regex(&mut self) -> Result<Regex, SyntaxError> {
        if self.scanner.is_done() {
            return Err(syntax_error!("string is empty"));
        }
        let start = self.scanner.cursor as u16;
        let mut length = 0;
        loop {
            match self.scanner.peek() {
                // TODO: ACTUALLY parse the regex
                // Some('[') => todo!(),
                // Some('(') => todo!(),
                // Some('\\') => todo!(),
                // ...
                Some(_char) => {
                    length += 1;
                    self.scanner.pop();
                }
                None => {
                    return Ok(Regex {
                        location: Location::VariableLength {
                            char: start,
                            line: 0,
                            length,
                        },
                    })
                }
            }
        }
    }

    pub(crate) fn parse_query(&mut self) -> Result<Query, SyntaxError> {
        if self.scanner.is_done() {
            return Err(syntax_error!("string is empty"));
        }
        let start = self.scanner.cursor as u16;
        let mut length = 0;
        loop {
            match self.scanner.peek() {
                // TODO: ACTUALLY parse the query
                Some(_char) => {
                    length += 1;
                    self.scanner.pop();
                }
                None => {
                    return Ok(Query {
                        location: Location::VariableLength {
                            char: start,
                            line: 0,
                            length,
                        },
                    })
                }
            }
        }
    }

    pub(crate) fn parse_identifier(&mut self) -> Result<Identifier, SyntaxError> {
        self.scanner.skip_whitespace();
        if self.scanner.is_done() {
            return Err(syntax_error!("string is empty"));
        }
        let name = self.parse_word()?;
        self.scanner.skip_whitespace();
        let mut result = Identifier::Name(name);
        loop {
            match self.scanner.peek() {
                Some('.') => {
                    let dot_location = Location::SingleCharacter {
                        char: self.scanner.cursor as u16,
                        line: 0,
                    };
                    self.scanner.pop();
                    self.scanner.skip_whitespace();
                    let name = self.parse_word()?;
                    self.scanner.skip_whitespace();
                    result = Identifier::FieldAccess {
                        identifier: Box::new(result),
                        field: name,
                        dot_location,
                    };
                }
                Some(_) => return Err(self.trailing_characters_error()),
                None => return Ok(result),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Object, SyntaxError> {
        let object = match self.scanner.peek() {
            Some('\'') => self.parse_string().map(Object::String)?,
            Some('!') => self.parse_interpolated_anchor().map(Object::Anchor)?,
            Some('$' | 'a'..='z' | 'A'..='Z' | '_') => self.parse_name_or_global_function()?,
            Some(char) => return Err(syntax_error!("unexpected char \"{}\"", char)),
            None => return Err(syntax_error!("unexpected end")),
        };
        return self.parse_object_access(object);
    }

    fn parse_expression(&mut self) -> Result<Expression, SyntaxError> {
        let result = self.parse_undecided_expression_content()?;
        return self.resolve_expression_content(result);
    }

    fn parse_undecided_expression_content(
        &mut self,
    ) -> Result<UndecidedExpressionContent, SyntaxError> {
        return match self.scanner.peek() {
            Some('\'') => self.parse_string().map(UndecidedExpressionContent::String),
            Some('$') => self
                .parse_interpolation()
                .map(UndecidedExpressionContent::Name),
            Some('(') => self.parse_bracketed_expression_content(),
            Some('+' | '-') => self
                .parse_signed_expression()
                .map(UndecidedExpressionContent::Expression),
            Some('0'..='9') => self
                .parse_number()
                .map(Expression::Number)
                .map(UndecidedExpressionContent::Expression),
            Some('!') => self
                .parse_negated_condition()
                .map(UndecidedExpressionContent::Condition),
            Some(_) => {
                match self.parse_name_or_global_function()? {
                    Object::Function(function) => {
                        Ok(UndecidedExpressionContent::Function(function))
                    }
                    Object::Name(Word { fragments }) if fragments.len() == 1 => {
                        match &fragments[0] {
                            WordFragment::String(StringLiteral { content, location })
                                if &**content == "null" =>
                            {
                                Ok(UndecidedExpressionContent::Null(Null {
                                    location: location.clone(),
                                }))
                            }
                            WordFragment::String(StringLiteral { content, location })
                                if &**content == "true" =>
                            {
                                Ok(UndecidedExpressionContent::Condition(Condition::True {
                                    location: location.clone(),
                                }))
                            }
                            WordFragment::String(StringLiteral { content, location })
                                if &**content == "false" =>
                            {
                                Ok(UndecidedExpressionContent::Condition(Condition::False {
                                    location: location.clone(),
                                }))
                            }
                            WordFragment::Interpolation(interpolation) => {
                                Ok(UndecidedExpressionContent::Name(interpolation.clone()))
                            }
                            WordFragment::String(StringLiteral { content, location }) => {
                                let new_string = format!("${{{}}}", content);
                                Err(syntax_error! {
                                    fmt: (
                                        "objects in expressions have to be interpolated. try `{}`",
                                        new_string,
                                    ),
                                    fix: [SyntaxFix::Replace(location.to_owned(), new_string)],
                                })
                            }
                        }
                    }
                    Object::Name(word) => {
                        let new_string = format!("${{{}}}", word);
                        let (char, line) = match word.fragments.first().unwrap() {
                            WordFragment::String(StringLiteral { location, .. }) => {
                                (location.line(), location.char())
                            }
                            WordFragment::Interpolation(Interpolation {
                                opening_bracket_location,
                                ..
                            }) => (
                                opening_bracket_location.line(),
                                opening_bracket_location.char(),
                            ),
                        };
                        let length = match word.fragments.last().unwrap() {
                            WordFragment::String(StringLiteral { location, .. }) => {
                                location.char() + location.len()
                            }
                            WordFragment::Interpolation(Interpolation {
                                closing_bracket_location,
                                ..
                            }) => closing_bracket_location.char() + closing_bracket_location.len(),
                        } - char;
                        Err(syntax_error! {
                            fmt: (
                                "objects in expressions have to be interpolated. try `{}`",
                                new_string
                            ),
                            fix: [SyntaxFix::Replace(
                                Location::VariableLength { char, line, length },
                                new_string,
                            )],
                        })
                    }
                    // Object::Anchor(_) => todo!(),
                    // Object::String(_) => todo!(),
                    // Object::FieldAccess { object, field, dot_location } => todo!(),
                    // Object::MethodAccess { object, function, dot_location } => todo!(),
                    // Object::ArrayAccess { object, index, opening_bracket_location, closing_bracket_location } => todo!(),
                    object => Err(syntax_error!("unexpected \"{}\"", object)),
                }
            }
            None => return Err(syntax_error!("unexpected end")),
        };
    }

    fn resolve_expression_content(
        &mut self,
        content: UndecidedExpressionContent,
    ) -> Result<Expression, SyntaxError> {
        self.scanner.skip_whitespace();
        return match self.resolve_undecided_expression_content(content)? {
            UndecidedExpressionContent::Expression(expression) => Ok(expression),
            UndecidedExpressionContent::Name(name) => Ok(Expression::Object(Box::new(name))),
            UndecidedExpressionContent::Null(null) => Ok(Expression::Null(null)),
            UndecidedExpressionContent::Function(function) => Ok(Expression::Function(function)),
            content => Err(syntax_error!("unexpected {}", content.r#type())),
        };
    }

    fn resolve_undecided_expression_content(
        &mut self,
        content: UndecidedExpressionContent,
    ) -> Result<UndecidedExpressionContent, SyntaxError> {
        // TODO: waaaaaay to many .clone()s!
        match content {
            UndecidedExpressionContent::Expression(expression) => {
                let expression = self.try_parse_binary_operation(expression)?;
                return match self
                    .try_parse_comparisson(&Comparable::Expression(expression.clone()))?
                {
                    Some(comparison) => {
                        let condition = self.resolve_comparable(comparison)?;
                        self.resolve_undecided_expression_content(
                            UndecidedExpressionContent::Condition(condition),
                        )
                    }
                    None => Ok(UndecidedExpressionContent::Expression(expression)),
                };
            }
            UndecidedExpressionContent::Function(function) => {
                let function_function = function.clone();
                let expression = Expression::Function(function);
                let expression_function = expression.clone();
                let binary_expression = self.try_parse_binary_operation(expression)?;
                return match self
                    .try_parse_comparisson(&Comparable::Expression(binary_expression.clone()))?
                {
                    Some(comparison) => {
                        let condition = self.resolve_comparable(comparison)?;
                        self.resolve_undecided_expression_content(
                            UndecidedExpressionContent::Condition(condition),
                        )
                    }
                    None => match self.try_parse_binary_condition(&Condition::Function(
                        function_function.clone(),
                    ))? {
                        Some(condition) => self.resolve_undecided_expression_content(
                            UndecidedExpressionContent::Condition(condition),
                        ),
                        None => match self
                            .try_parse_ternary(&Condition::Function(function_function))?
                        {
                            Some(expression) => self.resolve_undecided_expression_content(
                                UndecidedExpressionContent::Expression(expression),
                            ),
                            None => Ok(UndecidedExpressionContent::Expression(expression_function)),
                            // TODO: if try_parse_binary_operation did not find any operator this
                            // should be an UndecidedExpressionContent::Function !
                        },
                    },
                };
            }
            UndecidedExpressionContent::Condition(condition) => {
                return match self
                    .try_parse_comparisson(&Comparable::Condition(condition.clone()))?
                {
                    Some(comparison) => {
                        let condition = self.resolve_comparable(comparison)?;
                        self.resolve_undecided_expression_content(
                            UndecidedExpressionContent::Condition(condition),
                        )
                    }
                    None => match self.try_parse_binary_condition(&condition)? {
                        Some(condition) => self.resolve_undecided_expression_content(
                            UndecidedExpressionContent::Condition(condition),
                        ),
                        None => match self.try_parse_ternary(&condition)? {
                            Some(expression) => self.resolve_undecided_expression_content(
                                UndecidedExpressionContent::Expression(expression),
                            ),
                            None => Ok(UndecidedExpressionContent::Condition(condition)),
                        },
                    },
                };
            }
            UndecidedExpressionContent::Name(ref name) => {
                return match self
                    .try_parse_binary_operation(Expression::Object(Box::new(name.clone())))?
                {
                    Expression::Object(name) => {
                        match self.try_parse_comparisson(&Comparable::Object(*name.clone()))? {
                            Some(comparison) => {
                                let condition = self.resolve_comparable(comparison)?;
                                self.resolve_undecided_expression_content(
                                    UndecidedExpressionContent::Condition(condition),
                                )
                            }
                            None => {
                                let condition = Condition::Object(*name);
                                match self.try_parse_binary_condition(&condition)? {
                                    Some(condition) => self.resolve_undecided_expression_content(
                                        UndecidedExpressionContent::Condition(condition),
                                    ),
                                    None => match self.try_parse_ternary(&condition)? {
                                        Some(expression) => self
                                            .resolve_undecided_expression_content(
                                                UndecidedExpressionContent::Expression(expression),
                                            ),
                                        None => Ok(content),
                                    },
                                }
                            }
                        }
                    }
                    expression => self.resolve_undecided_expression_content(
                        UndecidedExpressionContent::Expression(expression),
                    ),
                };
            }
            UndecidedExpressionContent::String(ref string) => {
                return match self.try_parse_comparisson(&Comparable::String(string.clone()))? {
                    Some(comparison) => {
                        let condition = self.resolve_comparable(comparison)?;
                        self.resolve_undecided_expression_content(
                            UndecidedExpressionContent::Condition(condition),
                        )
                    }
                    None => Ok(content),
                };
            }
            UndecidedExpressionContent::Null(ref null) => {
                return match self.try_parse_comparisson(&Comparable::Null(null.clone()))? {
                    Some(comparison) => {
                        let condition = self.resolve_comparable(comparison)?;
                        self.resolve_undecided_expression_content(
                            UndecidedExpressionContent::Condition(condition),
                        )
                    }
                    None => Ok(content),
                };
            }
        };
    }

    fn try_parse_ternary(
        &mut self,
        condition: &Condition,
    ) -> Result<Option<Expression>, SyntaxError> {
        if !self.scanner.take(&'?') {
            return Ok(None);
        }
        let question_mark_location = Location::SingleCharacter {
            char: self.scanner.cursor as u16 - 1,
            line: 0,
        };
        self.scanner.skip_whitespace();
        let left = self.parse_expression()?;
        if !self.scanner.take(&':') {
            return Err(syntax_error! {
                msg: "incomplete ternary; missing `:`. try adding it",
                fix: [SyntaxFix::Insert(
                    Position {
                        line: 0,
                        character: self.scanner.cursor as u32,
                    },
                    " : 0".to_string(),
                )],
            });
        }
        let colon_location = Location::SingleCharacter {
            char: self.scanner.cursor as u16 - 1,
            line: 0,
        };
        self.scanner.skip_whitespace();
        let right = self.parse_expression()?;
        return Ok(Some(Expression::Ternary {
            condition: Box::new(condition.clone()),
            left: Box::new(left),
            right: Box::new(right),
            question_mark_location,
            colon_location,
        }));
    }

    fn parse_bracketed_expression_content(
        &mut self,
    ) -> Result<UndecidedExpressionContent, SyntaxError> {
        if !self.scanner.take(&'(') {
            return Err(syntax_error!("expected opening bracket"));
        }
        let opening_bracket_location = Location::SingleCharacter {
            char: self.scanner.cursor as u16 - 1,
            line: 0,
        };
        self.scanner.skip_whitespace();
        let content = self.parse_undecided_expression_content()?;
        self.scanner.skip_whitespace();
        let content = self.resolve_undecided_expression_content(content)?;
        self.scanner.skip_whitespace();
        if !self.scanner.take(&')') {
            return Err(syntax_error! {
                msg: "unclosed bracket. try adding it",
                fix: [SyntaxFix::Insert(
                    Position {
                        line: 0,
                        character: self.scanner.cursor as u32,
                    },
                    ")".to_string(),
                )],
            });
        }
        let closing_bracket_location = Location::SingleCharacter {
            char: self.scanner.cursor as u16 - 1,
            line: 0,
        };
        return Ok(match content {
            UndecidedExpressionContent::Expression(expression) => {
                UndecidedExpressionContent::Expression(Expression::BracketedExpression {
                    expression: Box::new(expression),
                    opening_bracket_location,
                    closing_bracket_location,
                })
            }
            UndecidedExpressionContent::Condition(condition) => {
                UndecidedExpressionContent::Condition(Condition::BracketedCondition {
                    condition: Box::new(condition),
                    opening_bracket_location,
                    closing_bracket_location,
                })
            }
            UndecidedExpressionContent::Name(_name) => todo!(),
            UndecidedExpressionContent::String(_string) => todo!(),
            UndecidedExpressionContent::Null(_null) => todo!(),
            UndecidedExpressionContent::Function(_function) => todo!(),
        });
    }

    fn parse_condition(&mut self) -> Result<Condition, SyntaxError> {
        let comparable = self.parse_comparable()?;
        return self.resolve_comparable(comparable);
    }

    pub(crate) fn parse_comparable(&mut self) -> Result<Comparable, SyntaxError> {
        return Ok(match self.scanner.peek() {
            Some('\'') => Comparable::String(self.parse_string()?),
            Some('$') => {
                let object = self.parse_interpolation()?;
                self.scanner.skip_whitespace();
                match self.try_parse_binary_operation(Expression::Object(Box::new(object)))? {
                    Expression::Object(interpolation) => Comparable::Object(*interpolation),
                    expression => Comparable::Expression(expression),
                }
            }
            Some('+' | '-') => {
                let expression = self.parse_signed_expression()?;
                Comparable::Expression(self.try_parse_binary_operation(expression)?)
            }
            Some('0'..='9') => {
                let expression = self.parse_number().map(Expression::Number)?;
                self.scanner.skip_whitespace();
                Comparable::Expression(self.try_parse_binary_operation(expression)?)
            }
            Some('(') => self.parse_bracketed_comparable()?,
            Some('!') => Comparable::Condition(self.parse_negated_condition()?),
            Some(_) => {
                let start = self.scanner.cursor as u16;
                let func = self.parse_name_or_global_function()?;
                match func {
                    Object::Name(Word { fragments }) if fragments.len() == 1 => {
                        match &fragments[0] {
                            WordFragment::String(StringLiteral { content, location }) => {
                                match &**content {
                                    "true" => Comparable::Condition(Condition::True {
                                        location: location.clone(),
                                    }),
                                    "false" => Comparable::Condition(Condition::False {
                                        location: location.clone(),
                                    }),
                                    "null" => Comparable::Null(Null {
                                        location: location.clone(),
                                    }),
                                    name => {
                                        let new_string = format!("${{{}}}", name);
                                        return Err(syntax_error! {
                                            fmt: (
                                                "objects in comparissons have to be interpolated. try `{}`",
                                                new_string,
                                            ),
                                            fix: [SyntaxFix::Replace(
                                                location.to_owned(),
                                                new_string,
                                            )],
                                        });
                                    }
                                }
                            }
                            WordFragment::Interpolation(interpolation) => {
                                self.scanner.skip_whitespace();
                                match self.try_parse_binary_operation(Expression::Object(
                                    Box::new(interpolation.clone()),
                                ))? {
                                    Expression::Object(interpolation) => {
                                        Comparable::Object(*interpolation)
                                    }
                                    expression => Comparable::Expression(expression),
                                }
                            }
                        }
                    }
                    Object::Function(function) => Comparable::Function(function),
                    object => {
                        // this should not be possible
                        let new_string = format!("${{{}}}", object);
                        return Err(syntax_error! {
                            fmt: (
                                "objects in comparissons have to be interpolated. try `{}`",
                                new_string,
                            ),
                            fix: [SyntaxFix::Replace(
                                Location::VariableLength {
                                    char: start,
                                    line: 0,
                                    length: self.scanner.cursor as u16 - start,
                                },
                                new_string,
                            )],
                        });
                    }
                }
            }
            None => return Err(syntax_error!("unexpected end")),
        });
    }

    fn resolve_comparable(&mut self, comparable: Comparable) -> Result<Condition, SyntaxError> {
        self.scanner.skip_whitespace();
        return match self.try_parse_comparisson(&comparable)? {
            Some(comparisson) => self.resolve_comparable(comparisson),
            None => {
                let condition = match comparable {
                    Comparable::Condition(condition) => condition,
                    Comparable::Object(interpolation) => Condition::Object(interpolation),
                    Comparable::Function(function) => Condition::Function(function),
                    Comparable::Null(Null { location }) => {
                        let new_string = "false".to_string();
                        return Err(syntax_error! {
                            fmt: (
                                "`null` is not a valid condition. did you mean `{}`?",
                                new_string,
                            ),
                            fix: [SyntaxFix::Replace(location, new_string)],
                        });
                    }
                    // Comparable::Expression(_) => todo!(),
                    // Comparable::String(_) => todo!(),
                    comparable => return Err(syntax_error!("unexpected {}", comparable.r#type())),
                };
                match self.try_parse_binary_condition(&condition)? {
                    Some(condition) => self.resolve_comparable(Comparable::Condition(condition)),
                    None => Ok(condition),
                }
            }
        };
    }

    fn try_parse_comparisson(
        &mut self,
        left: &Comparable,
    ) -> Result<Option<Comparable>, SyntaxError> {
        return Ok(match self.scanner.peek() {
            Some(char @ ('!' | '=' | '>' | '<')) => {
                let char = char.clone();
                self.scanner.pop();
                let equals = self.scanner.take(&'=');
                let operator = match (char, equals) {
                    ('=', true) => ComparissonOperator::Equal,
                    ('!', true) => ComparissonOperator::Unequal,
                    ('>', false) => ComparissonOperator::GreaterThan,
                    ('>', true) => ComparissonOperator::GreaterThanOrEqual,
                    ('<', false) => ComparissonOperator::LessThan,
                    ('<', true) => ComparissonOperator::LessThanOrEqual,
                    (char, _) => {
                        return Err(syntax_error! {
                            fmt: (
                                "`{}` is not a valid comparisson operator. did you mean `{}=`?",
                                char, char,
                            ),
                            fix: [SyntaxFix::Insert(
                                Position {
                                    line: 0,
                                    character: self.scanner.cursor as u32,
                                },
                                "=".to_string(),
                            )],
                        })
                    }
                };
                let operator_location = match equals {
                    true => Location::DoubleCharacter {
                        char: self.scanner.cursor as u16 - 2,
                        line: 0,
                    },
                    false => Location::SingleCharacter {
                        char: self.scanner.cursor as u16 - 1,
                        line: 0,
                    },
                };
                self.scanner.skip_whitespace();
                Some(Comparable::Condition(Condition::Comparisson {
                    left: Box::new(left.clone()),
                    operator,
                    right: Box::new(self.parse_comparable()?),
                    operator_location,
                }))
            }
            _ => None,
        });
    }

    fn try_parse_binary_condition(
        &mut self,
        left: &Condition,
    ) -> Result<Option<Condition>, SyntaxError> {
        return Ok(match self.scanner.peek() {
            Some(char @ ('&' | '|')) => {
                let first = &char.clone();
                let operator = match first == &'&' {
                    true => ConditionOperator::And,
                    false => ConditionOperator::Or,
                };
                let operator_location = Location::DoubleCharacter {
                    char: self.scanner.cursor as u16,
                    line: 0,
                };
                self.scanner.pop();
                if !self.scanner.take(first) {
                    return Err(syntax_error! {
                        fmt: (
                            "`{}` is not a valid condition operator. did you mean `{}{}`?",
                            first, first, first,
                        ),
                        fix: [SyntaxFix::Insert(
                            Position {
                                line: 0,
                                character: self.scanner.cursor as u32,
                            },
                            first.to_string(),
                        )],
                    });
                }
                self.scanner.skip_whitespace();
                Some(Condition::BinaryOperation {
                    left: Box::new(left.clone()),
                    operator,
                    right: Box::new(self.parse_condition()?),
                    operator_location,
                })
            }
            _ => None,
        });
    }

    fn parse_bracketed_comparable(&mut self) -> Result<Comparable, SyntaxError> {
        if !self.scanner.take(&'(') {
            return Err(syntax_error!("expected opening bracket"));
        }
        let opening_bracket_location = Location::SingleCharacter {
            char: self.scanner.cursor as u16 - 1,
            line: 0,
        };
        self.scanner.skip_whitespace();
        let mut comparable = self.parse_comparable()?;
        self.scanner.skip_whitespace();
        if let Some('&' | '|' | '=' | '!' | '<' | '>') = self.scanner.peek() {
            let condition = self.resolve_comparable(comparable)?;
            comparable = Comparable::Condition(condition);
            self.scanner.skip_whitespace();
        }
        return match self.scanner.pop() {
            Some(')') => {
                let closing_bracket_location = Location::SingleCharacter {
                    char: self.scanner.cursor as u16 - 1,
                    line: 0,
                };
                match comparable {
                    Comparable::Expression(expression) => {
                        Ok(Comparable::Expression(Expression::BracketedExpression {
                            expression: Box::new(expression),
                            opening_bracket_location,
                            closing_bracket_location,
                        }))
                    }
                    Comparable::Condition(condition) => {
                        Ok(Comparable::Condition(Condition::BracketedCondition {
                            condition: Box::new(condition),
                            opening_bracket_location,
                            closing_bracket_location,
                        }))
                    }
                    comparable => {
                        return Err(syntax_error!(
                            "unsupported brackets around \"{}\"",
                            comparable.r#type()
                        ))
                    }
                }
            }
            Some(char) => Err(syntax_error!("unexpected char \"{}\"", char)),
            None => Err(syntax_error! {
                msg: "unclosed bracket. try adding it",
                fix: [SyntaxFix::Insert(
                    Position {
                        line: 0,
                        character: self.scanner.cursor as u32,
                    },
                    ")".to_string(),
                )],
            }),
        };
    }

    fn parse_negated_condition(&mut self) -> Result<Condition, SyntaxError> {
        let exclamation_mark_location = Location::SingleCharacter {
            char: self.scanner.cursor as u16,
            line: 0,
        };
        self.scanner.pop();
        self.scanner.skip_whitespace();
        match self.parse_condition()? {
            Condition::NegatedCondition {
                exclamation_mark_location: second,
                ..
            } => Err(syntax_error! {
                msg: "doubly negated conditions are not supported. try removing the negations",
                fix: [SyntaxFix::Delete(Location::VariableLength {
                    char: exclamation_mark_location.char(),
                    line: 0,
                    length: second.char() + second.len() - exclamation_mark_location.char(),
                })],
            }),
            condition => Ok(Condition::NegatedCondition {
                condition: Box::new(condition),
                exclamation_mark_location,
            }),
        }
    }

    fn parse_number(&mut self) -> Result<Number, SyntaxError> {
        let start = self.scanner.cursor as u16;
        let mut result = self.parse_integer()?;
        loop {
            match self.scanner.peek() {
                Some(char @ '.') => {
                    result.push(*char);
                    self.scanner.pop();
                    result.push_str(&self.parse_integer()?);
                }
                Some(char @ ('e' | 'E')) => {
                    result.push(*char);
                    self.scanner.pop();
                    if let Some(char @ ('-' | '+')) = self.scanner.peek() {
                        result.push(*char);
                        self.scanner.pop();
                    }
                    result.push_str(&self.parse_integer()?);
                }
                _ => {
                    return Ok(Number {
                        content: result.into(),
                        location: Location::VariableLength {
                            char: start,
                            line: 0,
                            length: self.scanner.cursor as u16 - start,
                        },
                    })
                }
            }
        }
    }

    fn parse_integer(&mut self) -> Result<String, SyntaxError> {
        let mut result = String::new();
        match self.scanner.pop() {
            Some(char @ '0'..='9') => result.push(*char),
            Some(char) => return Err(syntax_error!("expected number, found \"{}\"", char)),
            None => return Err(syntax_error!("unexpected end")),
        };
        loop {
            match self.scanner.peek() {
                Some(char @ '0'..='9') => {
                    result.push(*char);
                    self.scanner.pop();
                }
                _ => return Ok(result),
            };
        }
    }

    fn parse_signed_expression(&mut self) -> Result<Expression, SyntaxError> {
        let sign = match self.scanner.pop() {
            Some('+') => Sign::Plus,
            Some('-') => Sign::Minus,
            Some(char) => return Err(syntax_error!("unexpected char \"{}\"", char)),
            None => return Err(syntax_error!("unexpected end")),
        };
        let sign_location = Location::SingleCharacter {
            char: self.scanner.cursor as u16 - 1,
            line: 0,
        };
        self.scanner.skip_whitespace();
        match self.parse_expression()? {
            Expression::SignedExpression { .. } => {
                return Err(syntax_error!("duplicate sign"));
            }
            expression => Ok(Expression::SignedExpression {
                expression: Box::new(expression),
                sign,
                sign_location,
            }),
        }
    }

    fn try_parse_binary_operation(&mut self, left: Expression) -> Result<Expression, SyntaxError> {
        return Ok(
            match self.scanner.transform(|c| match c {
                '+' => Some(ExpressionOperator::Addition),
                '-' => Some(ExpressionOperator::Subtraction),
                '/' => Some(ExpressionOperator::Division),
                '*' => Some(ExpressionOperator::Multiplication),
                '^' => Some(ExpressionOperator::Power),
                '%' => Some(ExpressionOperator::Modulo),
                _ => None,
            }) {
                Some(operation) => {
                    let operator_location = Location::SingleCharacter {
                        char: self.scanner.cursor as u16 - 1,
                        line: 0,
                    };
                    self.scanner.skip_whitespace();
                    let expression = self.parse_expression()?;
                    self.resolve_binary_operation_precidence(
                        left,
                        operation,
                        expression,
                        operator_location,
                    )
                }
                None => left,
            },
        );
    }

    fn resolve_binary_operation_precidence(
        &mut self,
        left_expression: Expression,
        left_operation: ExpressionOperator,
        right_expression: Expression,
        left_operation_location: Location,
    ) -> Expression {
        match right_expression {
            Expression::BinaryOperation {
                left,
                operator: right_operation,
                right,
                operator_location: right_operation_location,
            } if left_operation <= right_operation => Expression::BinaryOperation {
                left: Box::new(self.resolve_binary_operation_precidence(
                    left_expression,
                    left_operation,
                    *left,
                    left_operation_location,
                )),
                operator: right_operation,
                right,
                operator_location: right_operation_location,
            },
            _ => Expression::BinaryOperation {
                left: Box::new(left_expression),
                operator: left_operation,
                right: Box::new(right_expression),
                operator_location: left_operation_location,
            },
        }
    }

    fn parse_string(&mut self) -> Result<StringLiteral, SyntaxError> {
        let mut result = String::new();
        let start = self.scanner.cursor as u16;
        if !self.scanner.take(&'\'') {
            return Err(syntax_error!("expected string"));
        }
        loop {
            match self.scanner.peek() {
                Some('\\') => {
                    self.scanner.pop();
                    match self.scanner.pop() {
                        Some(char @ ('b' | 't' | 'n' | 'f' | 'r' | '"' | '\'' | '\\')) => {
                            result.push('\\');
                            result.push(*char);
                        }
                        Some('u') => {
                            // TODO: parse hexadecimal unicode
                            result.push('\\');
                            result.push('u');
                        }
                        Some(char) => {
                            return Err(syntax_error! {
                                fmt: (
                                    "invalid escape sequence `\\{}`. did you mean `\\\\`?",
                                    char,
                                ),
                                fix: [SyntaxFix::Insert(
                                    Position {
                                        line: 0,
                                        character: self.scanner.cursor as u32,
                                    },
                                    "\\".to_string(),
                                )],
                            });
                        }
                        None => {
                            return Err(syntax_error! {
                                msg: "missing qoute. try adding adding it",
                                fix: [SyntaxFix::Insert(
                                    Position {
                                        line: 0,
                                        character: self.scanner.cursor as u32,
                                    },
                                    "\\'".to_string(),
                                )],
                            })
                        }
                    }
                }
                Some('\'') => {
                    self.scanner.pop();
                    return Ok(StringLiteral {
                        content: result.as_str().into(),
                        location: Location::VariableLength {
                            char: start,
                            line: 0,
                            length: result.len() as u16 + 2,
                        },
                    });
                }
                // TODO: evaluate what characters are __actually__ allowed
                Some(char) if char.is_ascii() => {
                    result.push(*char);
                    self.scanner.pop();
                }
                Some(char) => return Err(syntax_error!("invalid character \"{}\"", char)),
                None => {
                    return Err(syntax_error! {
                        msg: "missing qoute. try adding adding it",
                        fix: [SyntaxFix::Insert(
                            Position {
                                line: 0,
                                character: self.scanner.cursor as u32,
                            },
                            "'".to_string(),
                        )],
                    })
                }
            }
        }
    }

    fn parse_interpolation(&mut self) -> Result<Interpolation, SyntaxError> {
        let start = self.scanner.cursor as u16;
        if !self.scanner.take_str(&"${") {
            return Err(syntax_error!("expected interpolation"));
        }
        self.scanner.skip_whitespace();
        let result = self.parse_object()?;
        self.scanner.skip_whitespace();
        return match self.scanner.pop() {
            Some('}') => Ok(Interpolation {
                content: result,
                opening_bracket_location: Location::DoubleCharacter {
                    char: start,
                    line: 0,
                },
                closing_bracket_location: Location::SingleCharacter {
                    char: self.scanner.cursor as u16 - 1,
                    line: 0,
                },
            }),
            Some(char) => Err(syntax_error!("unexpected char \"{}\"", char)),
            None => Err(syntax_error! {
                msg: "unclosed interpolation. try adding the missing bracket",
                fix: [SyntaxFix::Insert(
                    Position {
                        line: 0,
                        character: self.scanner.cursor as u32,
                    },
                    "}".to_string(),
                )],
            }),
        };
    }

    fn parse_interpolated_anchor(&mut self) -> Result<Anchor, SyntaxError> {
        let start = self.scanner.cursor as u16;
        if !self.scanner.take_str(&"!{") {
            return Err(syntax_error! {
                msg: "expected `{` after `!` for an interpolated anchor. try adding it",
                fix: [SyntaxFix::Insert(
                    Position {
                        line: 0,
                        character: self.scanner.cursor as u32 + 1,
                    },
                    "{".to_string(),
                )],
            });
        }
        self.scanner.skip_whitespace();
        let result = self.parse_word_with_additional_chars(&ADDITIONAL_CHARS_IN_ANCHOR)?;
        self.scanner.skip_whitespace();
        return match self.scanner.pop() {
            Some('}') => Ok(Anchor {
                name: result,
                opening_bracket_location: Location::DoubleCharacter {
                    char: start,
                    line: 0,
                },
                closing_bracket_location: Location::SingleCharacter {
                    char: self.scanner.cursor as u16 - 1,
                    line: 0,
                },
            }),
            Some(char) => Err(syntax_error!("unexpected char \"{}\"", char)),
            None => Err(syntax_error! {
                msg: "unclosed anchor interpolation. try adding the missing bracket",
                fix: [SyntaxFix::Insert(
                    Position {
                        line: 0,
                        character: self.scanner.cursor as u32,
                    },
                    "}".to_string(),
                )],
            }),
        };
    }

    fn parse_name_or_global_function(&mut self) -> Result<Object, SyntaxError> {
        let name = self.parse_word()?;
        if let [WordFragment::String(StringLiteral { content, location })] = &name.fragments[..] {
            self.scanner.skip_whitespace();
            if let Some(&'(') = self.scanner.peek() {
                let start = self.scanner.cursor as u16;
                let arguments = self.parse_function_arguments()?;
                return Ok(Object::Function(Function {
                    name: content.clone(),
                    arguments,
                    name_location: location.clone(),
                    opening_bracket_location: Location::SingleCharacter {
                        char: start,
                        line: 0,
                    },
                    closing_bracket_location: Location::SingleCharacter {
                        char: self.scanner.cursor as u16 - 1,
                        line: 0,
                    },
                }));
            }
        }
        return Ok(Object::Name(name));
    }

    fn parse_object_access(&mut self, mut object: Object) -> Result<Object, SyntaxError> {
        loop {
            self.scanner.skip_whitespace();
            match self.scanner.peek() {
                Some('[') => {
                    let start = self.scanner.cursor as u16;
                    self.scanner.pop();
                    self.scanner.skip_whitespace();
                    let expression = self.parse_expression()?;
                    self.scanner.skip_whitespace();
                    if !self.scanner.take(&']') {
                        return Err(syntax_error! {
                            msg: "unclosed array access. try adding the missing bracket",
                            fix: [SyntaxFix::Insert(
                                Position {
                                    line: 0,
                                    character: self.scanner.cursor as u32,
                                },
                                "]".to_string(),
                            )],
                        });
                    }
                    object = Object::ArrayAccess {
                        object: Box::new(object),
                        index: expression,
                        opening_bracket_location: Location::SingleCharacter {
                            char: start,
                            line: 0,
                        },
                        closing_bracket_location: Location::SingleCharacter {
                            char: self.scanner.cursor as u16 - 1,
                            line: 0,
                        },
                    }
                }
                Some('.') => {
                    let dot_location = Location::SingleCharacter {
                        char: self.scanner.cursor as u16,
                        line: 0,
                    };
                    self.scanner.pop();
                    self.scanner.skip_whitespace();
                    let name = self.parse_word()?;
                    self.scanner.skip_whitespace();
                    object = match (&name.fragments[..], self.scanner.peek()) {
                        (
                            [WordFragment::String(StringLiteral { content, location })],
                            Some('('),
                        ) => {
                            let start = self.scanner.cursor as u16;
                            let arguments = self.parse_function_arguments()?;
                            Object::MethodAccess {
                                object: Box::new(object),
                                dot_location,
                                function: Function {
                                    name: content.clone(),
                                    arguments,
                                    name_location: location.clone(),
                                    opening_bracket_location: Location::SingleCharacter {
                                        char: start,
                                        line: 0,
                                    },
                                    closing_bracket_location: Location::SingleCharacter {
                                        char: self.scanner.cursor as u16 - 1,
                                        line: 0,
                                    },
                                },
                            }
                        }
                        _ => Object::FieldAccess {
                            object: Box::new(object),
                            field: name,
                            dot_location,
                        },
                    }
                }
                _ => return Ok(object),
            }
        }
    }

    fn parse_word(&mut self) -> Result<Word, SyntaxError> {
        return self.parse_word_with_additional_chars(&[]);
    }

    fn parse_word_with_additional_chars(&mut self, chars: &[char]) -> Result<Word, SyntaxError> {
        let mut string = String::new();
        let mut fragments = Vec::new();
        let mut start = self.scanner.cursor as u16;
        loop {
            match self.scanner.peek() {
                // TODO: evaluate what characters are __actually__ allowed
                Some(char @ ('a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '*')) => {
                    string.push(*char);
                    self.scanner.pop();
                }
                Some(char) if chars.contains(char) => {
                    string.push(*char);
                    self.scanner.pop();
                }
                Some('$') => {
                    let interpolation = self.parse_interpolation()?;
                    let length = string.len() as u16;
                    if length > 0 {
                        fragments.push(WordFragment::String(StringLiteral {
                            content: string.into(),
                            location: Location::VariableLength {
                                char: start,
                                line: 0,
                                length,
                            },
                        }));
                        string = String::new();
                    }
                    start = self.scanner.cursor as u16;
                    fragments.push(WordFragment::Interpolation(interpolation))
                }
                _ => break,
            }
        }
        let length = string.len() as u16;
        if length > 0 {
            fragments.push(WordFragment::String(StringLiteral {
                content: string.into(),
                location: Location::VariableLength {
                    char: start,
                    line: 0,
                    length,
                },
            }));
        }
        return match fragments.len() > 0 {
            true => Ok(Word { fragments }),
            false => Err(match self.scanner.peek() {
                Some(char) => syntax_error!("unexpected char \"{}\"", char),
                _ => syntax_error!("unexpected end"),
            }),
        };
    }

    fn parse_function_arguments(&mut self) -> Result<Vec<FunctionArgument>, SyntaxError> {
        let mut arguments = Vec::new();
        if !self.scanner.take(&'(') {
            return Err(syntax_error!("expected opening brace"));
        }
        self.scanner.skip_whitespace();
        if self.scanner.take(&')') {
            return Ok(arguments);
        }
        loop {
            let argument = match self.scanner.peek() {
                Some('\'') => self.parse_string().map(Argument::String),
                Some('!') => self.parse_interpolated_anchor().map(Argument::Anchor),
                Some('$') => self.parse_interpolation().map(Argument::Object),
                Some('0'..='9') => self.parse_number().map(Argument::Number),
                Some(char @ ('-' | '+')) => {
                    let sign = match char {
                        '+' => Sign::Plus,
                        _ => Sign::Minus,
                    };
                    let sign_location = Location::SingleCharacter {
                        char: self.scanner.cursor as u16,
                        line: 0,
                    };
                    self.scanner.pop();
                    self.scanner.skip_whitespace();
                    self.parse_number()
                        .map(|number| SignedNumber {
                            sign,
                            number,
                            sign_location,
                        })
                        .map(Argument::SignedNumber)
                }
                Some(_) => match self.parse_name_or_global_function()? {
                    Object::Name(Word { fragments }) if fragments.len() == 1 => {
                        match &fragments[0] {
                            WordFragment::String(StringLiteral { content, location })
                                if &**content == "false" =>
                            {
                                Ok(Argument::False {
                                    location: location.clone(),
                                })
                            }
                            WordFragment::String(StringLiteral { content, location })
                                if &**content == "true" =>
                            {
                                Ok(Argument::True {
                                    location: location.clone(),
                                })
                            }
                            WordFragment::String(StringLiteral { content, location })
                                if &**content == "null" =>
                            {
                                Ok(Argument::Null(Null {
                                    location: location.clone(),
                                }))
                            }
                            WordFragment::String(StringLiteral { content, location }) => {
                                let new_string = format!("${{{}}}", content);
                                Err(syntax_error! {
                                    fmt: (
                                        "objects in arguments have to be interpolated. try `{}`",
                                        new_string,
                                    ),
                                    fix: [SyntaxFix::Replace(location.to_owned(), new_string)],
                                })
                            }
                            WordFragment::Interpolation(interpolation) => {
                                Ok(Argument::Object(interpolation.to_owned()))
                            }
                        }
                    }
                    Object::Function(function) => Ok(Argument::Function(function)),
                    object => {
                        // this should not be possible
                        Err(syntax_error!(
                            "objects in function arguments have to be interpolated. try `${{{}}}`",
                            object
                        ))
                    }
                },
                None => Err(syntax_error! {
                    msg: "unclosed function arguments. try adding the missing bracket",
                    fix: [match arguments
                        .last()
                        .and_then(|arg| arg.comma_location.as_ref())
                    {
                        Some(comma_location) => SyntaxFix::Replace(
                            comma_location.to_owned(),
                            ")".to_string(),
                        ),
                        None => SyntaxFix::Insert(
                            Position {
                                character: self.scanner.cursor as u32,
                                line: 0,
                            },
                            ")".to_string(),
                        ),
                    }],
                }),
            }?;
            self.scanner.skip_whitespace();
            match self.scanner.pop() {
                Some(')') => {
                    arguments.push(FunctionArgument {
                        argument,
                        comma_location: None,
                    });
                    return Ok(arguments);
                }
                Some(',') => {
                    arguments.push(FunctionArgument {
                        argument,
                        comma_location: Some(Location::SingleCharacter {
                            char: self.scanner.cursor as u16 - 1,
                            line: 0,
                        }),
                    });
                    self.scanner.skip_whitespace();
                }
                Some(char) => return Err(syntax_error!("unexpected char \"{}\"", char)),
                None => {
                    return Err(syntax_error! {
                        msg: "unclosed function arguments. try adding the missing bracket",
                        fix: [SyntaxFix::Insert(
                            Position {
                                character: self.scanner.cursor as u32,
                                line: 0,
                            },
                            ")".to_string(),
                        )],
                    })
                }
            };
        }
    }

    fn trailing_characters_error(&self) -> SyntaxError {
        let root_end = self.scanner.subtract_whitespace() as u16 + 1;
        return syntax_error! {
            fmt: ("trailing \"{}\". try removing it", self.scanner.rest()),
            fix: [SyntaxFix::Delete(Location::VariableLength {
                char: root_end,
                line: 0,
                length: self.scanner.characters.len() as u16 - root_end,
            })],
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::spel::ast::{
        Anchor, Argument, Comparable, ComparissonOperator, Condition, ConditionAst,
        ConditionOperator, Expression, ExpressionAst, ExpressionOperator, Function,
        FunctionArgument, Interpolation, Location, Null, Number, Object, ObjectAst, Sign,
        StringLiteral, Word, WordFragment,
    };

    #[test]
    fn test_parse_simple_string_object() {
        assert_eq!(
            parse_object("'test'"),
            ObjectAst {
                root: Object::String(StringLiteral {
                    content: "test".into(),
                    location: Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 6,
                    }
                })
            }
        );
    }

    #[test]
    fn test_parse_string_object_with_whitespace() {
        assert_eq!(
            parse_object("\t'test'   "),
            ObjectAst {
                root: Object::String(StringLiteral {
                    content: "test".into(),
                    location: Location::VariableLength {
                        char: 1,
                        line: 0,
                        length: 6,
                    }
                })
            }
        );
    }

    #[test]
    fn test_parse_escaped_string_object() {
        assert_eq!(
            parse_object("'tes\\\'t'"),
            ObjectAst {
                root: Object::String(StringLiteral {
                    content: "tes\\\'t".into(),
                    location: Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 8,
                    }
                })
            }
        );
    }

    #[test]
    fn test_parse_null() {
        assert_eq!(
            parse_expression("null"),
            ExpressionAst {
                root: Expression::Null(Null {
                    location: Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 4,
                    }
                })
            }
        );
    }

    #[test]
    fn test_parse_interpolated_null() {
        assert_eq!(
            parse_object("null${'notNull'}"),
            ObjectAst {
                root: Object::Name(Word {
                    fragments: vec![
                        WordFragment::String(StringLiteral {
                            content: "null".into(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 4,
                            }
                        }),
                        WordFragment::Interpolation(Interpolation {
                            content: Object::String(StringLiteral {
                                content: "notNull".into(),
                                location: Location::VariableLength {
                                    char: 6,
                                    line: 0,
                                    length: 9,
                                }
                            }),
                            opening_bracket_location: Location::DoubleCharacter {
                                char: 4,
                                line: 0,
                            },
                            closing_bracket_location: Location::SingleCharacter {
                                char: 15,
                                line: 0,
                            },
                        })
                    ],
                })
            }
        );
    }

    #[test]
    fn test_parse_null_function() {
        assert_eq!(
            parse_expression("null()"),
            ExpressionAst {
                root: Expression::Function(Function {
                    name: "null".into(),
                    arguments: vec![],
                    name_location: Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 4,
                    },
                    opening_bracket_location: Location::SingleCharacter { char: 4, line: 0 },
                    closing_bracket_location: Location::SingleCharacter { char: 5, line: 0 }
                })
            }
        );
    }

    #[test]
    fn test_parse_simple_interpolated_anchor() {
        assert_eq!(
            parse_object("!{home}"),
            ObjectAst {
                root: Object::Anchor(Anchor {
                    name: Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "home".into(),
                            location: Location::VariableLength {
                                char: 2,
                                line: 0,
                                length: 4,
                            }
                        })]
                    },
                    opening_bracket_location: Location::DoubleCharacter { char: 0, line: 0 },
                    closing_bracket_location: Location::SingleCharacter { char: 6, line: 0 },
                })
            }
        );
    }

    #[test]
    fn test_parse_nested_interpolated_anchor() {
        assert_eq!(
            parse_object("${!{home}}"),
            ObjectAst {
                root: Object::Name(Word {
                    fragments: vec![WordFragment::Interpolation(Interpolation {
                        content: Object::Anchor(Anchor {
                            name: Word {
                                fragments: vec![WordFragment::String(StringLiteral {
                                    content: "home".into(),
                                    location: Location::VariableLength {
                                        char: 4,
                                        line: 0,
                                        length: 4,
                                    }
                                })],
                            },
                            opening_bracket_location: Location::DoubleCharacter {
                                char: 2,
                                line: 0,
                            },
                            closing_bracket_location: Location::SingleCharacter {
                                char: 8,
                                line: 0,
                            },
                        }),
                        opening_bracket_location: Location::DoubleCharacter { char: 0, line: 0 },
                        closing_bracket_location: Location::SingleCharacter { char: 9, line: 0 }
                    })]
                })
            }
        );
    }

    #[test]
    fn test_parse_interpolated_anchor() {
        assert_eq!(
            parse_object("!{home-${_object}-content}"),
            ObjectAst {
                root: Object::Anchor(Anchor {
                    name: Word {
                        fragments: vec![
                            WordFragment::String(StringLiteral {
                                content: "home-".into(),
                                location: Location::VariableLength {
                                    char: 2,
                                    line: 0,
                                    length: 5,
                                }
                            }),
                            WordFragment::Interpolation(Interpolation {
                                content: Object::Name(Word {
                                    fragments: vec![WordFragment::String(StringLiteral {
                                        content: "_object".into(),
                                        location: Location::VariableLength {
                                            char: 9,
                                            line: 0,
                                            length: 7,
                                        }
                                    })]
                                }),
                                opening_bracket_location: Location::DoubleCharacter {
                                    char: 7,
                                    line: 0,
                                },
                                closing_bracket_location: Location::SingleCharacter {
                                    char: 16,
                                    line: 0,
                                },
                            }),
                            WordFragment::String(StringLiteral {
                                content: "-content".into(),
                                location: Location::VariableLength {
                                    char: 17,
                                    line: 0,
                                    length: 8,
                                }
                            }),
                        ],
                    },
                    opening_bracket_location: Location::DoubleCharacter { char: 0, line: 0 },
                    closing_bracket_location: Location::SingleCharacter { char: 25, line: 0 },
                })
            }
        );
    }

    #[test]
    fn test_parse_string_not_interpolating() {
        assert_eq!(
            parse_object("'hello, ${world}'"),
            ObjectAst {
                root: Object::String(StringLiteral {
                    content: "hello, ${world}".into(),
                    location: Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 17,
                    },
                })
            }
        );
    }

    #[test]
    fn test_parse_simple_global_function() {
        assert_eq!(
            parse_object("flush()"),
            ObjectAst {
                root: Object::Function(Function {
                    name: "flush".into(),
                    name_location: Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 5,
                    },
                    arguments: vec![],
                    opening_bracket_location: Location::SingleCharacter { char: 5, line: 0 },
                    closing_bracket_location: Location::SingleCharacter { char: 6, line: 0 },
                })
            }
        );
    }

    #[test]
    fn test_parse_global_function_with_argument() {
        assert_eq!(
            parse_object("is_string('test')"),
            ObjectAst {
                root: Object::Function(Function {
                    name: "is_string".into(),
                    name_location: Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 9,
                    },
                    arguments: vec![FunctionArgument {
                        argument: Argument::String(StringLiteral {
                            content: "test".into(),
                            location: Location::VariableLength {
                                char: 10,
                                line: 0,
                                length: 6,
                            },
                        }),
                        comma_location: None
                    }],
                    opening_bracket_location: Location::SingleCharacter { char: 9, line: 0 },
                    closing_bracket_location: Location::SingleCharacter { char: 16, line: 0 },
                })
            }
        );
    }

    #[test]
    fn test_parse_global_function_with_excessive_whitespace() {
        assert_eq!(
            parse_object("\tis_string (\t'test'  , 'test2' ) "),
            ObjectAst {
                root: Object::Function(Function {
                    name: "is_string".into(),
                    name_location: Location::VariableLength {
                        char: 1,
                        line: 0,
                        length: 9,
                    },
                    arguments: vec![
                        FunctionArgument {
                            argument: Argument::String(StringLiteral {
                                content: "test".into(),
                                location: Location::VariableLength {
                                    char: 13,
                                    line: 0,
                                    length: 6,
                                },
                            }),
                            comma_location: Some(Location::SingleCharacter { char: 21, line: 0 })
                        },
                        FunctionArgument {
                            argument: Argument::String(StringLiteral {
                                content: "test2".into(),
                                location: Location::VariableLength {
                                    char: 23,
                                    line: 0,
                                    length: 7,
                                },
                            }),
                            comma_location: None
                        }
                    ],
                    opening_bracket_location: Location::SingleCharacter { char: 11, line: 0 },
                    closing_bracket_location: Location::SingleCharacter { char: 31, line: 0 },
                })
            }
        );
    }

    #[test]
    fn test_parse_nested_global_function() {
        assert_eq!(
            parse_object("is_string(concat('hello', 'world'))"),
            ObjectAst {
                root: Object::Function(Function {
                    name: "is_string".into(),
                    name_location: Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 9,
                    },
                    arguments: vec![FunctionArgument {
                        argument: Argument::Function(Function {
                            name: "concat".into(),
                            name_location: Location::VariableLength {
                                char: 10,
                                line: 0,
                                length: 6,
                            },
                            arguments: vec![
                                FunctionArgument {
                                    argument: Argument::String(StringLiteral {
                                        content: "hello".into(),
                                        location: Location::VariableLength {
                                            char: 17,
                                            line: 0,
                                            length: 7,
                                        },
                                    }),
                                    comma_location: Some(Location::SingleCharacter {
                                        char: 24,
                                        line: 0
                                    })
                                },
                                FunctionArgument {
                                    argument: Argument::String(StringLiteral {
                                        content: "world".into(),
                                        location: Location::VariableLength {
                                            char: 26,
                                            line: 0,
                                            length: 7,
                                        },
                                    }),
                                    comma_location: None,
                                }
                            ],
                            opening_bracket_location: Location::SingleCharacter {
                                char: 16,
                                line: 0
                            },
                            closing_bracket_location: Location::SingleCharacter {
                                char: 33,
                                line: 0
                            },
                        }),
                        comma_location: None
                    }],
                    opening_bracket_location: Location::SingleCharacter { char: 9, line: 0 },
                    closing_bracket_location: Location::SingleCharacter { char: 34, line: 0 },
                })
            }
        );
    }

    #[test]
    fn test_parse_simple_field_access() {
        assert_eq!(
            parse_object("_string.length"),
            ObjectAst {
                root: Object::FieldAccess {
                    object: Box::new(Object::Name(Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "_string".into(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 7,
                            },
                        })]
                    })),
                    field: Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "length".into(),
                            location: Location::VariableLength {
                                char: 8,
                                line: 0,
                                length: 6,
                            },
                        })]
                    },
                    dot_location: Location::SingleCharacter { char: 7, line: 0 },
                }
            }
        );
    }

    #[test]
    fn test_parse_invalid_field_access() {
        let string = "_string..length";
        if let Ok(result) = (&mut super::Parser::new(&string)).parse_object_ast() {
            panic!(
                "successfully parsed invalid string \"{}\" into {:?}",
                string, result
            );
        }
    }

    #[test]
    fn test_parse_simple_method_access() {
        assert_eq!(
            parse_object("_string.length()"),
            ObjectAst {
                root: Object::MethodAccess {
                    object: Box::new(Object::Name(Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "_string".into(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 7,
                            },
                        })]
                    })),
                    function: Function {
                        name: "length".into(),
                        name_location: Location::VariableLength {
                            char: 8,
                            line: 0,
                            length: 6,
                        },
                        arguments: vec![],
                        opening_bracket_location: Location::SingleCharacter { char: 14, line: 0 },
                        closing_bracket_location: Location::SingleCharacter { char: 15, line: 0 },
                    },
                    dot_location: Location::SingleCharacter { char: 7, line: 0 },
                }
            }
        );
    }

    #[test]
    fn test_parse_simple_array_access() {
        assert_eq!(
            parse_object("_strings[0]"),
            ObjectAst {
                root: Object::ArrayAccess {
                    object: Box::new(Object::Name(Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "_strings".into(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 8,
                            },
                        })]
                    })),
                    index: Expression::Number(Number {
                        content: "0".into(),
                        location: Location::VariableLength {
                            char: 9,
                            line: 0,
                            length: 1,
                        }
                    }),
                    opening_bracket_location: Location::SingleCharacter { char: 8, line: 0 },
                    closing_bracket_location: Location::SingleCharacter { char: 10, line: 0 }
                }
            }
        );
    }

    #[test]
    fn test_parse_signed_array_access() {
        assert_eq!(
            parse_object("_strings[+1]"),
            ObjectAst {
                root: Object::ArrayAccess {
                    object: Box::new(Object::Name(Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "_strings".into(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 8,
                            },
                        })]
                    })),
                    index: Expression::SignedExpression {
                        expression: Box::new(Expression::Number(Number {
                            content: "1".into(),
                            location: Location::VariableLength {
                                char: 10,
                                line: 0,
                                length: 1,
                            }
                        })),
                        sign: Sign::Plus,
                        sign_location: Location::SingleCharacter { char: 9, line: 0 },
                    },
                    opening_bracket_location: Location::SingleCharacter { char: 8, line: 0 },
                    closing_bracket_location: Location::SingleCharacter { char: 11, line: 0 }
                }
            }
        );
    }

    #[test]
    fn test_parse_simple_number() {
        assert_eq!(
            parse_expression("123"),
            ExpressionAst {
                root: Expression::Number(Number {
                    content: "123".into(),
                    location: Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 3,
                    }
                })
            }
        );
    }

    #[test]
    fn test_parse_scientific_number() {
        assert_eq!(
            parse_expression("-13.5e-2"),
            ExpressionAst {
                root: Expression::SignedExpression {
                    expression: Box::new(Expression::Number(Number {
                        content: "13.5e-2".into(),
                        location: Location::VariableLength {
                            char: 1,
                            line: 0,
                            length: 7,
                        }
                    })),
                    sign: Sign::Minus,
                    sign_location: Location::SingleCharacter { char: 0, line: 0 },
                }
            }
        );
    }

    #[test]
    fn test_parse_bracketed_number() {
        assert_eq!(
            parse_expression(" -( -6E+2 ) "),
            ExpressionAst {
                root: Expression::SignedExpression {
                    expression: Box::new(Expression::BracketedExpression {
                        expression: Box::new(Expression::SignedExpression {
                            expression: Box::new(Expression::Number(Number {
                                content: "6E+2".into(),
                                location: Location::VariableLength {
                                    char: 5,
                                    line: 0,
                                    length: 4,
                                }
                            })),
                            sign: Sign::Minus,
                            sign_location: Location::SingleCharacter { char: 4, line: 0 },
                        }),
                        opening_bracket_location: Location::SingleCharacter { char: 2, line: 0 },
                        closing_bracket_location: Location::SingleCharacter { char: 10, line: 0 }
                    }),
                    sign: Sign::Minus,
                    sign_location: Location::SingleCharacter { char: 1, line: 0 },
                }
            }
        );
    }

    #[test]
    fn test_parse_simple_addition() {
        assert_eq!(
            parse_expression("6 + 9"),
            ExpressionAst {
                root: Expression::BinaryOperation {
                    left: Box::new(Expression::Number(Number {
                        content: "6".into(),
                        location: Location::VariableLength {
                            char: 0,
                            line: 0,
                            length: 1,
                        }
                    })),
                    operator: ExpressionOperator::Addition,
                    right: Box::new(Expression::Number(Number {
                        content: "9".into(),
                        location: Location::VariableLength {
                            char: 4,
                            line: 0,
                            length: 1,
                        }
                    })),
                    operator_location: Location::SingleCharacter { char: 2, line: 0 }
                }
            }
        );
    }

    #[test]
    fn test_parse_incomplete_addition() {
        let string = "12 +";
        (&mut super::Parser::new(&string))
            .parse_expression_ast()
            .expect_err(&format!(
                "successfully parsed invalid string \"{}\"",
                string
            ));
    }

    #[test]
    fn test_parse_multiple_additions() {
        assert_eq!(
            parse_expression("1 + 2 + -3"),
            ExpressionAst {
                root: Expression::BinaryOperation {
                    left: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::Number(Number {
                            content: "1".into(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 1,
                            }
                        })),
                        operator: ExpressionOperator::Addition,
                        right: Box::new(Expression::Number(Number {
                            content: "2".into(),
                            location: Location::VariableLength {
                                char: 4,
                                line: 0,
                                length: 1,
                            }
                        })),
                        operator_location: Location::SingleCharacter { char: 2, line: 0 }
                    }),
                    operator: ExpressionOperator::Addition,
                    right: Box::new(Expression::SignedExpression {
                        expression: Box::new(Expression::Number(Number {
                            content: "3".into(),
                            location: Location::VariableLength {
                                char: 9,
                                line: 0,
                                length: 1,
                            }
                        })),
                        sign: Sign::Minus,
                        sign_location: Location::SingleCharacter { char: 8, line: 0 },
                    }),
                    operator_location: Location::SingleCharacter { char: 6, line: 0 }
                }
            }
        );
    }

    #[test]
    fn test_addition_has_priority_over_division() {
        assert_eq!(
            parse_expression("6 + 10 / 2"),
            ExpressionAst {
                root: Expression::BinaryOperation {
                    left: Box::new(Expression::Number(Number {
                        content: "6".into(),
                        location: Location::VariableLength {
                            char: 0,
                            line: 0,
                            length: 1,
                        }
                    })),
                    operator: ExpressionOperator::Addition,
                    right: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::Number(Number {
                            content: "10".into(),
                            location: Location::VariableLength {
                                char: 4,
                                line: 0,
                                length: 2,
                            }
                        })),
                        operator: ExpressionOperator::Division,
                        right: Box::new(Expression::Number(Number {
                            content: "2".into(),
                            location: Location::VariableLength {
                                char: 9,
                                line: 0,
                                length: 1,
                            }
                        })),
                        operator_location: Location::SingleCharacter { char: 7, line: 0 }
                    }),
                    operator_location: Location::SingleCharacter { char: 2, line: 0 }
                }
            }
        );
    }

    #[test]
    fn test_binary_operation_precidences() {
        assert_eq!(
            parse_expression("1 + 2 / 3 * 4 ^ 5 % 6"),
            ExpressionAst {
                root: Expression::BinaryOperation {
                    left: Box::new(Expression::Number(Number {
                        content: "1".into(),
                        location: Location::VariableLength {
                            char: 0,
                            line: 0,
                            length: 1,
                        }
                    })),
                    operator: ExpressionOperator::Addition,
                    right: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::BinaryOperation {
                            left: Box::new(Expression::BinaryOperation {
                                left: Box::new(Expression::Number(Number {
                                    content: "2".into(),
                                    location: Location::VariableLength {
                                        char: 4,
                                        line: 0,
                                        length: 1,
                                    }
                                })),
                                operator: ExpressionOperator::Division,
                                right: Box::new(Expression::Number(Number {
                                    content: "3".into(),
                                    location: Location::VariableLength {
                                        char: 8,
                                        line: 0,
                                        length: 1,
                                    }
                                })),
                                operator_location: Location::SingleCharacter { char: 6, line: 0 }
                            }),
                            operator: ExpressionOperator::Multiplication,
                            right: Box::new(Expression::BinaryOperation {
                                left: Box::new(Expression::Number(Number {
                                    content: "4".into(),
                                    location: Location::VariableLength {
                                        char: 12,
                                        line: 0,
                                        length: 1,
                                    }
                                })),
                                operator: ExpressionOperator::Power,
                                right: Box::new(Expression::Number(Number {
                                    content: "5".into(),
                                    location: Location::VariableLength {
                                        char: 16,
                                        line: 0,
                                        length: 1,
                                    }
                                })),
                                operator_location: Location::SingleCharacter { char: 14, line: 0 }
                            }),
                            operator_location: Location::SingleCharacter { char: 10, line: 0 }
                        }),
                        operator: ExpressionOperator::Modulo,
                        right: Box::new(Expression::Number(Number {
                            content: "6".into(),
                            location: Location::VariableLength {
                                char: 20,
                                line: 0,
                                length: 1,
                            }
                        })),
                        operator_location: Location::SingleCharacter { char: 18, line: 0 }
                    }),
                    operator_location: Location::SingleCharacter { char: 2, line: 0 }
                }
            }
        );
    }

    #[test]
    fn test_parse_simple_ternary() {
        assert_eq!(
            parse_expression("true ? 1 : 2"),
            ExpressionAst {
                root: Expression::Ternary {
                    condition: Box::new(Condition::True {
                        location: Location::VariableLength {
                            char: 0,
                            line: 0,
                            length: 4,
                        }
                    }),
                    left: Box::new(Expression::Number(Number {
                        content: "1".into(),
                        location: Location::VariableLength {
                            char: 7,
                            line: 0,
                            length: 1
                        }
                    })),
                    right: Box::new(Expression::Number(Number {
                        content: "2".into(),
                        location: Location::VariableLength {
                            char: 11,
                            line: 0,
                            length: 1
                        }
                    })),
                    question_mark_location: Location::SingleCharacter { char: 5, line: 0 },
                    colon_location: Location::SingleCharacter { char: 9, line: 0 },
                },
            }
        );
    }

    #[test]
    fn test_parse_simple_condition() {
        assert_eq!(
            parse_condition("true"),
            ConditionAst {
                root: Condition::True {
                    location: Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 4
                    }
                }
            }
        );
    }

    #[test]
    fn test_parse_condition_with_whitespace() {
        assert_eq!(
            parse_condition("\t\tfalse "),
            ConditionAst {
                root: Condition::False {
                    location: Location::VariableLength {
                        char: 2,
                        line: 0,
                        length: 5
                    }
                }
            }
        );
    }

    #[test]
    fn test_parse_simple_or_condition() {
        assert_eq!(
            parse_condition("false || true"),
            ConditionAst {
                root: Condition::BinaryOperation {
                    left: Box::new(Condition::False {
                        location: Location::VariableLength {
                            char: 0,
                            line: 0,
                            length: 5
                        }
                    }),
                    operator: ConditionOperator::Or,
                    right: Box::new(Condition::True {
                        location: Location::VariableLength {
                            char: 9,
                            line: 0,
                            length: 4
                        }
                    }),
                    operator_location: Location::DoubleCharacter { char: 6, line: 0 }
                }
            }
        );
    }

    #[test]
    fn test_parse_nested_conditions() {
        assert_eq!(
            parse_condition("false || true && false"),
            ConditionAst {
                root: Condition::BinaryOperation {
                    left: Box::new(Condition::False {
                        location: Location::VariableLength {
                            char: 0,
                            line: 0,
                            length: 5
                        }
                    }),
                    operator: ConditionOperator::Or,
                    right: Box::new(Condition::BinaryOperation {
                        left: Box::new(Condition::True {
                            location: Location::VariableLength {
                                char: 9,
                                line: 0,
                                length: 4
                            }
                        }),
                        operator: ConditionOperator::And,
                        right: Box::new(Condition::False {
                            location: Location::VariableLength {
                                char: 17,
                                line: 0,
                                length: 5
                            }
                        }),
                        operator_location: Location::DoubleCharacter { char: 14, line: 0 }
                    }),
                    operator_location: Location::DoubleCharacter { char: 6, line: 0 }
                }
            }
        );
    }

    #[test]
    fn test_parse_simple_bracketed_condition() {
        assert_eq!(
            parse_condition("(false || true)"),
            ConditionAst {
                root: Condition::BracketedCondition {
                    condition: Box::new(Condition::BinaryOperation {
                        left: Box::new(Condition::False {
                            location: Location::VariableLength {
                                char: 1,
                                line: 0,
                                length: 5
                            }
                        }),
                        operator: ConditionOperator::Or,
                        right: Box::new(Condition::True {
                            location: Location::VariableLength {
                                char: 10,
                                line: 0,
                                length: 4
                            }
                        }),
                        operator_location: Location::DoubleCharacter { char: 7, line: 0 }
                    }),
                    opening_bracket_location: Location::SingleCharacter { char: 0, line: 0 },
                    closing_bracket_location: Location::SingleCharacter { char: 14, line: 0 },
                }
            }
        );
    }

    #[test]
    fn test_parse_object_comparisson() {
        assert_eq!(
            parse_condition("${test} == true"),
            ConditionAst {
                root: Condition::Comparisson {
                    left: Box::new(Comparable::Object(Interpolation {
                        opening_bracket_location: Location::DoubleCharacter { char: 0, line: 0 },
                        closing_bracket_location: Location::SingleCharacter { char: 6, line: 0 },
                        content: Object::Name(Word {
                            fragments: vec![WordFragment::String(StringLiteral {
                                content: "test".into(),
                                location: Location::VariableLength {
                                    char: 2,
                                    line: 0,
                                    length: 4
                                }
                            })]
                        })
                    })),
                    operator: ComparissonOperator::Equal,
                    right: Box::new(Comparable::Condition(Condition::True {
                        location: Location::VariableLength {
                            char: 11,
                            line: 0,
                            length: 4
                        }
                    })),
                    operator_location: Location::DoubleCharacter { char: 8, line: 0 }
                }
            }
        );
    }

    #[test]
    fn test_parse_function_condition() {
        assert_eq!(
            parse_condition("isNull(${_test})"),
            ConditionAst {
                root: Condition::Function(Function {
                    name: "isNull".into(),
                    name_location: Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 6
                    },
                    arguments: vec![FunctionArgument {
                        argument: Argument::Object(Interpolation {
                            content: Object::Name(Word {
                                fragments: vec![WordFragment::String(StringLiteral {
                                    content: "_test".into(),
                                    location: Location::VariableLength {
                                        char: 9,
                                        line: 0,
                                        length: 5
                                    }
                                })]
                            }),
                            opening_bracket_location: Location::DoubleCharacter {
                                char: 7,
                                line: 0
                            },
                            closing_bracket_location: Location::SingleCharacter {
                                char: 14,
                                line: 0
                            },
                        }),
                        comma_location: None
                    }],
                    opening_bracket_location: Location::SingleCharacter { char: 6, line: 0 },
                    closing_bracket_location: Location::SingleCharacter { char: 15, line: 0 }
                })
            }
        );
    }

    #[test]
    fn test_parse_expression_comparisson() {
        assert_eq!(
            parse_condition("3 >= 4"),
            ConditionAst {
                root: Condition::Comparisson {
                    left: Box::new(Comparable::Expression(Expression::Number(Number {
                        content: "3".into(),
                        location: Location::VariableLength {
                            char: 0,
                            line: 0,
                            length: 1
                        }
                    }))),
                    operator: ComparissonOperator::GreaterThanOrEqual,
                    right: Box::new(Comparable::Expression(Expression::Number(Number {
                        content: "4".into(),
                        location: Location::VariableLength {
                            char: 5,
                            line: 0,
                            length: 1
                        }
                    }))),
                    operator_location: Location::DoubleCharacter { char: 2, line: 0 }
                }
            }
        );
    }

    #[test]
    fn test_parse_binary_condition_comparisson() {
        assert_eq!(
            parse_condition("3 != 4 && true"),
            ConditionAst {
                root: Condition::BinaryOperation {
                    left: Box::new(Condition::Comparisson {
                        left: Box::new(Comparable::Expression(Expression::Number(Number {
                            content: "3".into(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 1
                            }
                        }))),
                        operator: ComparissonOperator::Unequal,
                        right: Box::new(Comparable::Expression(Expression::Number(Number {
                            content: "4".into(),
                            location: Location::VariableLength {
                                char: 5,
                                line: 0,
                                length: 1
                            }
                        }))),
                        operator_location: Location::DoubleCharacter { char: 2, line: 0 }
                    }),
                    operator: ConditionOperator::And,
                    right: Box::new(Condition::True {
                        location: Location::VariableLength {
                            char: 10,
                            line: 0,
                            length: 4,
                        }
                    }),
                    operator_location: Location::DoubleCharacter { char: 7, line: 0 }
                }
            }
        );
    }

    fn parse_object(string: &str) -> ObjectAst {
        return (&mut super::Parser::new(&string))
            .parse_object_ast()
            .expect(&format!("error parsing object \"{}\"", string));
    }

    fn parse_expression(string: &str) -> ExpressionAst {
        return (&mut super::Parser::new(&string))
            .parse_expression_ast()
            .expect(&format!("error parsing expression \"{}\"", string));
    }

    fn parse_condition(string: &str) -> ConditionAst {
        return (&mut super::Parser::new(&string))
            .parse_condition_ast()
            .expect(&format!("error parsing condition \"{}\"", string));
    }
}
