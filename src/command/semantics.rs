use super::{
    super::{TOKEN_MODIFIERS, TOKEN_TYPES},
    LsError, ResponseErrorCode,
};
use crate::{
    document_store, grammar, parser,
    spel::{ast, parser::Parser},
};
use anyhow::Result;
use lsp_types::{SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensParams};
use std::str::FromStr;
use tree_sitter::Node;

#[derive(Debug, PartialEq)]
struct Tokenizer {
    tokens: Vec<SemanticToken>,
    cursor_line: u32,
    cursor_char: u32,
}

#[derive(Debug, PartialEq)]
struct SpelTokenCollector<'a> {
    tokenizer: &'a mut Tokenizer,
    offset_line: u32,
    offset_char: u32,
}

impl Tokenizer {
    fn new() -> Self {
        return Tokenizer {
            tokens: vec![],
            cursor_line: 0,
            cursor_char: 0,
        };
    }

    fn add_node(
        &mut self,
        node: Node,
        r#type: &SemanticTokenType,
        modifiers: &Vec<SemanticTokenModifier>,
    ) {
        self.add(
            node.start_position().column as u32,
            node.start_position().row as u32,
            (node.end_byte() - node.start_byte()) as u32,
            r#type,
            modifiers,
        );
    }

    fn add(
        &mut self,
        char: u32,
        line: u32,
        length: u32,
        r#type: &SemanticTokenType,
        modifiers: &Vec<SemanticTokenModifier>,
    ) {
        // delta_line and delta_start are relative to each other
        let delta_line = line - self.cursor_line;
        let delta_start = match delta_line {
            0 => char - self.cursor_char,
            _ => char,
        };
        self.tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type: TOKEN_TYPES
                .iter()
                .enumerate()
                .find_map(|(index, token_type)| match token_type == r#type {
                    true => Some(index as u32),
                    false => None,
                })
                .expect(&format!("no token type \"{}\" found", r#type.as_str())),
            token_modifiers_bitset: TOKEN_MODIFIERS
                .iter()
                .enumerate()
                .filter_map(|(index, modifier)| match modifiers.contains(modifier) {
                    true => Some(1 << index as u32),
                    false => None,
                })
                .sum::<u32>(),
        });
        self.cursor_line = line;
        self.cursor_char = char;
    }

    fn collect(&self) -> Vec<SemanticToken> {
        return self.tokens.to_vec();
    }
}

impl SpelTokenCollector<'_> {
    fn new(
        tokenizer: &'_ mut Tokenizer,
        offset_line: u32,
        offset_char: u32,
    ) -> SpelTokenCollector<'_> {
        return SpelTokenCollector {
            tokenizer,
            offset_line,
            offset_char,
        };
    }

    fn add(
        &mut self,
        location: &ast::Location,
        r#type: &SemanticTokenType,
        modifiers: &Vec<SemanticTokenModifier>,
    ) {
        let char = location.char() as u32;
        let line = location.line() as u32;
        self.tokenizer.add(
            match line {
                0 => self.offset_char + char,
                _ => char,
            },
            self.offset_line + line as u32,
            location.len() as u32,
            r#type,
            modifiers,
        );
    }
}

/**
 * this adds highlighting details for small tokens - not the entire file!
 */
pub(crate) fn semantics(params: SemanticTokensParams) -> Result<Vec<SemanticToken>, LsError> {
    let uri = params.text_document.uri;
    let document = match document_store::get(&uri) {
        Some(document) => Ok(document),
        None => document_store::Document::new(&uri)
            .map(|document| document_store::put(&uri, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", uri, err);
                return LsError {
                    message: format!("cannot read file {}", uri),
                    code: ResponseErrorCode::RequestFailed,
                };
            }),
    }?;
    let tokenizer = &mut Tokenizer::new();
    index_document(document.tree.root_node(), &document.text, tokenizer).map_err(|err| {
        log::error!("semantic token parsing failed for {}: {}", uri, err);
        return LsError {
            message: format!("semantic token parsing failed for {}", uri),
            code: ResponseErrorCode::RequestFailed,
        };
    })?;
    return Ok(tokenizer.collect());
}

