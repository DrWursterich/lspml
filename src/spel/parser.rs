use super::{
    ast::{self, ConditionAst, ExpressionAst, Location},
    Scanner,
};
use crate::spel::ast::ObjectAst;
use anyhow::Result;

pub(crate) struct Parser {
    scanner: Scanner,
}

impl Parser {
    pub(crate) fn new(string: &str) -> Self {
        return Self {
            scanner: Scanner::new(string),
        };
    }

    pub(crate) fn parse_object_ast(&mut self) -> Result<ObjectAst> {
        self.scanner.skip_whitespace();
        if self.scanner.is_done() {
            return Err(anyhow::anyhow!("string is empty"));
        }
        let root = self.parse_object()?;
        self.scanner.skip_whitespace();
        return match self.scanner.is_done() {
            true => Ok(ObjectAst::new(root)),
            false => Err(anyhow::anyhow!(
                "trailing character \"{}\"",
                self.scanner.peek().unwrap()
            )),
        };
    }

    pub(crate) fn parse_expression_ast(&mut self) -> Result<ExpressionAst> {
        self.scanner.skip_whitespace();
        if self.scanner.is_done() {
            return Err(anyhow::anyhow!("string is empty"));
        }
        let root = self.parse_expression()?;
        self.scanner.skip_whitespace();
        return match self.scanner.is_done() {
            true => Ok(ExpressionAst::new(root)),
            false => Err(anyhow::anyhow!(
                "trailing character \"{}\"",
                self.scanner.peek().unwrap()
            )),
        };
    }

    pub(crate) fn parse_condition_ast(&mut self) -> Result<ConditionAst> {
        self.scanner.skip_whitespace();
        if self.scanner.is_done() {
            return Err(anyhow::anyhow!("string is empty"));
        }
        let root = self.parse_condition()?;
        self.scanner.skip_whitespace();
        return match self.scanner.is_done() {
            true => Ok(ConditionAst::new(root)),
            false => Err(anyhow::anyhow!(
                "trailing character \"{}\"",
                self.scanner.peek().unwrap()
            )),
        };
    }

