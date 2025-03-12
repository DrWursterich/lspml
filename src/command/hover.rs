use std::{cmp::Ordering, str::FromStr};

use lsp_server::ErrorCode;
use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind, Position};

use super::LsError;

use parser::{
    AttributeValue, HtmlAttributeValueContent, HtmlAttributeValueFragment, HtmlNode, Node,
    ParsableTag, ParsedAttribute, ParsedHtml, ParsedTag, SpmlTag,
};
use spel::{
    ast::{
        Argument, Comparable, Condition, Expression, Function, Identifier, Location, Object, Query,
        Regex, SpelAst, SpelResult, Uri, Word, WordFragment,
    },
    grammar::FunctionDefinition,
};

pub(crate) fn hover(params: HoverParams) -> Result<Option<Hover>, LsError> {
    let text_params = params.text_document_position_params;
    let file = &text_params.text_document.uri;
    let document = match document_store::get(file) {
        Some(document) => Ok(document),
        None => document_store::Document::from_uri(file)
            .map(|document| document_store::put(file, document))
            .map_err(|err| {
                log::error!("failed to read {:?}: {}", file, err);
                return LsError {
                    message: format!("cannot read file {:?}", file),
                    code: ErrorCode::RequestFailed,
                };
            }),
    }?;
    let cursor = text_params.position;
    return match document.tree.node_at(cursor) {
        Some(Node::Tag(ParsedTag::Valid(tag))) => hover_tag(tag, &cursor),
        Some(Node::Tag(ParsedTag::Erroneous(tag, _))) => hover_tag(tag, &cursor),
        Some(Node::Html(ParsedHtml::Valid(html))) => hover_html(html, &cursor),
        Some(Node::Html(ParsedHtml::Erroneous(html, _))) => hover_html(html, &cursor),
        _ => Ok(None),
    };
}

fn hover_tag(tag: &SpmlTag, cursor: &Position) -> Result<Option<Hover>, LsError> {
    if tag.open_location().contains(cursor) || tag.close_location().contains(cursor) {
        return Ok(tag.definition().documentation.map(|doc| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc.to_string(),
            }),
            range: None,
        }));
    }
    for (name, attribute) in tag.spel_attributes() {
        let attribute = match attribute {
            ParsedAttribute::Valid(attribute) => attribute,
            ParsedAttribute::Erroneous(attribute, _) => attribute,
            ParsedAttribute::Unparsable(_, _) => continue,
        };
        if attribute.key.location.contains(cursor) {
            return Ok(tag
                .definition()
                .attributes
                .get_by_name(name.strip_suffix("_attribute").unwrap_or(name))
                .and_then(|attribute| attribute.documentation)
                .map(|doc| Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: doc.to_string(),
                    }),
                    range: None,
                }));
        }
        if attribute.value.is_inside(&cursor) {
            let offset = Position {
                line: attribute.value.opening_quote_location.line as u32,
                character: (attribute.value.opening_quote_location.char + 1) as u32,
            };
            return Ok(match &attribute.value.spel {
                SpelAst::Comparable(SpelResult::Valid(comparable)) => {
                    hover_comparable(comparable, &cursor, &offset)
                }
                SpelAst::Condition(SpelResult::Valid(condition)) => {
                    hover_condition(condition, &cursor, &offset)
                }
                SpelAst::Expression(SpelResult::Valid(expression)) => {
                    hover_expression(expression, &cursor, &offset)
                }
                SpelAst::Identifier(SpelResult::Valid(identifier)) => {
                    hover_identifier(identifier, &cursor, &offset)
                }
                SpelAst::Object(SpelResult::Valid(object)) => {
                    hover_object(object, &cursor, &offset)
                }
                SpelAst::Query(SpelResult::Valid(query)) => hover_query(query, &cursor, &offset),
                SpelAst::Regex(SpelResult::Valid(regex)) => hover_regex(regex, &cursor, &offset),
                SpelAst::String(SpelResult::Valid(word)) => hover_text(word, &cursor, &offset),
                SpelAst::Uri(SpelResult::Valid(uri)) => hover_uri(uri, &cursor, &offset),
                _ => None,
            }
            .map(|value| Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                }),
                range: None,
            }));
        }
    }
    return Ok(None);
}

