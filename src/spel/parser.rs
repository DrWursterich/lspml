use super::{
    ast::{self, ExpressionAst, Location},
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
        let root = self.parse_object();
        self.scanner.skip_whitespace();
        return match self.scanner.is_done() {
            true => root.map(ObjectAst::new),
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
        let root = self.parse_expression();
        self.scanner.skip_whitespace();
        return match self.scanner.is_done() {
            true => root.map(ExpressionAst::new),
            false => Err(anyhow::anyhow!(
                "trailing character \"{}\"",
                self.scanner.peek().unwrap()
            )),
        };
    }

    pub(crate) fn parse_identifier(&mut self) -> Result<ast::Word> {
        self.scanner.skip_whitespace();
        if self.scanner.is_done() {
            return Err(anyhow::anyhow!("string is empty"));
        }
        let word = self.parse_word();
        self.scanner.skip_whitespace();
        return match self.scanner.is_done() {
            true => word,
            false => Err(anyhow::anyhow!(
                "trailing character \"{}\"",
                self.scanner.peek().unwrap()
            )),
        };
    }

    fn parse_object(&mut self) -> Result<ast::Object> {
        return match self.scanner.peek() {
            Some('\'') => self.parse_string(),
            Some('!') => self.parse_interpolated_anchor(),
            Some('$' | 'a'..='z' | 'A'..='Z' | '_') => self.parse_name_or_global_function(),
            Some(char) => Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
            None => Err(anyhow::anyhow!("unexpected end")),
        };
    }

    fn parse_expression(&mut self) -> Result<ast::Expression> {
        let result = match self.scanner.peek() {
            Some('(') => {
                let start = self.scanner.cursor as u16;
                self.scanner.pop();
                self.scanner.skip_whitespace();
                let expression = self.parse_expression()?;
                self.scanner.skip_whitespace();
                if self.scanner.pop() != Some(&')') {
                    return Err(anyhow::anyhow!("unclosed bracket"));
                }
                ast::Expression::BracketedExpression {
                    expression: Box::new(expression),
                    opening_bracket_location: Location {
                        char: start,
                        line: 0,
                        length: 1,
                    },
                    closing_bracket_location: Location {
                        char: self.scanner.cursor as u16 - 1,
                        line: 0,
                        length: 1,
                    },
                }
            }
            Some('0'..='9') => self.parse_number()?,
            _ => {
                let location = Location {
                    char: self.scanner.cursor as u16,
                    line: 0,
                    length: 1,
                };
                let sign = match self.scanner.pop() {
                    Some('+') => ast::Sign::Plus { location },
                    Some('-') => ast::Sign::Minus { location },
                    Some(char) => return Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
                    _ => return Err(anyhow::anyhow!("unexpected end")),
                };
                self.scanner.skip_whitespace();
                match self.parse_expression()? {
                    ast::Expression::SignedExpression { .. } => {
                        return Err(anyhow::anyhow!("duplicate sign"));
                    }
                    expression => ast::Expression::SignedExpression {
                        expression: Box::new(expression),
                        sign,
                    },
                }
            }
        };
        self.scanner.skip_whitespace();
        return Ok(
            match self.scanner.transform(|c| match c {
                '+' => Some(ast::Operation::Addition),
                '-' => Some(ast::Operation::Subtraction),
                '/' => Some(ast::Operation::Division),
                '*' => Some(ast::Operation::Multiplication),
                '^' => Some(ast::Operation::Power),
                '%' => Some(ast::Operation::Modulo),
                _ => None,
            }) {
                Some(operation) => {
                    self.scanner.skip_whitespace();
                    let expression = self.parse_expression()?;
                    self.resolve_binary_operation_precidence(result, operation, expression)
                }
                None => result,
            },
        );
    }

    fn parse_number(&mut self) -> Result<ast::Expression> {
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
                    return Ok(ast::Expression::Number {
                        content: result,
                        location: Location {
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

    fn resolve_binary_operation_precidence(
        &mut self,
        left_expression: ast::Expression,
        left_operation: ast::Operation,
        right_expression: ast::Expression,
    ) -> ast::Expression {
        match right_expression {
            ast::Expression::BinaryOperation {
                left,
                operation: right_operation,
                right,
            } if left_operation <= right_operation => ast::Expression::BinaryOperation {
                left: Box::new(self.resolve_binary_operation_precidence(
                    left_expression,
                    left_operation,
                    *left,
                )),
                operation: right_operation,
                right,
            },
            _ => ast::Expression::BinaryOperation {
                left: Box::new(left_expression),
                operation: left_operation,
                right: Box::new(right_expression),
            },
        }
    }

    fn parse_string(&mut self) -> Result<ast::Object> {
        let mut result = String::new();
        let mut interpolations = Vec::new();
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
                Some('$') => interpolations.push(self.parse_interpolation()?),
                Some('\'') => {
                    self.scanner.pop();
                    return Ok(ast::Object::String {
                        content: result.clone(),
                        interpolations,
                        location: Location {
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
                Some(char) => {
                    return Err(anyhow::anyhow!("invalid character \"{}\"", char));
                }
                None => return Err(anyhow::anyhow!("unexpected end")),
            }
        }
    }

    fn parse_interpolation(&mut self) -> Result<ast::Interpolation> {
        let start = self.scanner.cursor as u16;
        if !self.scanner.take(&'$') {
            return Err(anyhow::anyhow!("expected interpolation"));
        }
        if !self.scanner.take(&'{') {
            return Err(anyhow::anyhow!("expected interpolation"));
        }
        self.scanner.skip_whitespace();
        let result = self.parse_object()?;
        self.scanner.skip_whitespace();
        return match self.scanner.take(&'}') {
            true => Ok(ast::Interpolation {
                content: result,
                opening_bracket_location: Location {
                    char: start,
                    line: 0,
                    length: 2,
                },
                closing_bracket_location: Location {
                    char: self.scanner.cursor as u16 - 1,
                    line: 0,
                    length: 1,
                },
            }),
            false => Err(anyhow::anyhow!("unclosed interpolation")),
        };
    }

    fn parse_interpolated_anchor(&mut self) -> Result<ast::Object> {
        let start = self.scanner.cursor as u16;
        if !self.scanner.take(&'!') {
            return Err(anyhow::anyhow!("expected interpolated anchor"));
        }
        if !self.scanner.take(&'{') {
            return Err(anyhow::anyhow!("expected interpolated anchor"));
        }
        self.scanner.skip_whitespace();
        let result = self.parse_word()?;
        self.scanner.skip_whitespace();
        return match self.scanner.take(&'}') {
            true => Ok(ast::Object::Anchor {
                name: result,
                opening_bracket_location: Location {
                    char: start,
                    line: 0,
                    length: 2,
                },
                closing_bracket_location: Location {
                    char: self.scanner.cursor as u16 - 1,
                    line: 0,
                    length: 1,
                },
            }),
            false => Err(anyhow::anyhow!("unclosed interpolated anchor")),
        };
    }

    fn parse_name_or_global_function(&mut self) -> Result<ast::Object> {
        let name = self.parse_word()?;
        self.scanner.skip_whitespace();
        if name.name == "null" && name.interpolations.len() == 0 {
            return Ok(ast::Object::Null {
                location: name.location,
            });
        }
        let mut result = match self.scanner.peek() {
            Some(&'(') => {
                let start = self.scanner.cursor as u16;
                let arguments = self.parse_function_arguments()?;
                ast::Object::Function {
                    name,
                    arguments,
                    opening_bracket_location: Location {
                        char: start,
                        line: 0,
                        length: 1,
                    },
                    closing_bracket_location: Location {
                        char: self.scanner.cursor as u16 - 1,
                        line: 0,
                        length: 1,
                    },
                }
            }
            _ => ast::Object::Name { name },
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
                                opening_bracket_location: Location {
                                    char: start,
                                    line: 0,
                                    length: 1,
                                },
                                closing_bracket_location: Location {
                                    char: self.scanner.cursor as u16 - 1,
                                    line: 0,
                                    length: 1,
                                },
                            }
                        }
                        false => return Err(anyhow::anyhow!("unclosed array access")),
                    }
                }
                Some('.') => {
                    let dot_location = Location {
                        char: self.scanner.cursor as u16,
                        line: 0,
                        length: 1,
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
                                method: name,
                                arguments,
                                dot_location,
                                opening_bracket_location: Location {
                                    char: start,
                                    line: 0,
                                    length: 1,
                                },
                                closing_bracket_location: Location {
                                    char: self.scanner.cursor as u16 - 1,
                                    line: 0,
                                    length: 1,
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
        let mut result = String::new();
        let mut interpolations = Vec::new();
        let start = self.scanner.cursor as u16;
        loop {
            match self.scanner.peek() {
                Some(char @ ('a'..='z' | 'A'..='Z' | '_' | '-')) => {
                    result.push(*char);
                    self.scanner.pop();
                }
                Some('$') => interpolations.push(self.parse_interpolation()?),
                _ => break,
            }
        }
        return Ok(ast::Word {
            name: result.clone(),
            interpolations,
            location: Location {
                char: start,
                line: 0,
                length: result.len() as u16,
            },
        });
    }

    fn parse_function_arguments(&mut self) -> Result<Vec<ast::Object>> {
        let mut arguments = Vec::new();
        if !self.scanner.take(&'(') {
            return Err(anyhow::anyhow!("expected opening brace"));
        }
        self.scanner.skip_whitespace();
        if self.scanner.take(&')') {
            return Ok(arguments);
        }
        loop {
            arguments.push(self.parse_object()?);
            self.scanner.skip_whitespace();
            match self.scanner.pop() {
                Some(')') => return Ok(arguments),
                Some(',') => self.scanner.skip_whitespace(),
                Some(char) => return Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
                None => return Err(anyhow::anyhow!("unclosed function arguments")),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::spel::ast::{
        Expression, ExpressionAst, Interpolation, Location, Object, ObjectAst, Operation, Sign,
        Word,
    };

    #[test]
    fn test_parse_simple_string_object() {
        assert_eq!(
            parse_object("'test'"),
            ObjectAst {
                root: Object::String {
                    content: "test".to_string(),
                    interpolations: vec![],
                    location: Location {
                        char: 0,
                        line: 0,
                        length: 6,
                    }
                }
            }
        );
    }

    #[test]
    fn test_parse_string_object_with_whitespace() {
        assert_eq!(
            parse_object("\t'test'   "),
            ObjectAst {
                root: Object::String {
                    content: "test".to_string(),
                    interpolations: vec![],
                    location: Location {
                        char: 1,
                        line: 0,
                        length: 6,
                    }
                }
            }
        );
    }

    #[test]
    fn test_parse_escaped_string_object() {
        assert_eq!(
            parse_object("'tes\\\'t'"),
            ObjectAst {
                root: Object::String {
                    content: "tes\\\'t".to_string(),
                    interpolations: vec![],
                    location: Location {
                        char: 0,
                        line: 0,
                        length: 8,
                    }
                }
            }
        );
    }

    #[test]
    fn test_parse_null() {
        assert_eq!(
            parse_object("null"),
            ObjectAst {
                root: Object::Null {
                    location: Location {
                        char: 0,
                        line: 0,
                        length: 4,
                    }
                }
            }
        );
    }

    #[test]
    fn test_parse_interpolated_null() {
        assert_eq!(
            parse_object("null${'notNull'}"),
            ObjectAst {
                root: Object::Name {
                    name: Word {
                        name: "null".to_string(),
                        interpolations: vec![Interpolation {
                            content: Object::String {
                                content: "notNull".to_string(),
                                interpolations: vec![],
                                location: Location {
                                    char: 6,
                                    line: 0,
                                    length: 9,
                                }
                            },
                            opening_bracket_location: Location {
                                char: 4,
                                line: 0,
                                length: 2,
                            },
                            closing_bracket_location: Location {
                                char: 15,
                                line: 0,
                                length: 1,
                            },
                        }],
                        location: Location {
                            char: 0,
                            line: 0,
                            length: 4,
                        }
                    }
                }
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
                root: Object::Anchor {
                    name: Word {
                        name: "home".to_string(),
                        interpolations: vec![],
                        location: Location {
                            char: 2,
                            line: 0,
                            length: 4,
                        }
                    },
                    opening_bracket_location: Location {
                        char: 0,
                        line: 0,
                        length: 2,
                    },
                    closing_bracket_location: Location {
                        char: 6,
                        line: 0,
                        length: 1,
                    },
                }
            }
        );
    }

    #[test]
    fn test_parse_interpolated_string_object() {
        assert_eq!(
            parse_object("'hello, ${world}'"),
            ObjectAst {
                root: Object::String {
                    content: "hello, ".to_string(),
                    interpolations: vec![Interpolation {
                        content: Object::Name {
                            name: Word {
                                name: "world".to_string(),
                                interpolations: vec![],
                                location: Location {
                                    char: 10,
                                    line: 0,
                                    length: 5,
                                }
                            }
                        },
                        opening_bracket_location: Location {
                            char: 8,
                            line: 0,
                            length: 2,
                        },
                        closing_bracket_location: Location {
                            char: 15,
                            line: 0,
                            length: 1,
                        },
                    }],
                    location: Location {
                        char: 0,
                        line: 0,
                        length: 9,
                    },
                }
            }
        );
    }

    #[test]
    fn test_parse_simple_global_function() {
        assert_eq!(
            parse_object("flush()"),
            ObjectAst {
                root: Object::Function {
                    name: Word {
                        name: "flush".to_string(),
                        interpolations: vec![],
                        location: Location {
                            char: 0,
                            line: 0,
                            length: 5,
                        },
                    },
                    arguments: vec![],
                    opening_bracket_location: Location {
                        char: 5,
                        line: 0,
                        length: 1,
                    },
                    closing_bracket_location: Location {
                        char: 6,
                        line: 0,
                        length: 1,
                    },
                }
            }
        );
    }

    #[test]
    fn test_parse_global_function_with_argument() {
        assert_eq!(
            parse_object("is_string('test')"),
            ObjectAst {
                root: Object::Function {
                    name: Word {
                        name: "is_string".to_string(),
                        interpolations: vec![],
                        location: Location {
                            char: 0,
                            line: 0,
                            length: 9,
                        },
                    },
                    arguments: vec![Object::String {
                        content: "test".to_string(),
                        interpolations: vec![],
                        location: Location {
                            char: 10,
                            line: 0,
                            length: 6,
                        },
                    }],
                    opening_bracket_location: Location {
                        char: 9,
                        line: 0,
                        length: 1,
                    },
                    closing_bracket_location: Location {
                        char: 16,
                        line: 0,
                        length: 1,
                    },
                }
            }
        );
    }

    #[test]
    fn test_parse_interpolated_global_function() {
        assert_eq!(
            parse_object("is_${_type}()"),
            ObjectAst {
                root: Object::Function {
                    name: Word {
                        name: "is_".to_string(),
                        interpolations: vec![Interpolation {
                            content: Object::Name {
                                name: Word {
                                    name: "_type".to_string(),
                                    interpolations: vec![],
                                    location: Location {
                                        char: 5,
                                        line: 0,
                                        length: 5,
                                    },
                                },
                            },
                            opening_bracket_location: Location {
                                char: 3,
                                line: 0,
                                length: 2,
                            },
                            closing_bracket_location: Location {
                                char: 10,
                                line: 0,
                                length: 1,
                            },
                        }],
                        location: Location {
                            char: 0,
                            line: 0,
                            length: 3,
                        },
                    },
                    arguments: vec![],
                    opening_bracket_location: Location {
                        char: 11,
                        line: 0,
                        length: 1,
                    },
                    closing_bracket_location: Location {
                        char: 12,
                        line: 0,
                        length: 1,
                    },
                }
            }
        );
    }

    #[test]
    fn test_parse_global_function_with_excessive_whitespace() {
        assert_eq!(
            parse_object("\tis_string (\t'test'  , 'test2' ) "),
            ObjectAst {
                root: Object::Function {
                    name: Word {
                        name: "is_string".to_string(),
                        interpolations: vec![],
                        location: Location {
                            char: 1,
                            line: 0,
                            length: 9,
                        },
                    },
                    arguments: vec![
                        Object::String {
                            content: "test".to_string(),
                            interpolations: vec![],
                            location: Location {
                                char: 13,
                                line: 0,
                                length: 6,
                            },
                        },
                        Object::String {
                            content: "test2".to_string(),
                            interpolations: vec![],
                            location: Location {
                                char: 23,
                                line: 0,
                                length: 7,
                            },
                        }
                    ],
                    opening_bracket_location: Location {
                        char: 11,
                        line: 0,
                        length: 1,
                    },
                    closing_bracket_location: Location {
                        char: 31,
                        line: 0,
                        length: 1,
                    },
                }
            }
        );
    }

    #[test]
    fn test_parse_nested_global_function() {
        assert_eq!(
            parse_object("is_string(concat('hello', 'world'))"),
            ObjectAst {
                root: Object::Function {
                    name: Word {
                        name: "is_string".to_string(),
                        interpolations: vec![],
                        location: Location {
                            char: 0,
                            line: 0,
                            length: 9,
                        },
                    },
                    arguments: vec![Object::Function {
                        name: Word {
                            name: "concat".to_string(),
                            interpolations: vec![],
                            location: Location {
                                char: 10,
                                line: 0,
                                length: 6,
                            },
                        },
                        arguments: vec![
                            Object::String {
                                content: "hello".to_string(),
                                interpolations: vec![],
                                location: Location {
                                    char: 17,
                                    line: 0,
                                    length: 7,
                                },
                            },
                            Object::String {
                                content: "world".to_string(),
                                interpolations: vec![],
                                location: Location {
                                    char: 26,
                                    line: 0,
                                    length: 7,
                                },
                            }
                        ],
                        opening_bracket_location: Location {
                            char: 16,
                            line: 0,
                            length: 1,
                        },
                        closing_bracket_location: Location {
                            char: 33,
                            line: 0,
                            length: 1,
                        },
                    }],
                    opening_bracket_location: Location {
                        char: 9,
                        line: 0,
                        length: 1,
                    },
                    closing_bracket_location: Location {
                        char: 34,
                        line: 0,
                        length: 1,
                    },
                }
            }
        );
    }

    #[test]
    fn test_parse_simple_field_access() {
        assert_eq!(
            parse_object("_string.length"),
            ObjectAst {
                root: Object::FieldAccess {
                    object: Box::new(Object::Name {
                        name: Word {
                            name: "_string".to_string(),
                            interpolations: vec![],
                            location: Location {
                                char: 0,
                                line: 0,
                                length: 7,
                            },
                        }
                    }),
                    field: Word {
                        name: "length".to_string(),
                        interpolations: vec![],
                        location: Location {
                            char: 8,
                            line: 0,
                            length: 6,
                        },
                    },
                    dot_location: Location {
                        char: 7,
                        line: 0,
                        length: 1,
                    },
                }
            }
        );
    }

    #[test]
    fn test_parse_simple_method_access() {
        assert_eq!(
            parse_object("_string.length()"),
            ObjectAst {
                root: Object::MethodAccess {
                    object: Box::new(Object::Name {
                        name: Word {
                            name: "_string".to_string(),
                            interpolations: vec![],
                            location: Location {
                                char: 0,
                                line: 0,
                                length: 7,
                            },
                        }
                    }),
                    method: Word {
                        name: "length".to_string(),
                        interpolations: vec![],
                        location: Location {
                            char: 8,
                            line: 0,
                            length: 6,
                        },
                    },
                    arguments: vec![],
                    dot_location: Location {
                        char: 7,
                        line: 0,
                        length: 1,
                    },
                    opening_bracket_location: Location {
                        char: 14,
                        line: 0,
                        length: 1,
                    },
                    closing_bracket_location: Location {
                        char: 15,
                        line: 0,
                        length: 1,
                    },
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
                    object: Box::new(Object::Name {
                        name: Word {
                            name: "_strings".to_string(),
                            interpolations: vec![],
                            location: Location {
                                char: 0,
                                line: 0,
                                length: 8,
                            },
                        }
                    }),
                    index: Expression::Number {
                        content: "0".to_string(),
                        location: Location {
                            char: 9,
                            line: 0,
                            length: 1,
                        }
                    },
                    opening_bracket_location: Location {
                        char: 8,
                        line: 0,
                        length: 1,
                    },
                    closing_bracket_location: Location {
                        char: 10,
                        line: 0,
                        length: 1,
                    }
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
                    object: Box::new(Object::Name {
                        name: Word {
                            name: "_strings".to_string(),
                            interpolations: vec![],
                            location: Location {
                                char: 0,
                                line: 0,
                                length: 8,
                            },
                        }
                    }),
                    index: Expression::SignedExpression {
                        expression: Box::new(Expression::Number {
                            content: "1".to_string(),
                            location: Location {
                                char: 10,
                                line: 0,
                                length: 1,
                            }
                        }),
                        sign: Sign::Plus {
                            location: Location {
                                char: 9,
                                line: 0,
                                length: 1,
                            }
                        },
                    },
                    opening_bracket_location: Location {
                        char: 8,
                        line: 0,
                        length: 1,
                    },
                    closing_bracket_location: Location {
                        char: 11,
                        line: 0,
                        length: 1,
                    }
                }
            }
        );
    }

    #[test]
    fn test_parse_simple_number() {
        assert_eq!(
            parse_expression("123"),
            ExpressionAst {
                root: Expression::Number {
                    content: "123".to_string(),
                    location: Location {
                        char: 0,
                        line: 0,
                        length: 3,
                    }
                }
            }
        );
    }

    #[test]
    fn test_parse_scientific_number() {
        assert_eq!(
            parse_expression("-13.5e-2"),
            ExpressionAst {
                root: Expression::SignedExpression {
                    expression: Box::new(Expression::Number {
                        content: "13.5e-2".to_string(),
                        location: Location {
                            char: 1,
                            line: 0,
                            length: 7,
                        }
                    }),
                    sign: Sign::Minus {
                        location: Location {
                            char: 0,
                            line: 0,
                            length: 1,
                        }
                    }
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
                            expression: Box::new(Expression::Number {
                                content: "6E+2".to_string(),
                                location: Location {
                                    char: 5,
                                    line: 0,
                                    length: 4,
                                }
                            }),
                            sign: Sign::Minus {
                                location: Location {
                                    char: 4,
                                    line: 0,
                                    length: 1,
                                }
                            }
                        }),
                        opening_bracket_location: Location {
                            char: 2,
                            line: 0,
                            length: 1,
                        },
                        closing_bracket_location: Location {
                            char: 10,
                            line: 0,
                            length: 1,
                        }
                    }),
                    sign: Sign::Minus {
                        location: Location {
                            char: 1,
                            line: 0,
                            length: 1,
                        }
                    }
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
                    left: Box::new(Expression::Number {
                        content: "6".to_string(),
                        location: Location {
                            char: 0,
                            line: 0,
                            length: 1,
                        }
                    }),
                    operation: Operation::Addition,
                    right: Box::new(Expression::Number {
                        content: "9".to_string(),
                        location: Location {
                            char: 4,
                            line: 0,
                            length: 1,
                        }
                    }),
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
                        left: Box::new(Expression::Number {
                            content: "1".to_string(),
                            location: Location {
                                char: 0,
                                line: 0,
                                length: 1,
                            }
                        }),
                        operation: Operation::Addition,
                        right: Box::new(Expression::Number {
                            content: "2".to_string(),
                            location: Location {
                                char: 4,
                                line: 0,
                                length: 1,
                            }
                        })
                    }),
                    operation: Operation::Addition,
                    right: Box::new(Expression::SignedExpression {
                        expression: Box::new(Expression::Number {
                            content: "3".to_string(),
                            location: Location {
                                char: 9,
                                line: 0,
                                length: 1,
                            }
                        }),
                        sign: Sign::Minus {
                            location: Location {
                                char: 8,
                                line: 0,
                                length: 1,
                            }
                        }
                    })
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
                    left: Box::new(Expression::Number {
                        content: "6".to_string(),
                        location: Location {
                            char: 0,
                            line: 0,
                            length: 1,
                        }
                    }),
                    operation: Operation::Addition,
                    right: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::Number {
                            content: "10".to_string(),
                            location: Location {
                                char: 4,
                                line: 0,
                                length: 2,
                            }
                        }),
                        operation: Operation::Division,
                        right: Box::new(Expression::Number {
                            content: "2".to_string(),
                            location: Location {
                                char: 9,
                                line: 0,
                                length: 1,
                            }
                        }),
                    })
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
                    left: Box::new(Expression::Number {
                        content: "1".to_string(),
                        location: Location {
                            char: 0,
                            line: 0,
                            length: 1,
                        }
                    }),
                    operation: Operation::Addition,
                    right: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::BinaryOperation {
                            left: Box::new(Expression::BinaryOperation {
                                left: Box::new(Expression::Number {
                                    content: "2".to_string(),
                                    location: Location {
                                        char: 4,
                                        line: 0,
                                        length: 1,
                                    }
                                }),
                                operation: Operation::Division,
                                right: Box::new(Expression::Number {
                                    content: "3".to_string(),
                                    location: Location {
                                        char: 8,
                                        line: 0,
                                        length: 1,
                                    }
                                })
                            }),
                            operation: Operation::Multiplication,
                            right: Box::new(Expression::BinaryOperation {
                                left: Box::new(Expression::Number {
                                    content: "4".to_string(),
                                    location: Location {
                                        char: 12,
                                        line: 0,
                                        length: 1,
                                    }
                                }),
                                operation: Operation::Power,
                                right: Box::new(Expression::Number {
                                    content: "5".to_string(),
                                    location: Location {
                                        char: 16,
                                        line: 0,
                                        length: 1,
                                    }
                                })
                            })
                        }),
                        operation: Operation::Modulo,
                        right: Box::new(Expression::Number {
                            content: "6".to_string(),
                            location: Location {
                                char: 20,
                                line: 0,
                                length: 1,
                            }
                        }),
                    })
                }
            }
        );
    }

    fn parse_object(string: &str) -> ObjectAst {
        return (&mut super::Parser::new(&string))
            .parse_object_ast()
            .expect(&format!("error parsing \"{}\"", string));
    }

    fn parse_expression(string: &str) -> ExpressionAst {
        return (&mut super::Parser::new(&string))
            .parse_expression_ast()
            .expect(&format!("error parsing \"{}\"", string));
    }
}