fn index_document(root: Node, text: &String, tokenizer: &mut Tokenizer) -> Result<()> {
    for node in root.children(&mut root.walk()) {
        match node.kind() {
            "page_header" | "import_header" | "taglib_header" | "html_doctype" | "text"
            | "comment" | "xml_entity" => continue,
            "html_tag" | "html_option_tag" | "html_void_tag" | "xml_comment" | "java_tag"
            | "script_tag" | "style_tag" => index_children(node, &text, tokenizer)?,
            _ => match &grammar::Tag::from_str(node.kind()) {
                Ok(tag) => index_tag(tag.properties(), node, &text, tokenizer)?,
                Err(err) => log::info!(
                    "error while trying to interprete node \"{}\" as tag: {}",
                    node.kind(),
                    err
                ),
            },
        }
    }
    return Ok(());
}

fn index_tag(
    tag: grammar::TagProperties,
    node: Node,
    text: &String,
    tokenizer: &mut Tokenizer,
) -> Result<()> {
    if tag.deprecated {
        tokenizer.add_node(
            node,
            &SemanticTokenType::MACRO,
            &vec![SemanticTokenModifier::DEPRECATED],
        );
    }
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            // may need to check on kind of missing child
            "html_void_tag" | "java_tag" | "script_tag" | "style_tag" => {}
            "ERROR" | "html_tag" | "html_option_tag" => index_children(child, text, tokenizer)?,
            kind if kind.ends_with("_attribute") => {
                let attribute = parser::attribute_name_of(child, text).to_string();
                if let grammar::TagAttributes::These(definitions) = tag.attributes {
                    if let Some(definition) = definitions
                        .iter()
                        .find(|definition| definition.name == attribute)
                    {
                        let value_node = match child.child(2).and_then(|child| child.child(1)) {
                            Some(node) => node,
                            _ => continue,
                        };
                        let parser = &mut Parser::new(value_node.utf8_text(&text.as_bytes())?);
                        match definition.r#type {
                            grammar::TagAttributeType::Comparable => {
                                match parser.parse_comparable() {
                                    Ok(result) => {
                                        let position = value_node.start_position();
                                        index_comparable(
                                            &result,
                                            &mut SpelTokenCollector::new(
                                                tokenizer,
                                                position.row as u32,
                                                position.column as u32,
                                            ),
                                        );
                                    }
                                    Err(err) => {
                                        log::error!(
                                            "unparsable comparable \"{}\": {}",
                                            value_node.utf8_text(&text.as_bytes())?,
                                            err
                                        );
                                    }
                                }
                            }
                            grammar::TagAttributeType::Condition => {
                                match parser.parse_condition_ast() {
                                    Ok(result) => {
                                        let position = value_node.start_position();
                                        index_condition(
                                            &result.root,
                                            &mut SpelTokenCollector::new(
                                                tokenizer,
                                                position.row as u32,
                                                position.column as u32,
                                            ),
                                        );
                                    }
                                    Err(err) => {
                                        log::error!(
                                            "unparsable condition \"{}\": {}",
                                            value_node.utf8_text(&text.as_bytes())?,
                                            err
                                        );
                                    }
                                }
                            }
                            grammar::TagAttributeType::Expression => {
                                match parser.parse_expression_ast() {
                                    Ok(result) => {
                                        let position = value_node.start_position();
                                        index_expression(
                                            &result.root,
                                            &mut SpelTokenCollector::new(
                                                tokenizer,
                                                position.row as u32,
                                                position.column as u32,
                                            ),
                                        );
                                    }
                                    Err(err) => {
                                        log::error!(
                                            "unparsable expression \"{}\": {}",
                                            value_node.utf8_text(&text.as_bytes())?,
                                            err
                                        );
                                    }
                                }
                            }
                            grammar::TagAttributeType::Identifier => {
                                match parser.parse_identifier() {
                                    Ok(result) => {
                                        let position = value_node.start_position();
                                        index_identifier(
                                            &result,
                                            &mut SpelTokenCollector::new(
                                                tokenizer,
                                                position.row as u32,
                                                position.column as u32,
                                            ),
                                        );
                                    }
                                    Err(err) => {
                                        log::error!(
                                            "unparsable identifier \"{}\": {}",
                                            value_node.utf8_text(&text.as_bytes())?,
                                            err
                                        );
                                    }
                                }
                            }
                            grammar::TagAttributeType::Object => match parser.parse_object_ast() {
                                Ok(result) => {
                                    let position = value_node.start_position();
                                    index_object(
                                        &result.root,
                                        &mut SpelTokenCollector::new(
                                            tokenizer,
                                            position.row as u32,
                                            position.column as u32,
                                        ),
                                    );
                                }
                                Err(err) => {
                                    log::error!(
                                        "unparsable object \"{}\": {}",
                                        value_node.utf8_text(&text.as_bytes())?,
                                        err
                                    );
                                }
                            },
                            grammar::TagAttributeType::Regex => {}
                            grammar::TagAttributeType::String => match parser.parse_text() {
                                Ok(result) => {
                                    let position = value_node.start_position();
                                    for fragment in result.fragments {
                                        if let ast::WordFragment::Interpolation(interpolation) =
                                            fragment
                                        {
                                            index_interpolation(
                                                &interpolation,
                                                &mut SpelTokenCollector::new(
                                                    tokenizer,
                                                    position.row as u32,
                                                    position.column as u32,
                                                ),
                                            );
                                        }
                                    }
                                }
                                Err(err) => {
                                    log::error!(
                                        "unparsable text \"{}\": {}",
                                        value_node.utf8_text(&text.as_bytes())?,
                                        err
                                    );
                                }
                            },
                            grammar::TagAttributeType::Query => {}
                            grammar::TagAttributeType::Uri => match parser.parse_uri() {
                                Ok(result) => {
                                    let position = value_node.start_position();
                                    index_uri(
                                        &result,
                                        &mut SpelTokenCollector::new(
                                            tokenizer,
                                            position.row as u32,
                                            position.column as u32,
                                        ),
                                    );
                                }
                                Err(err) => {
                                    log::error!(
                                        "unparsable text \"{}\": {}",
                                        value_node.utf8_text(&text.as_bytes())?,
                                        err
                                    );
                                }
                            },
                        }
                    };
                }
            }
            kind if kind.ends_with("_tag") => match &grammar::Tag::from_str(kind) {
                Ok(child_tag) => index_tag(child_tag.properties(), child, text, tokenizer)?,
                Err(err) => {
                    log::info!("expected sp or spt tag: {}", err);
                }
            },
            _ => index_children(child, text, tokenizer)?,
        }
    }
    return Ok(());
}

