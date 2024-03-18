use super::{
    super::{TOKEN_MODIFIERS, TOKEN_TYPES},
    LsError, ResponseErrorCode,
};
use crate::{
    document_store, grammar, parser,
    spel::parser::Parser,
};
use anyhow::Result;
use lsp_types::{SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensParams};
use std::str::FromStr;
use tree_sitter::Node;

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
    let tokens = &mut Vec::new();
    parse_document(document.tree.root_node(), &document.text, tokens).map_err(|err| {
        log::error!("semantic token parsing failed for {}: {}", uri, err);
        return LsError {
            message: format!("semantic token parsing failed for {}", uri),
            code: ResponseErrorCode::RequestFailed,
        };
    })?;
    // the tokens delta_line and delta_start are relative to each other!
    let mut prev: Option<SemanticToken> = None;
    let mut result = Vec::new();
    for token in tokens.iter() {
        match prev {
            None => {
                result.push(*token);
            }
            Some(prev_token) => {
                let delta_line = token.delta_line - prev_token.delta_line;
                let delta_start;
                if delta_line == 0 {
                    delta_start = token.delta_start - prev_token.delta_start;
                } else {
                    delta_start = token.delta_start;
                }
                result.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length: token.length,
                    token_type: token.token_type,
                    token_modifiers_bitset: token.token_modifiers_bitset,
                });
            }
        }
        prev = Some(*token);
    }
    return Ok(result);
}

