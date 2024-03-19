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
        r#type: SemanticTokenType,
        modifiers: Vec<SemanticTokenModifier>,
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
        r#type: SemanticTokenType,
        modifiers: Vec<SemanticTokenModifier>,
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
                .find_map(|(index, token_type)| match *token_type == r#type {
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
        r#type: SemanticTokenType,
        modifiers: Vec<SemanticTokenModifier>,
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
            SemanticTokenType::MACRO,
            vec![SemanticTokenModifier::DEPRECATED],
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
                            grammar::TagAttributeType::Condition => {}
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
                                    Ok(_result) => tokenizer.add_node(
                                        value_node,
                                        SemanticTokenType::VARIABLE,
                                        vec![],
                                    ),
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
                            grammar::TagAttributeType::String => {}
                            grammar::TagAttributeType::Query => {}
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

fn index_object(node: &ast::Object, tokenizer: &mut SpelTokenCollector) {
    // TODO: index_word
    match node {
        ast::Object::Anchor {
            opening_bracket_location,
            closing_bracket_location,
            ..
        } => {
            tokenizer.add(
                opening_bracket_location,
                SemanticTokenType::OPERATOR,
                vec![],
            );
            // index_object(name, tokens);
            tokenizer.add(
                closing_bracket_location,
                SemanticTokenType::OPERATOR,
                vec![],
            );
        }
        ast::Object::Function {
            name,
            arguments,
            opening_bracket_location,
            closing_bracket_location,
        } => {
            tokenizer.add(&name.location, SemanticTokenType::METHOD, Vec::new());
            tokenizer.add(
                opening_bracket_location,
                SemanticTokenType::OPERATOR,
                vec![],
            );
            arguments
                .iter()
                .for_each(|arg| index_object(arg, tokenizer));
            tokenizer.add(
                closing_bracket_location,
                SemanticTokenType::OPERATOR,
                vec![],
            );
        }
        ast::Object::Name { name, .. } => {
            tokenizer.add(&name.location, SemanticTokenType::VARIABLE, vec![])
        }
        ast::Object::Null { location } => {
            tokenizer.add(location, SemanticTokenType::VARIABLE, vec![])
        }
        ast::Object::String { location, .. } => {
            tokenizer.add(location, SemanticTokenType::STRING, vec![])
        }
        // "number" => tokens.push(create_token(node, SemanticTokenType::NUMBER, Vec::new())),
        // "boolean" => tokens.push(create_token(node, SemanticTokenType::ENUM, Vec::new())),
        ast::Object::FieldAccess {
            object,
            field,
            dot_location,
        } => {
            index_object(object, tokenizer);
            tokenizer.add(dot_location, SemanticTokenType::OPERATOR, vec![]);
            tokenizer.add(&field.location, SemanticTokenType::PROPERTY, Vec::new());
        }
        ast::Object::MethodAccess {
            object,
            method,
            arguments,
            dot_location,
            opening_bracket_location,
            closing_bracket_location,
        } => {
            index_object(object, tokenizer);
            tokenizer.add(dot_location, SemanticTokenType::OPERATOR, vec![]);
            tokenizer.add(&method.location, SemanticTokenType::METHOD, Vec::new());
            tokenizer.add(
                opening_bracket_location,
                SemanticTokenType::OPERATOR,
                vec![],
            );
            arguments
                .iter()
                .for_each(|arg| index_object(arg, tokenizer));
            tokenizer.add(
                closing_bracket_location,
                SemanticTokenType::OPERATOR,
                vec![],
            );
        }
        ast::Object::ArrayAccess {
            object,
            index,
            opening_bracket_location,
            closing_bracket_location,
        } => {
            index_object(object, tokenizer);
            tokenizer.add(
                opening_bracket_location,
                SemanticTokenType::OPERATOR,
                vec![],
            );
            index_expression(index, tokenizer);
            tokenizer.add(
                closing_bracket_location,
                SemanticTokenType::OPERATOR,
                vec![],
            );
        } // "interpolated_string" => {
          //     tokens.push(create_token(
          //         node.child(0).unwrap(),
          //         SemanticTokenType::OPERATOR,
          //         Vec::new(),
          //     ));
          //     index_object(node.child(1).unwrap(), tokens)?;
          //     tokens.push(create_token(
          //         node.child(2).unwrap(),
          //         SemanticTokenType::OPERATOR,
          //         Vec::new(),
          //     ));
          // }
    }
}

fn index_expression(node: &ast::Expression, tokenizer: &mut SpelTokenCollector) {
    match node {
        ast::Expression::Number { location, .. } => {
            tokenizer.add(location, SemanticTokenType::NUMBER, vec![]);
        }
        ast::Expression::SignedExpression { expression, sign } => {
            tokenizer.add(&sign.location(), SemanticTokenType::OPERATOR, vec![]);
            index_expression(expression, tokenizer);
        }
        ast::Expression::BinaryOperation {
            left,
            right,
            operation_location,
            ..
        } => {
            index_expression(left, tokenizer);
            tokenizer.add(operation_location, SemanticTokenType::OPERATOR, vec![]);
            index_expression(right, tokenizer);
        }
        ast::Expression::BracketedExpression {
            expression,
            opening_bracket_location,
            closing_bracket_location,
        } => {
            tokenizer.add(
                opening_bracket_location,
                SemanticTokenType::OPERATOR,
                vec![],
            );
            index_expression(expression, tokenizer);
            tokenizer.add(
                closing_bracket_location,
                SemanticTokenType::OPERATOR,
                vec![],
            );
        } // "ternary_expression" => {
          //     index_condition(node.child(0).unwrap(), tokens)?;
          //     tokens.push(create_token(
          //         node.child(1).unwrap(),
          //         SemanticTokenType::OPERATOR,
          //         Vec::new(),
          //     ));
          //     index_expression(node.child(2).unwrap(), tokens)?;
          //     tokens.push(create_token(
          //         node.child(3).unwrap(),
          //         SemanticTokenType::OPERATOR,
          //         Vec::new(),
          //     ));
          //     index_expression(node.child(4).unwrap(), tokens)?;
          // }
          // "interpolated_string" => {
          //     tokens.push(create_token(
          //         node.child(0).unwrap(),
          //         SemanticTokenType::OPERATOR,
          //         Vec::new(),
          //     ));
          //     index_object(node.child(1).unwrap(), tokens)?;
          //     tokens.push(create_token(
          //         node.child(2).unwrap(),
          //         SemanticTokenType::OPERATOR,
          //         Vec::new(),
          //     ));
          // }
    }
}

// fn index_condition(node: Node, tokens: &mut Vec<SemanticToken>) -> Result<()> {
//     match node.kind() {
//         "boolean" => tokens.push(create_token(node, SemanticTokenType::ENUM, Vec::new())),
//         "condition" => {
//             index_condition(node.child(0).unwrap(), tokens)?;
//             tokens.push(create_token(
//                 node.child(1).unwrap(),
//                 SemanticTokenType::OPERATOR,
//                 Vec::new(),
//             ));
//             index_condition(node.child(2).unwrap(), tokens)?;
//         }
//         "bracketed_condition" => {
//             tokens.push(create_token(
//                 node.child(0).unwrap(),
//                 SemanticTokenType::OPERATOR,
//                 Vec::new(),
//             ));
//             index_condition(node.child(1).unwrap(), tokens)?;
//             tokens.push(create_token(
//                 node.child(2).unwrap(),
//                 SemanticTokenType::OPERATOR,
//                 Vec::new(),
//             ));
//         }
//         "equality_comparison" => {
//             index_object(node.child(0).unwrap(), tokens)?;
//             tokens.push(create_token(
//                 node.child(1).unwrap(),
//                 SemanticTokenType::OPERATOR,
//                 Vec::new(),
//             ));
//             index_object(node.child(2).unwrap(), tokens)?;
//         }
//         "expression_comparison" => {
//             index_expression(node.child(0).unwrap(), tokens)?;
//             tokens.push(create_token(
//                 node.child(1).unwrap(),
//                 SemanticTokenType::OPERATOR,
//                 Vec::new(),
//             ));
//             index_expression(node.child(2).unwrap(), tokens)?;
//         }
//         _ => {}
//     }
//     return Ok(());
// }

fn index_children(node: Node, text: &String, tokens: &mut Tokenizer) -> Result<()> {
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "text" | "java_tag" | "html_void_tag" => {}
            "ERROR" | "html_tag" | "html_option_tag" | "script_tag" | "style_tag" => {
                index_children(child, text, tokens)?;
            }
            kind if kind.ends_with("_tag") => match &grammar::Tag::from_str(kind) {
                Ok(child_tag) => index_tag(child_tag.properties(), child, text, tokens)?,
                Err(err) => {
                    log::info!("expected sp or spt tag: {}", err);
                }
            },
            _ => index_children(child, text, tokens)?,
        }
    }
    return Ok(());
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
                name: "_someVariable".to_string(),
                interpolations: vec![],
                location: ast::Location::VariableLength {
                    char: 0,
                    line: 0,
                    length: 13,
                },
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
        let root_object1 = ast::Object::Name {
            name: ast::Word {
                name: "_someVariable".to_string(),
                interpolations: vec![],
                location: ast::Location::VariableLength {
                    char: 0,
                    line: 0,
                    length: 13,
                },
            },
        };
        let root_object2 = ast::Object::Name {
            name: ast::Word {
                name: "_someVariable".to_string(),
                interpolations: vec![],
                location: ast::Location::VariableLength {
                    char: 0,
                    line: 0,
                    length: 13,
                },
            },
        };
        crate::command::semantics::index_object(
            &root_object1,
            &mut SpelTokenCollector {
                tokenizer,
                offset_line: 8,
                offset_char: 14,
            },
        );
        crate::command::semantics::index_object(
            &root_object2,
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