fn index_children(node: Node, text: &String, tokenizer: &mut Tokenizer) -> Result<()> {
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "text" | "java_tag" | "html_void_tag" => {}
            "ERROR" | "html_tag" | "html_option_tag" | "script_tag" | "style_tag" => {
                index_children(child, text, tokenizer)?;
            }
            kind if kind.ends_with("_tag") => match &grammar::Tag::from_str(kind) {
                Ok(child_tag) => index_tag(child_tag.properties(), child, text, tokenizer)?,
                Err(err) => {
                    log::info!("expected sp or spt tag: {}", err);
                }
            },
            _ => index_children(child, text, tokenizer)?,
        }
    }
    return Ok(());
}

fn index_identifier(identifier: &ast::Identifier, token_collector: &mut SpelTokenCollector) {
    match identifier {
        ast::Identifier::Name(name) => index_word(
            name,
            token_collector,
            Some(SemanticTokenType::VARIABLE),
            &vec![],
        ),
        ast::Identifier::FieldAccess {
            identifier,
            field,
            dot_location,
        } => {
            index_identifier(identifier, token_collector);
            token_collector.add(dot_location, &SemanticTokenType::OPERATOR, &vec![]);
            index_word(
                field,
                token_collector,
                Some(SemanticTokenType::VARIABLE),
                &vec![],
            );
        }
    };
}