fn parse_document(root: Node, text: &String, tokens: &mut Vec<SemanticToken>) -> Result<()> {
    for node in root.children(&mut root.walk()) {
        match node.kind() {
            "page_header" | "import_header" | "taglib_header" | "html_doctype" | "text"
            | "comment" | "xml_entity" => continue,
            "html_tag" | "html_option_tag" | "html_void_tag" | "xml_comment" | "java_tag"
            | "script_tag" | "style_tag" => parse_children(node, &text, tokens)?,
            _ => match &grammar::Tag::from_str(node.kind()) {
                Ok(tag) => parse_tag(tag.properties(), node, &text, tokens)?,
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

fn parse_tag(
    tag: grammar::TagProperties,
    node: Node,
    text: &String,
    tokens: &mut Vec<SemanticToken>,
) -> Result<()> {
    if tag.deprecated {
        tokens.push(create_token(
            node,
            SemanticTokenType::MACRO,
            vec![SemanticTokenModifier::DEPRECATED],
        ));
    }
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            // may need to check on kind of missing child
            "html_void_tag" | "java_tag" | "script_tag" | "style_tag" => {}
            "ERROR" | "html_tag" | "html_option_tag" => parse_children(child, text, tokens)?,
            kind if kind.ends_with("_attribute") => {
                let attribute = parser::attribute_name_of(child, text).to_string();
                if let grammar::TagAttributes::These(definitions) = tag.attributes {
                    if let Some(definition) = definitions
                        .iter()
                        .find(|definition| definition.name == attribute)
                    {
                        let value_node = child
                            .child(2)
                            .expect(
                                format!(
                                    "attribute {:?} did not have a attribute-value child",
                                    attribute
                                )
                                .as_str(),
                            )
                            .child(1)
                            .expect(
                                format!(
                                    "attribute {:?} did not have a child in its attribute-value",
                                    attribute
                                )
                                .as_str(),
                            );
                        let parser = &mut Parser::new(value_node.utf8_text(&text.as_bytes())?);
                        match definition.r#type {
                            grammar::TagAttributeType::Condition => {}
                            grammar::TagAttributeType::Expression => match parser.parse_expression_ast() {
                                Ok(_result) => tokens.push(create_token(
                                    value_node,
                                    SemanticTokenType::NUMBER,
                                    vec![]
                                )),
                                Err(err) => {
                                    log::error!(
                                        "unparsable expression \"{}\": {}",
                                        value_node.utf8_text(&text.as_bytes())?,
                                        err
                                    );
                                }
                            }
                            grammar::TagAttributeType::Identifier => match parser.parse_identifier() {
                                Ok(_result) => tokens.push(create_token(
                                    value_node,
                                    SemanticTokenType::VARIABLE,
                                    vec![]
                                )),
                                Err(err) => {
                                    log::error!(
                                        "unparsable identifier \"{}\": {}",
                                        value_node.utf8_text(&text.as_bytes())?,
                                        err
                                    );
                                }
                            }
                            grammar::TagAttributeType::Object => match parser.parse_object_ast() {
                                Ok(_result) => tokens.push(create_token(
                                    value_node,
                                    SemanticTokenType::VARIABLE,
                                    vec![]
                                )),
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
                Ok(child_tag) => parse_tag(child_tag.properties(), child, text, tokens)?,
                Err(err) => {
                    log::info!("expected sp or spt tag: {}", err);
                }
            },
            _ => parse_children(child, text, tokens)?,
        }
    }
    return Ok(());
}

fn parse_object(node: Node, tokens: &mut Vec<SemanticToken>) -> Result<()> {
    match node.kind() {
        "object" => tokens.push(create_token(node, SemanticTokenType::VARIABLE, Vec::new())),
        "number" => tokens.push(create_token(node, SemanticTokenType::NUMBER, Vec::new())),
        "boolean" => tokens.push(create_token(node, SemanticTokenType::ENUM, Vec::new())),
        "string" => {
            tokens.push(create_token(node, SemanticTokenType::STRING, Vec::new()));
            for child in node.children(&mut node.walk()) {
                parse_object(child, tokens)?;
            }
        }
        "field_access" => {
            parse_object(node.child(0).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(1).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            let field = node.child(2).unwrap();
            match field.child(0) {
                Some(child) => parse_object(child, tokens)?,
                _ => tokens.push(create_token(field, SemanticTokenType::PROPERTY, Vec::new())),
            };
        }
        "method_access" => {
            parse_object(node.child(0).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(1).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_function(node.child(2).unwrap(), tokens)?;
        }
        "array_offset" => {
            parse_object(node.child(0).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(1).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_expression(node.child(2).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(3).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
        }
        "interpolated_string" => {
            tokens.push(create_token(
                node.child(0).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_object(node.child(1).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(2).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
        }
        "interpolated_anchor" => {
            for child in node.children(&mut node.walk()) {
                match child.kind() {
                    "!{" | "}" => {
                        tokens.push(create_token(child, SemanticTokenType::OPERATOR, Vec::new()))
                    }
                    _ => parse_object(child, tokens)?,
                }
            }
        }
        "global_function" => parse_function(node, tokens)?,
        _ => {}
    }
    return Ok(());
}

fn parse_function(node: Node, tokens: &mut Vec<SemanticToken>) -> Result<()> {
    for argument in node.children(&mut node.walk()) {
        match argument.kind() {
            "function_name" => tokens.push(create_token(
                argument,
                SemanticTokenType::METHOD,
                Vec::new(),
            )),
            "argument" => parse_object(argument.child(0).unwrap(), tokens)?,
            "(" | "," | ")" => tokens.push(create_token(
                argument,
                SemanticTokenType::OPERATOR,
                Vec::new(),
            )),
            _ => {}
        }
    }
    return Ok(());
}

fn parse_expression(node: Node, tokens: &mut Vec<SemanticToken>) -> Result<()> {
    match node.kind() {
        "number" => tokens.push(create_token(node, SemanticTokenType::NUMBER, Vec::new())),
        "expression" => {
            parse_expression(node.child(0).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(1).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_expression(node.child(2).unwrap(), tokens)?;
        }
        "bracketed_expression" => {
            tokens.push(create_token(
                node.child(0).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_expression(node.child(1).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(2).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
        }
        "unary_expression" => {
            tokens.push(create_token(
                node.child(0).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_expression(node.child(1).unwrap(), tokens)?;
        }
        "ternary_expression" => {
            parse_condition(node.child(0).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(1).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_expression(node.child(2).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(3).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_expression(node.child(4).unwrap(), tokens)?;
        }
        "interpolated_string" => {
            tokens.push(create_token(
                node.child(0).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_object(node.child(1).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(2).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
        }
        _ => {}
    }
    return Ok(());
}

fn parse_condition(node: Node, tokens: &mut Vec<SemanticToken>) -> Result<()> {
    match node.kind() {
        "boolean" => tokens.push(create_token(node, SemanticTokenType::ENUM, Vec::new())),
        "condition" => {
            parse_condition(node.child(0).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(1).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_condition(node.child(2).unwrap(), tokens)?;
        }
        "bracketed_condition" => {
            tokens.push(create_token(
                node.child(0).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_condition(node.child(1).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(2).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
        }
        "equality_comparison" => {
            parse_object(node.child(0).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(1).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_object(node.child(2).unwrap(), tokens)?;
        }
        "expression_comparison" => {
            parse_expression(node.child(0).unwrap(), tokens)?;
            tokens.push(create_token(
                node.child(1).unwrap(),
                SemanticTokenType::OPERATOR,
                Vec::new(),
            ));
            parse_expression(node.child(2).unwrap(), tokens)?;
        }
        _ => {}
    }
    return Ok(());
}

fn parse_children(node: Node, text: &String, tokens: &mut Vec<SemanticToken>) -> Result<()> {
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "text" | "java_tag" | "html_void_tag" => {}
            "ERROR" | "html_tag" | "html_option_tag" | "script_tag" | "style_tag" => {
                parse_children(child, text, tokens)?;
            }
            kind if kind.ends_with("_tag") => match &grammar::Tag::from_str(kind) {
                Ok(child_tag) => parse_tag(child_tag.properties(), child, text, tokens)?,
                Err(err) => {
                    log::info!("expected sp or spt tag: {}", err);
                }
            },
            _ => parse_children(child, text, tokens)?,
        }
    }
    return Ok(());
}

fn create_token(
    node: Node,
    r#type: SemanticTokenType,
    modifiers: Vec<SemanticTokenModifier>,
) -> SemanticToken {
    return SemanticToken {
        delta_line: node.start_position().row as u32,
        delta_start: node.start_position().column as u32,
        length: (node.end_byte() - node.start_byte()) as u32,
        token_type: TOKEN_TYPES
            .iter()
            .enumerate()
            .find_map(|(index, token_type)| match *token_type == r#type {
                true => Some(index as u32),
                false => None,
            })
            .expect("no function token exists"),
        token_modifiers_bitset: TOKEN_MODIFIERS
            .iter()
            .enumerate()
            .filter_map(|(index, modifier)| match modifiers.contains(modifier) {
                true => Some(1 << index as u32),
                false => None,
            })
            .sum::<u32>(),
    };
}