fn hover_html(html: &HtmlNode, cursor: &Position) -> Result<Option<Hover>, LsError> {
    for attribute in &html.attributes {
        let attribute = match attribute {
            ParsedAttribute::Valid(attribute) => attribute,
            ParsedAttribute::Erroneous(attribute, _) => attribute,
            ParsedAttribute::Unparsable(_, _) => continue,
        };
        if let Some(value) = &attribute.value {
            if &value.closing_quote_location.start() < cursor {
                continue;
            }
            if &value.opening_quote_location.end() > cursor {
                break;
            }
            match &value.content {
                HtmlAttributeValueContent::Tag(ParsedTag::Valid(tag)) => {
                    return hover_tag(tag, &cursor);
                }
                HtmlAttributeValueContent::Tag(ParsedTag::Erroneous(tag, _)) => {
                    return hover_tag(tag, &cursor);
                }
                HtmlAttributeValueContent::Fragmented(fragments) => {
                    for fragment in fragments {
                        return match fragment {
                            HtmlAttributeValueFragment::Tag(ParsedTag::Valid(tag)) => {
                                hover_tag(tag, &cursor)
                            }
                            HtmlAttributeValueFragment::Tag(ParsedTag::Erroneous(tag, _)) => {
                                hover_tag(tag, &cursor)
                            }
                            _ => continue,
                        };
                    }
                }
                _ => continue,
            }
        }
    }
    Ok(None)
}

fn hover_identifier(
    _identifier: &Identifier,
    _cursor: &Position,
    _offset: &Position,
) -> Option<String> {
    // TODO
    return None;
}