fn index_object(object: &ast::Object, token_collector: &mut SpelTokenCollector) {
    match object {
        ast::Object::Anchor {
            name,
            opening_bracket_location,
            closing_bracket_location,
        } => {
            token_collector.add(
                opening_bracket_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
            index_word(
                name,
                token_collector,
                Some(SemanticTokenType::ENUM_MEMBER),
                &vec![],
            );
            token_collector.add(
                closing_bracket_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
        }
        ast::Object::Function(function) => index_function(function, token_collector),
        ast::Object::Name { name } => {
            index_word(
                name,
                token_collector,
                Some(SemanticTokenType::VARIABLE),
                &vec![],
            );
        }
        ast::Object::Null(ast::Null { location }) => {
            token_collector.add(location, &SemanticTokenType::ENUM_MEMBER, &vec![])
        }
        ast::Object::String(ast::StringLiteral { location, .. }) => {
            token_collector.add(location, &SemanticTokenType::STRING, &vec![])
        }
        ast::Object::FieldAccess {
            object,
            field,
            dot_location,
        } => {
            index_object(object, token_collector);
            token_collector.add(dot_location, &SemanticTokenType::OPERATOR, &vec![]);
            index_word(
                &field,
                token_collector,
                Some(SemanticTokenType::PROPERTY),
                &vec![],
            );
        }
        ast::Object::MethodAccess {
            object,
            dot_location,
            function,
        } => {
            index_object(object, token_collector);
            token_collector.add(dot_location, &SemanticTokenType::OPERATOR, &vec![]);
            index_function(function, token_collector);
        }
        ast::Object::ArrayAccess {
            object,
            index,
            opening_bracket_location,
            closing_bracket_location,
        } => {
            index_object(object, token_collector);
            token_collector.add(
                opening_bracket_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
            index_expression(index, token_collector);
            token_collector.add(
                closing_bracket_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
        }
    };
}

fn index_expression(expression: &ast::Expression, token_collector: &mut SpelTokenCollector) {
    match expression {
        ast::Expression::Number { location, .. } => {
            token_collector.add(location, &SemanticTokenType::NUMBER, &vec![]);
        }
        ast::Expression::Object(interpolation) => {
            token_collector.add(
                &interpolation.opening_bracket_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
            index_object(&interpolation.content, token_collector);
            token_collector.add(
                &interpolation.closing_bracket_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
        }
        ast::Expression::SignedExpression {
            expression,
            sign_location,
            ..
        } => {
            token_collector.add(&sign_location, &SemanticTokenType::OPERATOR, &vec![]);
            index_expression(expression, token_collector);
        }
        ast::Expression::BinaryOperation {
            left,
            right,
            operator_location: operation_location,
            ..
        } => {
            index_expression(left, token_collector);
            token_collector.add(operation_location, &SemanticTokenType::OPERATOR, &vec![]);
            index_expression(right, token_collector);
        }
        ast::Expression::BracketedExpression {
            expression,
            opening_bracket_location,
            closing_bracket_location,
        } => {
            token_collector.add(
                opening_bracket_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
            index_expression(expression, token_collector);
            token_collector.add(
                closing_bracket_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
        }
        ast::Expression::Ternary {
            condition,
            left,
            right,
            question_mark_location,
            colon_location,
        } => {
            index_condition(condition, token_collector);
            token_collector.add(
                question_mark_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
            index_expression(left, token_collector);
            token_collector.add(colon_location, &SemanticTokenType::OPERATOR, &vec![]);
            index_expression(right, token_collector);
        }
    };
}

fn index_condition(condition: &ast::Condition, token_collector: &mut SpelTokenCollector) {
    match condition {
        ast::Condition::True { location } | ast::Condition::False { location } => {
            token_collector.add(location, &SemanticTokenType::ENUM_MEMBER, &vec![])
        }
        ast::Condition::Object(location) => {
            token_collector.add(
                &location.opening_bracket_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
            index_object(&location.content, token_collector);
            token_collector.add(
                &location.closing_bracket_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
        }
        ast::Condition::BinaryOperation {
            left,
            right,
            operator_location,
            ..
        } => {
            index_condition(left, token_collector);
            token_collector.add(operator_location, &SemanticTokenType::OPERATOR, &vec![]);
            index_condition(right, token_collector);
        }
        ast::Condition::BracketedCondition {
            condition,
            opening_bracket_location,
            closing_bracket_location,
        } => {
            token_collector.add(
                opening_bracket_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
            index_condition(condition, token_collector);
            token_collector.add(
                closing_bracket_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
        }
        ast::Condition::NegatedCondition {
            condition,
            exclamation_mark_location,
        } => {
            token_collector.add(
                &exclamation_mark_location,
                &SemanticTokenType::OPERATOR,
                &vec![],
            );
            index_condition(condition, token_collector);
        }
        ast::Condition::Comparisson {
            left,
            right,
            operator_location,
            ..
        } => {
            index_comparable(left, token_collector);
            token_collector.add(&operator_location, &SemanticTokenType::OPERATOR, &vec![]);
            index_comparable(right, token_collector);
        }
    };
}

fn index_function(function: &ast::Function, token_collector: &mut SpelTokenCollector) {
    index_word(
        &function.name,
        token_collector,
        Some(SemanticTokenType::METHOD),
        &vec![],
    );
    token_collector.add(
        &function.opening_bracket_location,
        &SemanticTokenType::OPERATOR,
        &vec![],
    );
    for arg in function.arguments.iter() {
        index_object(&arg.object, token_collector);
        if let Some(comma_location) = &arg.comma_location {
            token_collector.add(&comma_location, &SemanticTokenType::OPERATOR, &vec![]);
        }
    }
    token_collector.add(
        &function.closing_bracket_location,
        &SemanticTokenType::OPERATOR,
        &vec![],
    );
}

fn index_uri(uri: &ast::Uri, token_collector: &mut SpelTokenCollector) {
    match uri {
        ast::Uri::Literal(literal) => {
            for fragment in &literal.fragments {
                token_collector.add(
                    &fragment.slash_location,
                    &SemanticTokenType::OPERATOR,
                    &vec![],
                );
                index_word(&fragment.content, token_collector, None, &vec![]);
            }
            if let Some(extension) = &literal.file_extension {
                token_collector.add(
                    &extension.dot_location,
                    &SemanticTokenType::OPERATOR,
                    &vec![],
                );
            }
        }
        ast::Uri::Object(object) => index_interpolation(object, token_collector),
    };
}
fn index_word(
    word: &ast::Word,
    token_collector: &mut SpelTokenCollector,
    token_type: Option<SemanticTokenType>,
    token_modifiers: &Vec<SemanticTokenModifier>,
) {
    for fragment in &word.fragments {
        match fragment {
            ast::WordFragment::String(ast::StringLiteral { location, .. }) => {
                if let Some(token_type) = &token_type {
                    token_collector.add(&location, token_type, token_modifiers);
                }
            }
            ast::WordFragment::Interpolation(interpolation) => {
                index_interpolation(&interpolation, token_collector)
            }
        }
    }
}

fn index_comparable(comparable: &ast::Comparable, token_collector: &mut SpelTokenCollector) {
    match comparable {
        ast::Comparable::Condition(condition) => index_condition(&condition, token_collector),
        ast::Comparable::Expression(expression) => index_expression(&expression, token_collector),
        ast::Comparable::Function(function) => index_function(&function, token_collector),
        ast::Comparable::Object(interpolation) => {
            index_interpolation(interpolation, token_collector);
        }
        ast::Comparable::String(ast::StringLiteral { location, .. }) => {
            token_collector.add(&location, &SemanticTokenType::STRING, &vec![])
        }
        ast::Comparable::Null(ast::Null { location }) => {
            token_collector.add(&location, &SemanticTokenType::VARIABLE, &vec![])
        }
    }
}

fn index_interpolation(
    interpolation: &ast::Interpolation,
    token_collector: &mut SpelTokenCollector,
) {
    token_collector.add(
        &interpolation.opening_bracket_location,
        &SemanticTokenType::OPERATOR,
        &vec![],
    );
    index_object(&interpolation.content, token_collector);
    token_collector.add(
        &interpolation.closing_bracket_location,
        &SemanticTokenType::OPERATOR,
        &vec![],
    );
}

#[cfg(test)]
mod tests {
    use lsp_types::{SemanticToken, SemanticTokenType};

    use crate::{
        command::semantics::{SpelTokenCollector, Tokenizer},
        spel::ast,
        TOKEN_TYPES,
    };

    #[test]
    fn test_index_single_object() {
        let tokenizer = &mut Tokenizer::new();
        let root_object = ast::Object::Name {
            name: ast::Word {
                fragments: vec![ast::WordFragment::String(ast::StringLiteral {
                    content: "_someVariable".to_string(),
                    location: ast::Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 13,
                    },
                })],
            },
        };
        crate::command::semantics::index_object(
            &root_object,
            &mut SpelTokenCollector {
                tokenizer,
                offset_line: 8,
                offset_char: 14,
            },
        );
        assert_eq!(
            tokenizer.collect(),
            vec![SemanticToken {
                delta_start: 14,
                delta_line: 8,
                length: 13,
                token_type: TOKEN_TYPES
                    .iter()
                    .enumerate()
                    .find_map(|(index, token_type)| {
                        match *token_type == SemanticTokenType::VARIABLE {
                            true => Some(index as u32),
                            false => None,
                        }
                    })
                    .expect("no variable token exists"),
                token_modifiers_bitset: 0,
            }],
        );
    }

    #[test]
    fn test_index_multiple_objects() {
        let tokenizer = &mut Tokenizer::new();
        let root_object = ast::Object::Name {
            name: ast::Word {
                fragments: vec![ast::WordFragment::String(ast::StringLiteral {
                    content: "_someVariable".to_string(),
                    location: ast::Location::VariableLength {
                        char: 0,
                        line: 0,
                        length: 13,
                    },
                })],
            },
        };
        crate::command::semantics::index_object(
            &root_object,
            &mut SpelTokenCollector {
                tokenizer,
                offset_line: 8,
                offset_char: 14,
            },
        );
        crate::command::semantics::index_object(
            &root_object,
            &mut SpelTokenCollector {
                tokenizer,
                offset_line: 9,
                offset_char: 14,
            },
        );
        assert_eq!(
            tokenizer.collect(),
            vec![
                SemanticToken {
                    delta_start: 14,
                    delta_line: 8,
                    length: 13,
                    token_type: TOKEN_TYPES
                        .iter()
                        .enumerate()
                        .find_map(|(index, token_type)| {
                            match *token_type == SemanticTokenType::VARIABLE {
                                true => Some(index as u32),
                                false => None,
                            }
                        })
                        .expect("no variable token exists"),
                    token_modifiers_bitset: 0,
                },
                SemanticToken {
                    delta_start: 14,
                    delta_line: 1,
                    length: 13,
                    token_type: TOKEN_TYPES
                        .iter()
                        .enumerate()
                        .find_map(|(index, token_type)| {
                            match *token_type == SemanticTokenType::VARIABLE {
                                true => Some(index as u32),
                                false => None,
                            }
                        })
                        .expect("no variable token exists"),
                    token_modifiers_bitset: 0,
                }
            ],
        );
    }
}
