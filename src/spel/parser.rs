use super::{
    ast::{self, ExpressionAst},
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
        let root = self.next_object();
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
        let root = self.next_expression();
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

    fn next_object(&mut self) -> Result<ast::Object> {
        return match self.scanner.peek() {
            Some('\'') => self.parse_string(),
            Some('!') => self.parse_interpolated_anchor(),
            Some('$' | 'a'..='z' | 'A'..='Z' | '_') => self.parse_name_or_global_function(),
            Some(char) => Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
            None => Err(anyhow::anyhow!("unexpected end")),
        };
    }

    fn next_expression(&mut self) -> Result<ast::Expression> {
        let result = match self.scanner.peek() {
            Some('(') => {
                self.scanner.pop();
                self.scanner.skip_whitespace();
                let expression = self.next_expression()?;
                self.scanner.skip_whitespace();
                if self.scanner.pop() != Some(&')') {
                    return Err(anyhow::anyhow!("unclosed bracket"));
                }
                ast::Expression::BracketedExpression(Box::new(expression))
            }
            Some('0'..='9') => self.parse_number()?,
            _ => {
                let sign = match self.scanner.pop() {
                    Some('+') => ast::Sign::Plus,
                    Some('-') => ast::Sign::Minus,
                    Some(char) => return Err(anyhow::anyhow!("unexpected char \"{}\"", char)),
                    _ => return Err(anyhow::anyhow!("unexpected end")),
                };
                self.scanner.skip_whitespace();
                match self.next_expression()? {
                    ast::Expression::SignedExpression(_, _) => {
                        return Err(anyhow::anyhow!("duplicate sign"));
                    }
                    expression => ast::Expression::SignedExpression(Box::new(expression), sign),
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
                    let expression = self.next_expression()?;
                    self.resolve_binary_operation_precidence(result, operation, expression)
                }
                None => result,
            },
        );
    }

    fn parse_number(&mut self) -> Result<ast::Expression> {
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
                _ => return Ok(ast::Expression::Number(result)),
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
            ast::Expression::BinaryOperation(left, right_operation, right)
                if left_operation <= right_operation =>
            {
                ast::Expression::BinaryOperation(
                    Box::new(self.resolve_binary_operation_precidence(
                        left_expression,
                        left_operation,
                        *left,
                    )),
                    right_operation,
                    right,
                )
            }
            _ => ast::Expression::BinaryOperation(
                Box::new(left_expression),
                left_operation,
                Box::new(right_expression),
            ),
        }
    }

    fn parse_string(&mut self) -> Result<ast::Object> {
        let mut result = String::new();
        let mut interpolations = Vec::new();
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
                    return Ok(ast::Object::String(result.clone(), interpolations));
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
        if !self.scanner.take(&'$') {
            return Err(anyhow::anyhow!("expected interpolation"));
        }
        if !self.scanner.take(&'{') {
            return Err(anyhow::anyhow!("expected interpolation"));
        }
        self.scanner.skip_whitespace();
        let result = self.next_object()?;
        self.scanner.skip_whitespace();
        return match self.scanner.take(&'}') {
            true => Ok(ast::Interpolation { content: result }),
            false => Err(anyhow::anyhow!("unclosed interpolation")),
        };
    }

    fn parse_interpolated_anchor(&mut self) -> Result<ast::Object> {
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
            true => Ok(ast::Object::Anchor(result)),
            false => Err(anyhow::anyhow!("unclosed interpolated anchor")),
        };
    }

    fn parse_name_or_global_function(&mut self) -> Result<ast::Object> {
        let name = self.parse_word()?;
        self.scanner.skip_whitespace();
        if name.name == "null" && name.interpolations.len() == 0 {
            return Ok(ast::Object::Null);
        }
        let mut result = match self.scanner.peek() {
            Some(&'(') => {
                let arguments = self.parse_function_arguments()?;
                self.scanner.skip_whitespace();
                ast::Object::Function(name, arguments)
            }
            _ => ast::Object::Name(name),
        };
        loop {
            match self.scanner.peek() {
                Some('[') => {
                    self.scanner.pop();
                    self.scanner.skip_whitespace();
                    let expression = self.next_expression()?;
                    self.scanner.skip_whitespace();
                    match self.scanner.take(&']') {
                        true => result = ast::Object::ArrayAccess(Box::new(result), expression),
                        false => return Err(anyhow::anyhow!("unclosed array access")),
                    }
                }
                Some('.') => {
                    self.scanner.pop();
                    self.scanner.skip_whitespace();
                    let name = self.parse_word()?;
                    self.scanner.skip_whitespace();
                    result = match self.scanner.peek() {
                        Some('(') => {
                            let arguments = self.parse_function_arguments()?;
                            self.scanner.skip_whitespace();
                            ast::Object::MethodAccess(Box::new(result), name, arguments)
                        }
                        _ => ast::Object::FieldAccess(Box::new(result), name),
                    }
                }
                _ => return Ok(result),
            }
        }
    }

    fn parse_word(&mut self) -> Result<ast::Word> {
        let mut result = String::new();
        let mut interpolations = Vec::new();
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
            arguments.push(self.next_object()?);
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
        Expression, ExpressionAst, Interpolation, Object, ObjectAst, Operation, Sign, Word,
    };

    #[test]
    fn test_parse_simple_string_object() {
        assert_eq!(
            parse_object("'test'"),
            ObjectAst {
                root: Object::String("test".to_string(), vec![])
            }
        );
    }

    #[test]
    fn test_parse_string_object_with_whitespace() {
        assert_eq!(
            parse_object("\t'test'   "),
            ObjectAst {
                root: Object::String("test".to_string(), vec![])
            }
        );
    }

    #[test]
    fn test_parse_escaped_string_object() {
        assert_eq!(
            parse_object("'tes\\\'t'"),
            ObjectAst {
                root: Object::String("tes\\\'t".to_string(), vec![])
            }
        );
    }

    #[test]
    fn test_parse_null() {
        assert_eq!(parse_object("null"), ObjectAst { root: Object::Null });
    }

    #[test]
    fn test_parse_interpolated_null() {
        assert_eq!(
            parse_object("null${'notNull'}"),
            ObjectAst {
                root: Object::Name(Word {
                    name: "null".to_string(),
                    interpolations: vec![Interpolation {
                        content: Object::String("notNull".to_string(), vec![])
                    }],
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
    fn test_parse_interpolated_string_object() {
        assert_eq!(
            parse_object("'hello, ${world}'"),
            ObjectAst {
                root: Object::String(
                    "hello, ".to_string(),
                    vec![Interpolation {
                        content: Object::Name(Word {
                            name: "world".to_string(),
                            ..Default::default()
                        }),
                    }]
                )
            }
        );
    }

    #[test]
    fn test_parse_simple_global_function() {
        assert_eq!(
            parse_object("flush()"),
            ObjectAst {
                root: Object::Function(
                    Word {
                        name: "flush".to_string(),
                        ..Default::default()
                    },
                    vec![]
                )
            }
        );
    }

    #[test]
    fn test_parse_global_function_with_argument() {
        assert_eq!(
            parse_object("is_string('test')"),
            ObjectAst {
                root: Object::Function(
                    Word {
                        name: "is_string".to_string(),
                        ..Default::default()
                    },
                    vec![Object::String("test".to_string(), vec![]),]
                )
            }
        );
    }

    #[test]
    fn test_parse_interpolated_global_function() {
        assert_eq!(
            parse_object("is_${_type}()"),
            ObjectAst {
                root: Object::Function(
                    Word {
                        name: "is_".to_string(),
                        interpolations: vec![Interpolation {
                            content: Object::Name(Word {
                                name: "_type".to_string(),
                                ..Default::default()
                            }),
                        }]
                    },
                    vec![]
                )
            }
        );
    }

    #[test]
    fn test_parse_global_function_with_excessive_whitespace() {
        assert_eq!(
            parse_object("\tis_string (\t'test'  , 'test2' ) "),
            ObjectAst {
                root: Object::Function(
                    Word {
                        name: "is_string".to_string(),
                        ..Default::default()
                    },
                    vec![
                        Object::String("test".to_string(), vec![]),
                        Object::String("test2".to_string(), vec![])
                    ]
                )
            }
        );
    }

    #[test]
    fn test_parse_nested_global_function() {
        assert_eq!(
            parse_object("is_string(concat('hello', 'world'))"),
            ObjectAst {
                root: Object::Function(
                    Word {
                        name: "is_string".to_string(),
                        ..Default::default()
                    },
                    vec![Object::Function(
                        Word {
                            name: "concat".to_string(),
                            ..Default::default()
                        },
                        vec![
                            Object::String("hello".to_string(), vec![]),
                            Object::String("world".to_string(), vec![])
                        ]
                    )]
                )
            }
        );
    }

    #[test]
    fn test_parse_simple_field_access() {
        assert_eq!(
            parse_object("_string.length"),
            ObjectAst {
                root: Object::FieldAccess(
                    Box::new(Object::Name(Word {
                        name: "_string".to_string(),
                        ..Default::default()
                    })),
                    Word {
                        name: "length".to_string(),
                        ..Default::default()
                    }
                )
            }
        );
    }

    #[test]
    fn test_parse_simple_method_access() {
        assert_eq!(
            parse_object("_string.length()"),
            ObjectAst {
                root: Object::MethodAccess(
                    Box::new(Object::Name(Word {
                        name: "_string".to_string(),
                        ..Default::default()
                    })),
                    Word {
                        name: "length".to_string(),
                        ..Default::default()
                    },
                    vec![]
                )
            }
        );
    }

    #[test]
    fn test_parse_simple_array_access() {
        assert_eq!(
            parse_object("_strings[0]"),
            ObjectAst {
                root: Object::ArrayAccess(
                    Box::new(Object::Name(Word {
                        name: "_strings".to_string(),
                        ..Default::default()
                    })),
                    Expression::Number("0".to_string()),
                )
            }
        );
    }

    #[test]
    fn test_parse_signed_array_access() {
        assert_eq!(
            parse_object("_strings[+1]"),
            ObjectAst {
                root: Object::ArrayAccess(
                    Box::new(Object::Name(Word {
                        name: "_strings".to_string(),
                        ..Default::default()
                    })),
                    Expression::SignedExpression(
                        Box::new(Expression::Number("1".to_string())),
                        Sign::Plus
                    ),
                )
            }
        );
    }

    #[test]
    fn test_parse_simple_number() {
        assert_eq!(
            parse_expression("123"),
            ExpressionAst {
                root: Expression::Number("123".to_string())
            }
        );
    }

    #[test]
    fn test_parse_scientific_number() {
        assert_eq!(
            parse_expression("-13.5e-2"),
            ExpressionAst {
                root: Expression::SignedExpression(
                    Box::new(Expression::Number("13.5e-2".to_string())),
                    Sign::Minus
                )
            }
        );
    }

    #[test]
    fn test_parse_bracketed_number() {
        assert_eq!(
            parse_expression(" -( -6E+2 ) "),
            ExpressionAst {
                root: Expression::SignedExpression(
                    Box::new(Expression::BracketedExpression(Box::new(
                        Expression::SignedExpression(
                            Box::new(Expression::Number("6E+2".to_string())),
                            Sign::Minus
                        )
                    ))),
                    Sign::Minus
                )
            }
        );
    }

    #[test]
    fn test_parse_simple_addition() {
        assert_eq!(
            parse_expression("6 + 9"),
            ExpressionAst {
                root: Expression::BinaryOperation(
                    Box::new(Expression::Number("6".to_string())),
                    Operation::Addition,
                    Box::new(Expression::Number("9".to_string())),
                )
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
                root: Expression::BinaryOperation(
                    Box::new(Expression::BinaryOperation(
                        Box::new(Expression::Number("1".to_string())),
                        Operation::Addition,
                        Box::new(Expression::Number("2".to_string()))
                    )),
                    Operation::Addition,
                    Box::new(Expression::SignedExpression(
                        Box::new(Expression::Number("3".to_string())),
                        Sign::Minus
                    ))
                )
            }
        );
    }

    #[test]
    fn test_addition_has_priority_over_division() {
        assert_eq!(
            parse_expression("6 + 10 / 2"),
            ExpressionAst {
                root: Expression::BinaryOperation(
                    Box::new(Expression::Number("6".to_string())),
                    Operation::Addition,
                    Box::new(Expression::BinaryOperation(
                        Box::new(Expression::Number("10".to_string())),
                        Operation::Division,
                        Box::new(Expression::Number("2".to_string())),
                    ))
                )
            }
        );
    }

    #[test]
    fn test_binary_operation_precidences() {
        assert_eq!(
            parse_expression("1 + 2 / 3 * 4 ^ 5 % 6"),
            ExpressionAst {
                root: Expression::BinaryOperation(
                    Box::new(Expression::Number("1".to_string())),
                    Operation::Addition,
                    Box::new(Expression::BinaryOperation(
                        Box::new(Expression::BinaryOperation(
                            Box::new(Expression::BinaryOperation(
                                Box::new(Expression::Number("2".to_string())),
                                Operation::Division,
                                Box::new(Expression::Number("3".to_string()))
                            )),
                            Operation::Multiplication,
                            Box::new(Expression::BinaryOperation(
                                Box::new(Expression::Number("4".to_string())),
                                Operation::Power,
                                Box::new(Expression::Number("5".to_string()))
                            ))
                        )),
                        Operation::Modulo,
                        Box::new(Expression::Number("6".to_string())),
                    ))
                )
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
