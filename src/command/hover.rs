use std::{cmp::Ordering, str::FromStr};

use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind, Position};
use tree_sitter::{Node, Point};

use super::{LsError, ResponseErrorCode};

use crate::{
    document_store,
    grammar::{self, TagAttribute, TagAttributeType, TagAttributes, TagDefinition},
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
                            Some(TagAttributeType::Comparable) => match parser.parse_comparable() {
                                Ok(root) => result = hover_comparable(root, &cursor, &offset),
                                Err(err) => log::info!(
                                    "no hover information about invalid comparable: {}",
                                    err
                                ),
                            },
                            Some(TagAttributeType::Condition) => {
                                match parser.parse_condition_ast() {
                                    Ok(ast) => result = hover_condition(ast.root, &cursor, &offset),
                                    Err(err) => log::info!(
                                        "no hover information about invalid condition: {}",
                                        err
                                    ),
                                }
                            }
                            Some(TagAttributeType::Expression) => {
                                match parser.parse_expression_ast() {
                                    Ok(ast) => {
                                        result = hover_expression(ast.root, &cursor, &offset)
                                    }
                                    Err(err) => log::info!(
                                        "no hover information about invalid expression: {}",
                                        err
                                    ),
                                }
                            }
                            Some(TagAttributeType::Identifier) => match parser.parse_identifier() {
                                Ok(root) => result = hover_identifier(root, &cursor, &offset),
                                Err(err) => log::info!(
                                    "no hover information about invalid identifier: {}",
                                    err
                                ),
                            },
                            Some(TagAttributeType::Object) => match parser.parse_object_ast() {
                                Ok(ast) => result = hover_object(ast.root, &cursor, &offset),
                                Err(err) => {
                                    log::info!("no hover information about invalid object: {}", err)
                                }
                            },
                            Some(TagAttributeType::Regex) => match parser.parse_regex() {
                                Ok(regex) => result = hover_regex(regex, &cursor, &offset),
                                Err(err) => {
                                    log::info!("no hover information about invalid regex: {}", err)
                                }
                            },
                            // Some(TagAttributeType::String) => todo!(),
                            Some(TagAttributeType::Uri) => match parser.parse_uri() {
                                Ok(uri) => result = hover_uri(uri, &cursor, &offset),
                                Err(err) => {
                                    log::info!("no hover information about invalid uri: {}", err)
                                }
                            },
                            // Some(TagAttributeType::Query) => todo!(),
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
            match TagDefinition::from_str(kind.rsplit_once("_").unwrap().0) {
                Ok(tag) => tag.documentation.map(|d| d.to_string()),
                Err(_) => return Ok(None),
            }
        }
        kind => match node.parent() {
            Some(parent) if parent.kind().ends_with("_attribute") => {
                match find_containing_tag(parent).map(|tag| tag.attributes) {
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

fn hover_identifier(
    _identifier: ast::Identifier,
    _cursor: &Position,
    _offset: &Point,
) -> Option<String> {
    // TODO
    return None;
}

fn hover_comparable(
    comparable: ast::Comparable,
    cursor: &Position,
    offset: &Point,
) -> Option<String> {
    return match comparable {
        ast::Comparable::Condition(condition) => hover_condition(condition, cursor, offset),
        ast::Comparable::Expression(expression) => hover_expression(expression, cursor, offset),
        ast::Comparable::Function(function) => hover_global_function(function, cursor, offset),
        ast::Comparable::Object(interpolation) => {
            hover_object(interpolation.content, cursor, offset)
        }
        // ast::Comparable::String(_) => todo!(),
        // ast::Comparable::Null(_) => todo!(),
        _ => None,
    };
}
fn hover_condition(condition: ast::Condition, cursor: &Position, offset: &Point) -> Option<String> {
    return match condition {
        ast::Condition::True { location }
            if compare_cursor_to_location(&location, cursor, offset) == Ordering::Equal =>
        {
            Some("boolisches true".to_string())
        }
        ast::Condition::False { location }
            if compare_cursor_to_location(&location, cursor, offset) == Ordering::Equal =>
        {
            Some("boolisches false".to_string())
        }
        ast::Condition::Object(interpolation) => {
            hover_object(interpolation.content, cursor, offset)
        }
        ast::Condition::Function(function) => hover_global_function(function, cursor, offset),
        ast::Condition::BinaryOperation {
            left,
            right,
            operator_location,
            ..
        } => {
            match compare_cursor_to_location(&operator_location, cursor, offset) {
                Ordering::Less => hover_condition(*left, cursor, offset),
                Ordering::Equal => None,
                Ordering::Greater => hover_condition(*right, cursor, offset),
            }
        },
        ast::Condition::BracketedCondition {
            condition,
            ..
        } => hover_condition(*condition, cursor, offset),
        ast::Condition::NegatedCondition {
            condition,
            ..
        } => hover_condition(*condition, cursor, offset),
        ast::Condition::Comparisson {
            left,
            right,
            operator_location,
            ..
        } => {
            match compare_cursor_to_location(&operator_location, cursor, offset) {
                Ordering::Less => hover_comparable(*left, cursor, offset),
                Ordering::Equal => None,
                Ordering::Greater => hover_comparable(*right, cursor, offset),
            }
        },
        _ => None,
    };
}

fn hover_expression(
    expression: ast::Expression,
    cursor: &Position,
    offset: &Point,
) -> Option<String> {
    return match expression {
        // ast::Expression::Number(_) => todo!(),
        ast::Expression::Object(interpolation) => {
            hover_object(interpolation.content, cursor, offset)
        }
        ast::Expression::SignedExpression { expression, .. } => {
            hover_expression(*expression, cursor, offset)
        }
        ast::Expression::BracketedExpression { expression, .. } => {
            hover_expression(*expression, cursor, offset)
        }
        ast::Expression::BinaryOperation {
            left,
            right,
            operator_location,
            ..
        } => match compare_cursor_to_location(&operator_location, cursor, offset) {
            Ordering::Less => hover_expression(*left, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => hover_expression(*right, cursor, offset),
        },
        ast::Expression::Ternary {
            condition,
            left,
            right,
            question_mark_location,
            colon_location,
        } => match compare_cursor_to_location(&question_mark_location, cursor, offset) {
            Ordering::Less => hover_condition(*condition, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => {
                match compare_cursor_to_location(&colon_location, cursor, offset) {
                    Ordering::Less => hover_expression(*left, cursor, offset),
                    Ordering::Equal => None,
                    Ordering::Greater => hover_expression(*right, cursor, offset),
                }
            }
        },
        _ => None,
    };
}

fn hover_object(object: ast::Object, cursor: &Position, offset: &Point) -> Option<String> {
    return match object {
        // ast::Object::Anchor(_) => todo!(),
        ast::Object::Function(function) => hover_global_function(function, cursor, offset),
        ast::Object::Name(_interpolation) => None, // TODO
        // ast::Object::Null(_) => todo!(),
        // ast::Object::String(_) => todo!(),
        ast::Object::FieldAccess {
            object,
            field: _field,
            dot_location,
        } => {
            match compare_cursor_to_location(&dot_location, cursor, offset) {
                Ordering::Less => hover_object(*object, cursor, offset),
                Ordering::Equal => None,
                Ordering::Greater => None, // TODO
            }
        }
        ast::Object::MethodAccess {
            object,
            function: _function,
            dot_location,
        } => {
            match compare_cursor_to_location(&dot_location, cursor, offset) {
                Ordering::Less => hover_object(*object, cursor, offset),
                Ordering::Equal => None,
                Ordering::Greater => None, // TODO
            }
        }
        ast::Object::ArrayAccess {
            object,
            index,
            opening_bracket_location,
            ..
        } => match compare_cursor_to_location(&opening_bracket_location, cursor, offset) {
            Ordering::Less => hover_object(*object, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => hover_expression(index, cursor, offset),
        },
        _ => None,
    };
}

fn hover_global_function(
    function: ast::Function,
    cursor: &Position,
    offset: &Point,
) -> Option<String> {
    return match compare_cursor_to_location(&function.opening_bracket_location, cursor, offset) {
        Ordering::Less => spel::grammar::Function::from_str(&function.name)
            .ok()
            .map(|tag| tag.documentation.to_owned().to_string()),
        Ordering::Equal => None,
        Ordering::Greater => {
            for argument in function.arguments {
                if argument
                    .comma_location
                    .filter(|l| compare_cursor_to_location(&l, cursor, offset) == Ordering::Less)
                    .is_none()
                {
                    return match argument.argument {
                        // ast::Argument::Anchor(_) => todo!(),
                        ast::Argument::Function(function) => {
                            hover_global_function(function, cursor, offset)
                        }
                        // ast::Argument::Null(_) => todo!(),
                        // ast::Argument::Number(_) => todo!(),
                        ast::Argument::Object(interpolation) => {
                            hover_object(interpolation.content, cursor, offset)
                        }
                        // ast::Argument::SignedNumber(_) => todo!(),
                        // ast::Argument::String(_) => todo!(),
                        _ => None,
                    };
                }
            }
            return None;
        }
    };
}

fn hover_regex(_regex: ast::Regex, _cursor: &Position, _offset: &Point) -> Option<String> {
    // TODO
    return None;
}

fn hover_uri(_uri: ast::Uri, _cursor: &Position, _offset: &Point) -> Option<String> {
    // TODO
    return None;
}

fn find_containing_tag(node: Node<'_>) -> Option<TagDefinition> {
    return node
        .parent()
        .and_then(|parent| TagDefinition::from_str(parent.kind()).ok());
}

fn find_attribute_definition(tag: TagDefinition, attribute: Node<'_>) -> Option<&TagAttribute> {
    let attribute_name = attribute.kind().strip_suffix("_attribute")?;
    if let TagAttributes::These(definitions) = tag.attributes {
        for definition in definitions {
            if definition.name == attribute_name {
                return Some(definition);
            }
        }
    }
    return None;
}

// TODO: only respects single line spels
fn compare_cursor_to_location(location: &Location, cursor: &Position, offset: &Point) -> Ordering {
    let cursor = cursor.character as usize - offset.column;
    let start = location.char() as usize;
    if start > cursor {
        return Ordering::Less;
    }
    if location.len() as usize + start < cursor {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}