fn hover_comparable(
    comparable: &Comparable,
    cursor: &Position,
    offset: &Position,
) -> Option<String> {
    return match comparable {
        Comparable::Condition(condition) => hover_condition(condition, cursor, offset),
        Comparable::Expression(expression) => hover_expression(expression, cursor, offset),
        Comparable::Function(function) => hover_global_function(function, cursor, offset),
        Comparable::Object(interpolation) => hover_object(&interpolation.content, cursor, offset),
        // Comparable::String(_) => todo!(),
        // Comparable::Null(_) => todo!(),
        _ => None,
    };
}
fn hover_condition(condition: &Condition, cursor: &Position, offset: &Position) -> Option<String> {
    return match condition {
        Condition::Object(interpolation) => hover_object(&interpolation.content, cursor, offset),
        Condition::Function(function) => hover_global_function(function, cursor, offset),
        Condition::BinaryOperation {
            left,
            right,
            operator_location,
            ..
        } => match compare_cursor_to_location(&operator_location, cursor, offset) {
            Ordering::Less => hover_condition(left, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => hover_condition(right, cursor, offset),
        },
        Condition::BracketedCondition { condition, .. } => {
            hover_condition(condition, cursor, offset)
        }
        Condition::NegatedCondition { condition, .. } => hover_condition(condition, cursor, offset),
        Condition::Comparisson {
            left,
            right,
            operator_location,
            ..
        } => match compare_cursor_to_location(&operator_location, cursor, offset) {
            Ordering::Less => hover_comparable(left, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => hover_comparable(right, cursor, offset),
        },
        _ => None,
    };
}

fn hover_expression(
    expression: &Expression,
    cursor: &Position,
    offset: &Position,
) -> Option<String> {
    return match expression {
        // Expression::Number(_) => todo!(),
        Expression::Function(function) => hover_global_function(function, cursor, offset),
        Expression::Object(interpolation) => hover_object(&interpolation.content, cursor, offset),
        Expression::SignedExpression { expression, .. } => {
            hover_expression(expression, cursor, offset)
        }
        Expression::BracketedExpression { expression, .. } => {
            hover_expression(expression, cursor, offset)
        }
        Expression::BinaryOperation {
            left,
            right,
            operator_location,
            ..
        } => match compare_cursor_to_location(&operator_location, cursor, offset) {
            Ordering::Less => hover_expression(left, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => hover_expression(right, cursor, offset),
        },
        Expression::Ternary {
            condition,
            left,
            right,
            question_mark_location,
            colon_location,
        } => match compare_cursor_to_location(&question_mark_location, cursor, offset) {
            Ordering::Less => hover_condition(condition, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => {
                match compare_cursor_to_location(&colon_location, cursor, offset) {
                    Ordering::Less => hover_expression(left, cursor, offset),
                    Ordering::Equal => None,
                    Ordering::Greater => hover_expression(right, cursor, offset),
                }
            }
        },
        _ => None,
    };
}

fn hover_object(object: &Object, cursor: &Position, offset: &Position) -> Option<String> {
    return match object {
        // Object::Anchor(_) => todo!(),
        Object::Function(function) => hover_global_function(function, cursor, offset),
        Object::Name(Word { fragments }) => {
            match fragments.len() {
                1 => match &fragments[0] {
                    WordFragment::String(_) => {
                        // there are no doc comments in spml
                    }
                    WordFragment::Interpolation(interpolation) => {
                        return hover_object(&interpolation.content, cursor, offset);
                    }
                },
                _ => {
                    for fragment in fragments {
                        if let WordFragment::Interpolation(interpolation) = fragment {
                            if let Ordering::Less = compare_cursor_to_location(
                                &interpolation.closing_bracket_location,
                                cursor,
                                offset,
                            ) {
                                return hover_object(&interpolation.content, cursor, offset);
                            };
                        }
                    }
                }
            };
            return None;
        }
        // Object::Null(_) => todo!(),
        // Object::String(_) => todo!(),
        Object::FieldAccess {
            object,
            field: _field,
            dot_location,
        } => {
            match compare_cursor_to_location(&dot_location, cursor, offset) {
                Ordering::Less => hover_object(object, cursor, offset),
                Ordering::Equal => None,
                Ordering::Greater => None, // TODO
            }
        }
        Object::MethodAccess {
            object,
            function: _function,
            dot_location,
        } => {
            match compare_cursor_to_location(&dot_location, cursor, offset) {
                Ordering::Less => hover_object(object, cursor, offset),
                Ordering::Equal => None,
                Ordering::Greater => None, // TODO
            }
        }
        Object::ArrayAccess {
            object,
            index,
            opening_bracket_location,
            ..
        } => match compare_cursor_to_location(&opening_bracket_location, cursor, offset) {
            Ordering::Less => hover_object(object, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => hover_expression(index, cursor, offset),
        },
        _ => None,
    };
}

fn hover_global_function(
    function: &Function,
    cursor: &Position,
    offset: &Position,
) -> Option<String> {
    return match compare_cursor_to_location(&function.opening_bracket_location, cursor, offset) {
        Ordering::Less => FunctionDefinition::from_str(&function.name)
            .ok()
            .map(|tag| tag.documentation.to_owned().to_string()),
        Ordering::Equal => None,
        Ordering::Greater => {
            for argument in &function.arguments {
                if argument
                    .comma_location
                    .as_ref()
                    .filter(|l| compare_cursor_to_location(&l, cursor, offset) == Ordering::Less)
                    .is_none()
                {
                    return match &argument.argument {
                        // Argument::Anchor(_) => todo!(),
                        Argument::Function(function) => {
                            hover_global_function(&function, cursor, offset)
                        }
                        // Argument::Null(_) => todo!(),
                        // Argument::Number(_) => todo!(),
                        Argument::Object(interpolation) => {
                            hover_object(&interpolation.content, cursor, offset)
                        }
                        // Argument::SignedNumber(_) => todo!(),
                        // Argument::String(_) => todo!(),
                        _ => None,
                    };
                }
            }
            return None;
        }
    };
}

fn hover_query(_query: &Query, _cursor: &Position, _offset: &Position) -> Option<String> {
    // TODO
    return None;
}

fn hover_regex(_regex: &Regex, _cursor: &Position, _offset: &Position) -> Option<String> {
    // TODO
    return None;
}

fn hover_text(text: &Word, cursor: &Position, offset: &Position) -> Option<String> {
    match text.fragments.len() {
        1 => match &text.fragments[0] {
            WordFragment::String(_) => {
                // there are no doc comments in spml
            }
            WordFragment::Interpolation(interpolation) => {
                return hover_object(&interpolation.content, cursor, offset);
            }
        },
        _ => {
            for fragment in &text.fragments {
                if let WordFragment::Interpolation(interpolation) = fragment {
                    if let Ordering::Less = compare_cursor_to_location(
                        &interpolation.closing_bracket_location,
                        cursor,
                        offset,
                    ) {
                        return hover_object(&interpolation.content, cursor, offset);
                    };
                }
            }
        }
    };
    return None;
}

fn hover_uri(_uri: &Uri, _cursor: &Position, _offset: &Position) -> Option<String> {
    // TODO
    return None;
}

// TODO: only respects single line spels
fn compare_cursor_to_location(
    location: &Location,
    cursor: &Position,
    offset: &Position,
) -> Ordering {
    let cursor = cursor.character - offset.character;
    let start = location.char() as u32;
    if start > cursor {
        return Ordering::Less;
    }
    if location.len() as u32 + start < cursor {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}