    pub(crate) fn parse_text(&mut self) -> Result<ast::Word> {
        let mut string = String::new();
        let mut fragments = Vec::new();
        let mut start = self.scanner.cursor as u16;
        loop {
            match self.scanner.peek() {
                Some('$') => {
                    let interpolation = self.parse_interpolation()?;
                    let length = string.len() as u16;
                    if length > 0 {
                        fragments.push(ast::WordFragment::String(ast::StringLiteral {
                            content: string,
                            location: Location::VariableLength {
                                char: start,
                                line: 0,
                                length,
                            },
                        }));
                        string = String::new();
                        start = self.scanner.cursor as u16;
                    }
                    fragments.push(ast::WordFragment::Interpolation(interpolation))
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
            fragments.push(ast::WordFragment::String(ast::StringLiteral {
                content: string,
                location: Location::VariableLength {
                    char: start,
                    line: 0,
                    length,
                },
            }));
        }
        return Ok(ast::Word { fragments });
    }

    pub(crate) fn parse_uri(&mut self) -> Result<ast::Uri> {
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
                    fragments.push(ast::UriFragment {
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
                    return Ok(ast::Uri::Literal(ast::UriLiteral {
                        fragments,
                        file_extension: Some(ast::UriFileExtension {
                            content,
                            dot_location,
                        }),
                    }));
                }
                Some('$') if fragments.len() == 0 => {
                    let interpolation = self.parse_interpolation()?;
                    self.scanner.skip_whitespace();
                    return match self.scanner.is_done() {
                        true => Ok(ast::Uri::Object(interpolation)),
                        false => Err(anyhow::anyhow!(
                            "trailing character \"{}\"",
                            self.scanner.peek().unwrap()
                        )),
                    };
                }
                Some(char) => return Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
                None => {
                    return Ok(ast::Uri::Literal(ast::UriLiteral {
                        fragments,
                        file_extension: None,
                    }))
                }
            }
        }
    }

    pub(crate) fn parse_regex(&mut self) -> Result<ast::Regex> {
        if self.scanner.is_done() {
            return Err(anyhow::anyhow!("string is empty"));
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
                    return Ok(ast::Regex {
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

    pub(crate) fn parse_identifier(&mut self) -> Result<ast::Identifier> {
        self.scanner.skip_whitespace();
        if self.scanner.is_done() {
            return Err(anyhow::anyhow!("string is empty"));
        }
        let name = self.parse_word()?;
        self.scanner.skip_whitespace();
        let mut result = ast::Identifier::Name(name);
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
                    result = ast::Identifier::FieldAccess {
                        identifier: Box::new(result),
                        field: name,
                        dot_location,
                    };
                }
                Some(char) => return Err(anyhow::anyhow!("trailing character \"{}\"", char)),
                None => return Ok(result),
            }
        }
    }

    fn parse_object(&mut self) -> Result<ast::Object> {
        return match self.scanner.peek() {
            Some('\'') => self.parse_string().map(|s| ast::Object::String(s)),
            Some('!') => self.parse_interpolated_anchor().map(ast::Object::Anchor),
            Some('$' | 'a'..='z' | 'A'..='Z' | '_') => self.parse_name_or_global_function(),
            Some(char) => Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
            None => Err(anyhow::anyhow!("unexpected end")),
        };
    }

    fn parse_expression(&mut self) -> Result<ast::Expression> {
        let result = self.parse_undecided_expression_content()?;
        return self.resolve_expression_content(result);
    }

    fn parse_undecided_expression_content(&mut self) -> Result<ast::UndecidedExpressionContent> {
        return match self.scanner.peek() {
            Some('\'') => self
                .parse_string()
                .map(ast::UndecidedExpressionContent::String),
            Some('$') => self
                .parse_interpolation()
                .map(ast::UndecidedExpressionContent::Name),
            Some('(') => self.parse_bracketed_expression_content(),
            Some('+' | '-') => self
                .parse_signed_expression()
                .map(ast::UndecidedExpressionContent::Expression),
            Some('0'..='9') => self
                .parse_number()
                .map(ast::Expression::Number)
                .map(ast::UndecidedExpressionContent::Expression),
            Some('n') => self
                .parse_null_literal()
                .map(ast::UndecidedExpressionContent::Null),
            Some('f') => self
                .parse_false_literal()
                .map(ast::UndecidedExpressionContent::Condition),
            Some('t') => self
                .parse_true_literal()
                .map(ast::UndecidedExpressionContent::Condition),
            Some('!') => self
                .parse_negated_condition()
                .map(ast::UndecidedExpressionContent::Condition),
            Some(char) => return Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
            None => return Err(anyhow::anyhow!("unexpected end")),
        };
    }

    fn resolve_expression_content(
        &mut self,
        content: ast::UndecidedExpressionContent,
    ) -> Result<ast::Expression> {
        self.scanner.skip_whitespace();
        return match self.resolve_undecided_expression_content(content)? {
            ast::UndecidedExpressionContent::Expression(expression) => Ok(expression),
            ast::UndecidedExpressionContent::Name(name) => {
                Ok(ast::Expression::Object(Box::new(name)))
            }
            content => Err(anyhow::anyhow!("unexpected {}", content.r#type())),
        };
    }

    fn resolve_undecided_expression_content(
        &mut self,
        content: ast::UndecidedExpressionContent,
    ) -> Result<ast::UndecidedExpressionContent> {
        // TODO: waaaaaay to many .clone()s!
        match content {
            ast::UndecidedExpressionContent::Expression(expression) => {
                let expression = self.try_parse_binary_operation(expression)?;
                return match self
                    .try_parse_comparisson(&ast::Comparable::Expression(expression.clone()))?
                {
                    Some(comparison) => {
                        let condition = self.resolve_comparable(comparison)?;
                        self.resolve_undecided_expression_content(
                            ast::UndecidedExpressionContent::Condition(condition),
                        )
                    }
                    None => Ok(ast::UndecidedExpressionContent::Expression(expression)),
                };
            }
            ast::UndecidedExpressionContent::Condition(condition) => {
                return match self
                    .try_parse_comparisson(&ast::Comparable::Condition(condition.clone()))?
                {
                    Some(comparison) => {
                        let condition = self.resolve_comparable(comparison)?;
                        self.resolve_undecided_expression_content(
                            ast::UndecidedExpressionContent::Condition(condition),
                        )
                    }
                    None => match self.try_parse_binary_condition(&condition)? {
                        Some(condition) => self.resolve_undecided_expression_content(
                            ast::UndecidedExpressionContent::Condition(condition),
                        ),
                        None => match self.try_parse_ternary(&condition)? {
                            Some(expression) => self.resolve_undecided_expression_content(
                                ast::UndecidedExpressionContent::Expression(expression),
                            ),
                            None => Ok(ast::UndecidedExpressionContent::Condition(condition)),
                        },
                    },
                };
            }
            ast::UndecidedExpressionContent::Name(ref name) => {
                return match self
                    .try_parse_binary_operation(ast::Expression::Object(Box::new(name.clone())))?
                {
                    ast::Expression::Object(name) => {
                        match self.try_parse_comparisson(&ast::Comparable::Object(*name.clone()))? {
                            Some(comparison) => {
                                let condition = self.resolve_comparable(comparison)?;
                                self.resolve_undecided_expression_content(
                                    ast::UndecidedExpressionContent::Condition(condition),
                                )
                            }
                            None => {
                                let condition = ast::Condition::Object(*name);
                                match self.try_parse_binary_condition(&condition)? {
                                    Some(condition) => self.resolve_undecided_expression_content(
                                        ast::UndecidedExpressionContent::Condition(condition),
                                    ),
                                    None => match self.try_parse_ternary(&condition)? {
                                        Some(expression) => self
                                            .resolve_undecided_expression_content(
                                                ast::UndecidedExpressionContent::Expression(
                                                    expression,
                                                ),
                                            ),
                                        None => Ok(content),
                                    },
                                }
                            }
                        }
                    }
                    expression => self.resolve_undecided_expression_content(
                        ast::UndecidedExpressionContent::Expression(expression),
                    ),
                };
            }
            ast::UndecidedExpressionContent::String(ref string) => {
                return match self.try_parse_comparisson(&ast::Comparable::String(string.clone()))? {
                    Some(comparison) => {
                        let condition = self.resolve_comparable(comparison)?;
                        self.resolve_undecided_expression_content(
                            ast::UndecidedExpressionContent::Condition(condition),
                        )
                    }
                    None => Ok(content),
                };
            }
            ast::UndecidedExpressionContent::Null(ref null) => {
                return match self.try_parse_comparisson(&ast::Comparable::Null(null.clone()))? {
                    Some(comparison) => {
                        let condition = self.resolve_comparable(comparison)?;
                        self.resolve_undecided_expression_content(
                            ast::UndecidedExpressionContent::Condition(condition),
                        )
                    }
                    None => Ok(content),
                };
            }
        };
    }

    fn try_parse_ternary(&mut self, condition: &ast::Condition) -> Result<Option<ast::Expression>> {
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
            return Err(anyhow::anyhow!("missing \":\" in ternary"));
        }
        let colon_location = Location::SingleCharacter {
            char: self.scanner.cursor as u16 - 1,
            line: 0,
        };
        self.scanner.skip_whitespace();
        let right = self.parse_expression()?;
        return Ok(Some(ast::Expression::Ternary {
            condition: Box::new(condition.clone()),
            left: Box::new(left),
            right: Box::new(right),
            question_mark_location,
            colon_location,
        }));
    }

    fn parse_bracketed_expression_content(&mut self) -> Result<ast::UndecidedExpressionContent> {
        if !self.scanner.take(&'(') {
            return Err(anyhow::anyhow!("expected opening bracket"));
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
            return Err(anyhow::anyhow!("unclosed bracket"));
        }
        let closing_bracket_location = Location::SingleCharacter {
            char: self.scanner.cursor as u16 - 1,
            line: 0,
        };
        return Ok(match content {
            ast::UndecidedExpressionContent::Expression(expression) => {
                ast::UndecidedExpressionContent::Expression(ast::Expression::BracketedExpression {
                    expression: Box::new(expression),
                    opening_bracket_location,
                    closing_bracket_location,
                })
            }
            ast::UndecidedExpressionContent::Condition(condition) => {
                ast::UndecidedExpressionContent::Condition(ast::Condition::BracketedCondition {
                    condition: Box::new(condition),
                    opening_bracket_location,
                    closing_bracket_location,
                })
            }
            ast::UndecidedExpressionContent::Name(_name) => todo!(),
            ast::UndecidedExpressionContent::String(_string) => todo!(),
            ast::UndecidedExpressionContent::Null(_null) => todo!(),
        });
    }

    fn parse_condition(&mut self) -> Result<ast::Condition> {
        let comparable = self.parse_comparable()?;
        return self.resolve_comparable(comparable);
    }

    pub(crate) fn parse_comparable(&mut self) -> Result<ast::Comparable> {
        return Ok(match self.scanner.peek() {
            Some('\'') => ast::Comparable::String(self.parse_string()?),
            Some('$') => {
                let object = self.parse_interpolation()?;
                self.scanner.skip_whitespace();
                match self.try_parse_binary_operation(ast::Expression::Object(Box::new(object)))? {
                    ast::Expression::Object(interpolation) => {
                        ast::Comparable::Object(*interpolation)
                    }
                    expression => ast::Comparable::Expression(expression),
                }
            }
            Some('+' | '-') => {
                let expression = self.parse_signed_expression()?;
                ast::Comparable::Expression(self.try_parse_binary_operation(expression)?)
            }
            Some('0'..='9') => {
                let expression = self.parse_number().map(ast::Expression::Number)?;
                self.scanner.skip_whitespace();
                ast::Comparable::Expression(self.try_parse_binary_operation(expression)?)
            }
            Some('(') => self.parse_bracketed_comparable()?,
            Some('!') => ast::Comparable::Condition(self.parse_negated_condition()?),
            Some(_) => {
                let func = self.parse_name_or_global_function()?;
                match func {
                    ast::Object::Name(ast::Word { fragments }) if fragments.len() == 1 => {
                        match &fragments[0] {
                            ast::WordFragment::String(ast::StringLiteral { content, location }) => {
                                match content.as_str() {
                                    "true" => ast::Comparable::Condition(ast::Condition::True {
                                        location: location.clone(),
                                    }),
                                    "false" => ast::Comparable::Condition(ast::Condition::False {
                                        location: location.clone(),
                                    }),
                                    name => {
                                        return Err(anyhow::anyhow!(
                                        "objects in comparissons have to be interpolated. Try \"${{{}}}\"",
                                        name
                                    ));
                                    }
                                }
                            }
                            ast::WordFragment::Interpolation(interpolation) => {
                                self.scanner.skip_whitespace();
                                match self.try_parse_binary_operation(ast::Expression::Object(
                                    Box::new(interpolation.clone()),
                                ))? {
                                    ast::Expression::Object(interpolation) => {
                                        ast::Comparable::Object(*interpolation)
                                    }
                                    expression => ast::Comparable::Expression(expression),
                                }
                            }
                        }
                    }
                    ast::Object::Null(null) => ast::Comparable::Null(null),
                    ast::Object::Function(function) => ast::Comparable::Function(function),
                    object => {
                        return Err(anyhow::anyhow!(
                            "objects in comparissons have to be interpolated. Try \"${{{}}}\"",
                            object
                        ))
                    }
                }
            }
            None => return Err(anyhow::anyhow!("unexpected end")),
        });
    }

    fn resolve_comparable(&mut self, comparable: ast::Comparable) -> Result<ast::Condition> {
        self.scanner.skip_whitespace();
        return match self.try_parse_comparisson(&comparable)? {
            Some(comparisson) => self.resolve_comparable(comparisson),
            None => {
                let condition = match comparable {
                    ast::Comparable::Condition(condition) => condition,
                    ast::Comparable::Object(interpolation) => {
                        ast::Condition::Object(interpolation)
                    }
                    ast::Comparable::Function(function) => ast::Condition::Function(function),
                    comparable => {
                        return Err(anyhow::anyhow!("unexpected {}", comparable.r#type()))
                    }
                };
                match self.try_parse_binary_condition(&condition)? {
                    Some(condition) => {
                        self.resolve_comparable(ast::Comparable::Condition(condition))
                    }
                    None => Ok(condition),
                }
            }
        };
    }

    fn try_parse_comparisson(&mut self, left: &ast::Comparable) -> Result<Option<ast::Comparable>> {
        return Ok(match self.scanner.peek() {
            Some(char @ ('!' | '=' | '>' | '<')) => {
                let char = char.clone();
                self.scanner.pop();
                let equals = self.scanner.take(&'=');
                let operator = match (char, equals) {
                    ('=', true) => ast::ComparissonOperator::Equal,
                    ('!', true) => ast::ComparissonOperator::Unequal,
                    ('>', false) => ast::ComparissonOperator::GreaterThan,
                    ('>', true) => ast::ComparissonOperator::GreaterThanOrEqual,
                    ('<', false) => ast::ComparissonOperator::LessThan,
                    ('<', true) => ast::ComparissonOperator::LessThanOrEqual,
                    (_, _) => return Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
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
                Some(ast::Comparable::Condition(ast::Condition::Comparisson {
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
        left: &ast::Condition,
    ) -> Result<Option<ast::Condition>> {
        return Ok(match self.scanner.peek() {
            Some(char @ ('&' | '|')) => {
                let first = &char.clone();
                let operator = match first == &'&' {
                    true => ast::ConditionOperator::And,
                    false => ast::ConditionOperator::Or,
                };
                let operator_location = Location::DoubleCharacter {
                    char: self.scanner.cursor as u16,
                    line: 0,
                };
                self.scanner.pop();
                if !self.scanner.take(first) {
                    return Err(anyhow::anyhow!(
                        "unexpected char \"{}\"",
                        match self.scanner.pop() {
                            Some(char) => char,
                            None => first,
                        }
                    ));
                }
                self.scanner.skip_whitespace();
                Some(ast::Condition::BinaryOperation {
                    left: Box::new(left.clone()),
                    operator,
                    right: Box::new(self.parse_condition()?),
                    operator_location,
                })
            }
            _ => None,
        });
    }

    fn parse_bracketed_comparable(&mut self) -> Result<ast::Comparable> {
        if !self.scanner.take(&'(') {
            return Err(anyhow::anyhow!("expected opening bracket"));
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
            comparable = ast::Comparable::Condition(condition);
            self.scanner.skip_whitespace();
        }
        return match self.scanner.pop() {
            Some(')') => {
                let closing_bracket_location = Location::SingleCharacter {
                    char: self.scanner.cursor as u16 - 1,
                    line: 0,
                };
                match comparable {
                    ast::Comparable::Expression(expression) => Ok(ast::Comparable::Expression(
                        ast::Expression::BracketedExpression {
                            expression: Box::new(expression),
                            opening_bracket_location,
                            closing_bracket_location,
                        },
                    )),
                    ast::Comparable::Condition(condition) => Ok(ast::Comparable::Condition(
                        ast::Condition::BracketedCondition {
                            condition: Box::new(condition),
                            opening_bracket_location,
                            closing_bracket_location,
                        },
                    )),
                    comparable => {
                        return Err(anyhow::anyhow!(
                            "unsupported brackets aroun {}",
                            comparable.r#type()
                        ))
                    }
                }
            }
            Some(char) => return Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
            None => return Err(anyhow::anyhow!("unclosed bracket")),
        };
    }

    fn parse_true_literal(&mut self) -> Result<ast::Condition> {
        match self.scanner.take_str(&"true") {
            true => Ok(ast::Condition::True {
                location: Location::VariableLength {
                    char: self.scanner.cursor as u16 - 4,
                    line: 0,
                    length: 4,
                },
            }),
            false => {
                return Err(match self.scanner.pop() {
                    Some(char) => anyhow::anyhow!("unexpected char \"{}\"", char),
                    None => anyhow::anyhow!("unexpected end"),
                })
            }
        }
    }

    fn parse_false_literal(&mut self) -> Result<ast::Condition> {
        match self.scanner.take_str(&"false") {
            true => Ok(ast::Condition::False {
                location: Location::VariableLength {
                    char: self.scanner.cursor as u16 - 5,
                    line: 0,
                    length: 5,
                },
            }),
            false => {
                return Err(match self.scanner.pop() {
                    Some(char) => anyhow::anyhow!("unexpected char \"{}\"", char),
                    None => anyhow::anyhow!("unexpected end"),
                })
            }
        }
    }

    fn parse_null_literal(&mut self) -> Result<ast::Null> {
        match self.scanner.take_str(&"null") {
            true => Ok(ast::Null {
                location: Location::VariableLength {
                    char: self.scanner.cursor as u16 - 4,
                    line: 0,
                    length: 4,
                },
            }),
            false => {
                return Err(match self.scanner.pop() {
                    Some(char) => anyhow::anyhow!("unexpected char \"{}\"", char),
                    None => anyhow::anyhow!("unexpected end"),
                })
            }
        }
    }

    fn parse_negated_condition(&mut self) -> Result<ast::Condition> {
        let exclamation_mark_location = Location::SingleCharacter {
            char: self.scanner.cursor as u16,
            line: 0,
        };
        self.scanner.pop();
        self.scanner.skip_whitespace();
        match self.parse_condition()? {
            ast::Condition::NegatedCondition { .. } => {
                return Err(anyhow::anyhow!("duplicate condition negation"));
            }
            condition => Ok(ast::Condition::NegatedCondition {
                condition: Box::new(condition),
                exclamation_mark_location,
            }),
        }
    }

    fn parse_number(&mut self) -> Result<ast::Number> {
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
                    return Ok(ast::Number {
                        content: result,
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

    fn parse_integer(&mut self) -> Result<String> {
        let mut result = String::new();
        match self.scanner.pop() {
            Some(char @ '0'..='9') => result.push(*char),
            Some(char) => return Err(anyhow::anyhow!("expected number, found \"{}\"", char)),
            None => return Err(anyhow::anyhow!("unexpected end")),
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

    fn parse_signed_expression(&mut self) -> Result<ast::Expression> {
        let sign = match self.scanner.pop() {
            Some('+') => ast::Sign::Plus,
            Some('-') => ast::Sign::Minus,
            Some(char) => return Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
            None => return Err(anyhow::anyhow!("unexpected end")),
        };
        let sign_location = Location::SingleCharacter {
            char: self.scanner.cursor as u16 - 1,
            line: 0,
        };
        self.scanner.skip_whitespace();
        match self.parse_expression()? {
            ast::Expression::SignedExpression { .. } => {
                return Err(anyhow::anyhow!("duplicate sign"));
            }
            expression => Ok(ast::Expression::SignedExpression {
                expression: Box::new(expression),
                sign,
                sign_location,
            }),
        }
    }

    fn try_parse_binary_operation(&mut self, left: ast::Expression) -> Result<ast::Expression> {
        return Ok(
            match self.scanner.transform(|c| match c {
                '+' => Some(ast::ExpressionOperator::Addition),
                '-' => Some(ast::ExpressionOperator::Subtraction),
                '/' => Some(ast::ExpressionOperator::Division),
                '*' => Some(ast::ExpressionOperator::Multiplication),
                '^' => Some(ast::ExpressionOperator::Power),
                '%' => Some(ast::ExpressionOperator::Modulo),
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
        left_expression: ast::Expression,
        left_operation: ast::ExpressionOperator,
        right_expression: ast::Expression,
        left_operation_location: Location,
    ) -> ast::Expression {
        match right_expression {
            ast::Expression::BinaryOperation {
                left,
                operator: right_operation,
                right,
                operator_location: right_operation_location,
            } if left_operation <= right_operation => ast::Expression::BinaryOperation {
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
            _ => ast::Expression::BinaryOperation {
                left: Box::new(left_expression),
                operator: left_operation,
                right: Box::new(right_expression),
                operator_location: left_operation_location,
            },
        }
    }

    fn parse_string(&mut self) -> Result<ast::StringLiteral> {
        let mut result = String::new();
        let start = self.scanner.cursor as u16;
        if !self.scanner.take(&'\'') {
            return Err(anyhow::anyhow!("expected string"));
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
                            todo!("parse hexadecimal unicode");
                        }
                        Some(char) => {
                            return Err(anyhow::anyhow!("invalid escape sequence \"\\{}\"", char))
                        }
                        None => return Err(anyhow::anyhow!("unexpected end")),
                    }
                }
                Some('\'') => {
                    self.scanner.pop();
                    return Ok(ast::StringLiteral {
                        content: result.clone(),
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
                Some(char) => return Err(anyhow::anyhow!("invalid character \"{}\"", char)),
                None => return Err(anyhow::anyhow!("unexpected end")),
            }
        }
    }

    fn parse_interpolation(&mut self) -> Result<ast::Interpolation> {
        let start = self.scanner.cursor as u16;
        if !self.scanner.take_str(&"${") {
            return Err(anyhow::anyhow!("expected interpolation"));
        }
        self.scanner.skip_whitespace();
        let result = self.parse_object()?;
        self.scanner.skip_whitespace();
        return match self.scanner.take(&'}') {
            true => Ok(ast::Interpolation {
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
            false => Err(anyhow::anyhow!("unclosed interpolation")),
        };
    }

    fn parse_interpolated_anchor(&mut self) -> Result<ast::Anchor> {
        let start = self.scanner.cursor as u16;
        if !self.scanner.take_str(&"!{") {
            return Err(anyhow::anyhow!("expected interpolated anchor"));
        }
        self.scanner.skip_whitespace();
        let result = self.parse_word()?;
        self.scanner.skip_whitespace();
        return match self.scanner.take(&'}') {
            true => Ok(ast::Anchor {
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
            false => Err(anyhow::anyhow!("unclosed interpolated anchor")),
        };
    }

    fn parse_name_or_global_function(&mut self) -> Result<ast::Object> {
        let name = self.parse_word()?;
        // TODO: there probably is a better way...
        match &name.fragments[0] {
            ast::WordFragment::String(ast::StringLiteral { content, location })
                if name.fragments.len() == 1 && content == "null" =>
            {
                return Ok(ast::Object::Null(ast::Null {
                    location: location.clone(),
                }));
            }
            _ => {}
        }
        self.scanner.skip_whitespace();
        let next = self.scanner.peek();
        let mut result = match next {
            Some(&'(') => {
                let start = self.scanner.cursor as u16;
                let arguments = self.parse_function_arguments()?;
                ast::Object::Function(ast::Function {
                    name,
                    arguments,
                    opening_bracket_location: Location::SingleCharacter {
                        char: start,
                        line: 0,
                    },
                    closing_bracket_location: Location::SingleCharacter {
                        char: self.scanner.cursor as u16 - 1,
                        line: 0,
                    },
                })
            }
            _ => ast::Object::Name(name),
        };
        loop {
            match self.scanner.peek() {
                Some('[') => {
                    let start = self.scanner.cursor as u16;
                    self.scanner.pop();
                    self.scanner.skip_whitespace();
                    let expression = self.parse_expression()?;
                    self.scanner.skip_whitespace();
                    match self.scanner.take(&']') {
                        true => {
                            result = ast::Object::ArrayAccess {
                                object: Box::new(result),
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
                        false => return Err(anyhow::anyhow!("unclosed array access")),
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
                    result = match self.scanner.peek() {
                        Some('(') => {
                            let start = self.scanner.cursor as u16;
                            let arguments = self.parse_function_arguments()?;
                            ast::Object::MethodAccess {
                                object: Box::new(result),
                                dot_location,
                                function: ast::Function {
                                    name,
                                    arguments,
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
                        _ => ast::Object::FieldAccess {
                            object: Box::new(result),
                            field: name,
                            dot_location,
                        },
                    }
                }
                _ => return Ok(result),
            }
        }
    }

    fn parse_word(&mut self) -> Result<ast::Word> {
        let mut string = String::new();
        let mut fragments = Vec::new();
        let mut start = self.scanner.cursor as u16;
        loop {
            match self.scanner.peek() {
                Some(char @ ('a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-')) => {
                    string.push(*char);
                    self.scanner.pop();
                }
                Some('$') => {
                    let interpolation = self.parse_interpolation()?;
                    let length = string.len() as u16;
                    if length > 0 {
                        fragments.push(ast::WordFragment::String(ast::StringLiteral {
                            content: string,
                            location: Location::VariableLength {
                                char: start,
                                line: 0,
                                length,
                            },
                        }));
                        string = String::new();
                        start = self.scanner.cursor as u16;
                    }
                    fragments.push(ast::WordFragment::Interpolation(interpolation))
                }
                _ => break,
            }
        }
        let length = string.len() as u16;
        if length > 0 {
            fragments.push(ast::WordFragment::String(ast::StringLiteral {
                content: string,
                location: Location::VariableLength {
                    char: start,
                    line: 0,
                    length,
                },
            }));
        }
        return match fragments.len() > 0 {
            true => Ok(ast::Word { fragments }),
            false => Err(match self.scanner.peek() {
                Some(char) => anyhow::anyhow!("unexpected char \"{}\"", char),
                _ => anyhow::anyhow!("unexpected end"),
            }),
        };
    }

    fn parse_function_arguments(&mut self) -> Result<Vec<ast::FunctionArgument>> {
        let mut arguments = Vec::new();
        if !self.scanner.take(&'(') {
            return Err(anyhow::anyhow!("expected opening brace"));
        }
        self.scanner.skip_whitespace();
        if self.scanner.take(&')') {
            return Ok(arguments);
        }
        loop {
            let argument = match self.scanner.peek() {
                Some('\'') => self.parse_string().map(ast::Argument::String),
                Some('!') => self.parse_interpolated_anchor().map(ast::Argument::Anchor),
                Some('$') => self.parse_interpolation().map(ast::Argument::Object),
                Some('0'..='9') => self.parse_number().map(ast::Argument::Number),
                Some(char @ ('-' | '+')) => {
                    let sign = match char {
                        '+' => ast::Sign::Plus,
                        _ => ast::Sign::Minus,
                    };
                    let sign_location = Location::SingleCharacter {
                        char: self.scanner.cursor as u16,
                        line: 0,
                    };
                    self.scanner.pop();
                    self.scanner.skip_whitespace();
                    self.parse_number()
                        .map(|number| ast::SignedNumber {
                            sign,
                            number,
                            sign_location,
                        })
                        .map(ast::Argument::SignedNumber)
                }
                Some('n') => self.parse_null_literal().map(ast::Argument::Null),
                Some(char) => Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
                None => Err(anyhow::anyhow!("unexpected end")),
            }?;
            self.scanner.skip_whitespace();
            match self.scanner.pop() {
                Some(')') => {
                    arguments.push(ast::FunctionArgument {
                        argument,
                        comma_location: None,
                    });
                    return Ok(arguments);
                }
                Some(',') => {
                    arguments.push(ast::FunctionArgument {
                        argument,
                        comma_location: Some(ast::Location::SingleCharacter {
                            char: self.scanner.cursor as u16 - 1,
                            line: 0,
                        }),
                    });
                    self.scanner.skip_whitespace();
                }
                Some(char) => return Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
                None => return Err(anyhow::anyhow!("unclosed function arguments")),
            };
        }
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
                    content: "test".to_string(),
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
                    content: "test".to_string(),
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
                    content: "tes\\\'t".to_string(),
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
            parse_object("null"),
            ObjectAst {
                root: Object::Null(Null {
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
                            content: "null".to_string(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 4,
                            }
                        }),
                        WordFragment::Interpolation(Interpolation {
                            content: Object::String(StringLiteral {
                                content: "notNull".to_string(),
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
    fn test_parse_null_not_a_function() {
        let string = "null()";
        (&mut super::Parser::new(&string))
            .parse_object_ast()
            .expect_err(&format!(
                "successfully parsed invalid string \"{}\"",
                string
            ));
    }

    #[test]
    fn test_parse_simple_interpolated_anchor() {
        assert_eq!(
            parse_object("!{home}"),
            ObjectAst {
                root: Object::Anchor(Anchor {
                    name: Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "home".to_string(),
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
                                    content: "home".to_string(),
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
                                content: "home-".to_string(),
                                location: Location::VariableLength {
                                    char: 2,
                                    line: 0,
                                    length: 5,
                                }
                            }),
                            WordFragment::Interpolation(Interpolation {
                                content: Object::Name(Word {
                                    fragments: vec![WordFragment::String(StringLiteral {
                                        content: "_object".to_string(),
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
                                content: "-content".to_string(),
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
                    content: "hello, ${world}".to_string(),
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
                    name: Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "flush".to_string(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 5,
                            },
                        })]
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
                    name: Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "is_string".to_string(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 9,
                            },
                        })]
                    },
                    arguments: vec![FunctionArgument {
                        argument: Argument::String(StringLiteral {
                            content: "test".to_string(),
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
    fn test_parse_interpolated_global_function() {
        assert_eq!(
            parse_object("is_${_type}()"),
            ObjectAst {
                root: Object::Function(Function {
                    name: Word {
                        fragments: vec![
                            WordFragment::String(StringLiteral {
                                content: "is_".to_string(),
                                location: Location::VariableLength {
                                    char: 0,
                                    line: 0,
                                    length: 3,
                                },
                            }),
                            WordFragment::Interpolation(Interpolation {
                                content: Object::Name(Word {
                                    fragments: vec![WordFragment::String(StringLiteral {
                                        content: "_type".to_string(),
                                        location: Location::VariableLength {
                                            char: 5,
                                            line: 0,
                                            length: 5,
                                        },
                                    })]
                                }),
                                opening_bracket_location: Location::DoubleCharacter {
                                    char: 3,
                                    line: 0,
                                },
                                closing_bracket_location: Location::SingleCharacter {
                                    char: 10,
                                    line: 0,
                                },
                            })
                        ]
                    },
                    arguments: vec![],
                    opening_bracket_location: Location::SingleCharacter { char: 11, line: 0 },
                    closing_bracket_location: Location::SingleCharacter { char: 12, line: 0 },
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
                    name: Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "is_string".to_string(),
                            location: Location::VariableLength {
                                char: 1,
                                line: 0,
                                length: 9,
                            },
                        })]
                    },
                    arguments: vec![
                        FunctionArgument {
                            argument: Argument::String(StringLiteral {
                                content: "test".to_string(),
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
                                content: "test2".to_string(),
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

    // TODO: I don't think this is possible, it would have to be like this:
    //
    //       is_string(${concat('hello', 'world')})
    //
    // #[test]
    // fn test_parse_nested_global_function() {
    //     assert_eq!(
    //         parse_object("is_string(concat('hello', 'world'))"),
    //         ObjectAst {
    //             root: Object::Function(Function {
    //                 name: Word {
    //                     fragments: vec![WordFragment::String(StringLiteral {
    //                         content: "is_string".to_string(),
    //                         location: Location::VariableLength {
    //                             char: 0,
    //                             line: 0,
    //                             length: 9,
    //                         },
    //                     })]
    //                 },
    //                 arguments: vec![FunctionArgument {
    //                     argument: Object::Function(Function {
    //                         name: Word {
    //                             fragments: vec![WordFragment::String(StringLiteral {
    //                                 content: "concat".to_string(),
    //                                 location: Location::VariableLength {
    //                                     char: 10,
    //                                     line: 0,
    //                                     length: 6,
    //                                 },
    //                             })]
    //                         },
    //                         arguments: vec![
    //                             FunctionArgument {
    //                                 argument: Object::String(StringLiteral {
    //                                     content: "hello".to_string(),
    //                                     location: Location::VariableLength {
    //                                         char: 17,
    //                                         line: 0,
    //                                         length: 7,
    //                                     },
    //                                 }),
    //                                 comma_location: Some(Location::SingleCharacter {
    //                                     char: 24,
    //                                     line: 0
    //                                 })
    //                             },
    //                             FunctionArgument {
    //                                 argument: Object::String(StringLiteral {
    //                                     content: "world".to_string(),
    //                                     location: Location::VariableLength {
    //                                         char: 26,
    //                                         line: 0,
    //                                         length: 7,
    //                                     },
    //                                 }),
    //                                 comma_location: None,
    //                             }
    //                         ],
    //                         opening_bracket_location: Location::SingleCharacter {
    //                             char: 16,
    //                             line: 0
    //                         },
    //                         closing_bracket_location: Location::SingleCharacter {
    //                             char: 33,
    //                             line: 0
    //                         },
    //                     }),
    //                     comma_location: None
    //                 }],
    //                 opening_bracket_location: Location::SingleCharacter { char: 9, line: 0 },
    //                 closing_bracket_location: Location::SingleCharacter { char: 34, line: 0 },
    //             })
    //         }
    //     );
    // }

    #[test]
    fn test_parse_simple_field_access() {
        assert_eq!(
            parse_object("_string.length"),
            ObjectAst {
                root: Object::FieldAccess {
                    object: Box::new(Object::Name(Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "_string".to_string(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 7,
                            },
                        })]
                    })),
                    field: Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "length".to_string(),
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
                            content: "_string".to_string(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 7,
                            },
                        })]
                    })),
                    function: Function {
                        name: Word {
                            fragments: vec![WordFragment::String(StringLiteral {
                                content: "length".to_string(),
                                location: Location::VariableLength {
                                    char: 8,
                                    line: 0,
                                    length: 6,
                                },
                            })]
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
                            content: "_strings".to_string(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 8,
                            },
                        })]
                    })),
                    index: Expression::Number(Number {
                        content: "0".to_string(),
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
                            content: "_strings".to_string(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 8,
                            },
                        })]
                    })),
                    index: Expression::SignedExpression {
                        expression: Box::new(Expression::Number(Number {
                            content: "1".to_string(),
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
                    content: "123".to_string(),
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
                        content: "13.5e-2".to_string(),
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
                                content: "6E+2".to_string(),
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
                        content: "6".to_string(),
                        location: Location::VariableLength {
                            char: 0,
                            line: 0,
                            length: 1,
                        }
                    })),
                    operator: ExpressionOperator::Addition,
                    right: Box::new(Expression::Number(Number {
                        content: "9".to_string(),
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
                            content: "1".to_string(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 1,
                            }
                        })),
                        operator: ExpressionOperator::Addition,
                        right: Box::new(Expression::Number(Number {
                            content: "2".to_string(),
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
                            content: "3".to_string(),
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
                        content: "6".to_string(),
                        location: Location::VariableLength {
                            char: 0,
                            line: 0,
                            length: 1,
                        }
                    })),
                    operator: ExpressionOperator::Addition,
                    right: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::Number(Number {
                            content: "10".to_string(),
                            location: Location::VariableLength {
                                char: 4,
                                line: 0,
                                length: 2,
                            }
                        })),
                        operator: ExpressionOperator::Division,
                        right: Box::new(Expression::Number(Number {
                            content: "2".to_string(),
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
                        content: "1".to_string(),
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
                                    content: "2".to_string(),
                                    location: Location::VariableLength {
                                        char: 4,
                                        line: 0,
                                        length: 1,
                                    }
                                })),
                                operator: ExpressionOperator::Division,
                                right: Box::new(Expression::Number(Number {
                                    content: "3".to_string(),
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
                                    content: "4".to_string(),
                                    location: Location::VariableLength {
                                        char: 12,
                                        line: 0,
                                        length: 1,
                                    }
                                })),
                                operator: ExpressionOperator::Power,
                                right: Box::new(Expression::Number(Number {
                                    content: "5".to_string(),
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
                            content: "6".to_string(),
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
                        content: "1".to_string(),
                        location: Location::VariableLength {
                            char: 7,
                            line: 0,
                            length: 1
                        }
                    })),
                    right: Box::new(Expression::Number(Number {
                        content: "2".to_string(),
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
                                content: "test".to_string(),
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
                    name: Word {
                        fragments: vec![WordFragment::String(StringLiteral {
                            content: "isNull".to_string(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 6
                            }
                        })]
                    },
                    arguments: vec![FunctionArgument {
                        argument: Argument::Object(Interpolation {
                            content: Object::Name(Word {
                                fragments: vec![WordFragment::String(StringLiteral {
                                    content: "_test".to_string(),
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
                        content: "3".to_string(),
                        location: Location::VariableLength {
                            char: 0,
                            line: 0,
                            length: 1
                        }
                    }))),
                    operator: ComparissonOperator::GreaterThanOrEqual,
                    right: Box::new(Comparable::Expression(Expression::Number(Number {
                        content: "4".to_string(),
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
                            content: "3".to_string(),
                            location: Location::VariableLength {
                                char: 0,
                                line: 0,
                                length: 1
                            }
                        }))),
                        operator: ComparissonOperator::Unequal,
                        right: Box::new(Comparable::Expression(Expression::Number(Number {
                            content: "4".to_string(),
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
