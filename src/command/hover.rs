use std::str::FromStr;

use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind, Position};
use tree_sitter::{Node, Point};

use super::{LsError, ResponseErrorCode};

use crate::{
    document_store,
    grammar::{self, Tag, TagAttribute, TagAttributeType, TagAttributes},
    parser,
    spel::{
        self,
        ast::{self, Location},
        parser::Parser,
    },
};

pub(crate) fn hover(params: HoverParams) -> Result<Option<Hover>, LsError> {
    let text_params = params.text_document_position_params;
    let file = &text_params.text_document.uri;
    let document = match document_store::get(file) {
        Some(document) => Ok(document),
        None => document_store::Document::new(file)
            .map(|document| document_store::put(file, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", file, err);
                return LsError {
                    message: format!("cannot read file {}", file),
                    code: ResponseErrorCode::RequestFailed,
                };
            }),
    }?;
    let node =
        parser::find_current_node(&document.tree, text_params.position).ok_or_else(|| LsError {
            message: format!(
                "could not determine node in {} at line {}, character {}",
                file, text_params.position.line, text_params.position.character
            ),
            code: ResponseErrorCode::RequestFailed,
        })?;
    return Ok((match node.kind() {
        "string_content" => {
            let mut result = None;
            match node.parent().and_then(|p| p.parent()) {
                Some(attribute) => match node.utf8_text(&document.text.as_bytes()) {
                    Ok(text) => {
                        let parser = &mut Parser::new(text);
                        let cursor = text_params.position;
                        let offset = node.start_position();
                        match find_containing_tag(attribute)
                            .and_then(|tag| find_attribute_definition(tag, attribute))
                            .map(|definition| &definition.r#type)
                        {
                            Some(TagAttributeType::Condition) => {
                                match parser.parse_condition_ast() {
                                    Ok(ast) => match ast.root {
                                        ast::Condition::True { location }
                                            if location_contains_cursor(
                                                &location, &cursor, &offset,
                                            ) =>
                                        {
                                            result = Some("boolisches true".to_string());
                                        }
                                        ast::Condition::False { location }
                                            if location_contains_cursor(
                                                &location, &cursor, &offset,
                                            ) =>
                                        {
                                            result = Some("boolisches false".to_string());
                                        }
                                        // ast::Condition::Object(_) => todo!(),
                                        ast::Condition::Function(ast::Function {
                                            name,
                                            name_location,
                                            ..
                                        }) if location_contains_cursor(
                                            &name_location,
                                            &cursor,
                                            &offset,
                                        ) =>
                                        {
                                            if let Some(definition) =
                                                spel::grammar::Function::from_str(&name).ok()
                                            {
                                                result = Some(
                                                    definition.documentation.to_owned().to_string(),
                                                );
                                            }
                                        }
                                        // ast::Condition::BinaryOperation {
                                        //     left,
                                        //     operator,
                                        //     right,
                                        //     operator_location,
                                        // } => todo!(),
                                        // ast::Condition::BracketedCondition {
                                        //     condition,
                                        //     opening_bracket_location,
                                        //     closing_bracket_location,
                                        // } => todo!(),
                                        // ast::Condition::NegatedCondition {
                                        //     condition,
                                        //     exclamation_mark_location,
                                        // } => todo!(),
                                        // ast::Condition::Comparisson {
                                        //     left,
                                        //     operator,
                                        //     right,
                                        //     operator_location,
                                        // } => todo!(),
                                        _ => {}
                                    },
                                    Err(err) => log::info!(
                                        "no hover information about invalid condition: {}",
                                        err
                                    ),
                                }
                            }
                            Some(r#type) => {
                                log::info!(
                                    "no hover information about tag attribute type {:?}",
                                    r#type
                                )
                            }
                            None => log::info!(
                                "no hover information about attribute not contained in a tag"
                            ),
                        }
                    }
                    Err(err) => {
                        log::info!("failed to parse attribute value text for hover: {}", err)
                    }
                },
                None => log::info!("no hover information about attribute value without parent"),
            };
            result
        }
        kind if kind.ends_with("_tag_open") || kind.ends_with("_tag_close") => {
            match grammar::Tag::from_str(kind.rsplit_once("_").unwrap().0) {
                Ok(tag) => tag.properties().documentation.map(|d| d.to_string()),
                Err(_) => return Ok(None),
            }
        }
        kind => match node.parent() {
            Some(parent) if parent.kind().ends_with("_attribute") => {
                match find_containing_tag(parent).map(|tag| tag.properties().attributes) {
                    Some(grammar::TagAttributes::These(attributes)) => {
                        let kind = &parent.kind();
                        let attribute_name = &kind[..kind.len() - "_attribute".len()];
                        attributes
                            .iter()
                            .find(|attribute| attribute.name == attribute_name)
                            .and_then(|attribute| attribute.documentation)
                            .map(|documentation| documentation.to_string())
                    }
                    _ => {
                        log::info!("no hover information about node \"{}\"", kind);
                        return Ok(None);
                    }
                }
            }
            _ => {
                log::info!("no hover information about node \"{}\"", kind);
                return Ok(None);
            }
        },
    })
    .map(|value| Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        }),
        range: None,
    }));
}

fn find_containing_tag(node: Node<'_>) -> Option<Tag> {
    return node
        .parent()
        .and_then(|parent| grammar::Tag::from_str(parent.kind()).ok());
}

fn find_attribute_definition(tag: Tag, attribute: Node<'_>) -> Option<&TagAttribute> {
    let attribute_name = attribute.kind().strip_suffix("_attribute")?;
    if let TagAttributes::These(definitions) = tag.properties().attributes {
        for definition in definitions {
            if definition.name == attribute_name {
                return Some(definition);
            }
        }
    }
    return None;
}

fn location_contains_cursor(location: &Location, cursor: &Position, offset: &Point) -> bool {
    let start = location.char() as usize;
    return (start..start + location.len() as usize)
        .contains(&(cursor.character as usize - offset.column));
}
