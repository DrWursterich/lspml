use lsp_server::ErrorCode;
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Position, Range, Uri as Url};
use std::{cmp::Ordering, iter, path::Path};

use grammar::{TagAttributeType, TagAttributes};
use parser::{
    AttributeValue, HtmlAttributeValueContent, HtmlAttributeValueFragment, HtmlNode, Node,
    ParsableTag, ParsedAttribute, ParsedHtml, ParsedTag, SpelAttribute, SpelAttributeValue,
    SpmlTag,
};
use spel::ast::{
    Argument, Comparable, Condition, Expression, Function, Identifier, Location, Object, Query,
    Regex, SpelAst, SpelResult, StringLiteral, Uri, Word, WordFragment,
};

use super::LsError;

type Variable = String;

struct DefinitionsFinder<'a> {
    document: &'a document_store::Document,
    uri: &'a Url,
    variable: Variable,
    upper_bound: Position,
    definitions: Vec<lsp_types::Location>,
}

impl DefinitionsFinder<'_> {
    fn find(
        upper_bound: Position,
        document: &document_store::Document,
        uri: &Url,
        variable: Variable,
    ) -> Vec<lsp_types::Location> {
        let mut finder = DefinitionsFinder {
            document,
            uri,
            upper_bound,
            variable,
            definitions: Vec::new(),
        };
        finder.collect();
        return finder.definitions;
    }

    fn collect(&mut self) {
        let nodes = &self.document.tree.nodes;
        for node in nodes {
            let tag = match node {
                Node::Tag(ParsedTag::Valid(tag)) => tag,
                Node::Tag(ParsedTag::Erroneous(tag, _)) => tag,
                Node::Html(ParsedHtml::Valid(html)) => {
                    for tag in tags_in_attributes(html) {
                        self.collect_from_tag(&tag);
                    }
                    continue;
                }
                Node::Html(ParsedHtml::Erroneous(html, _)) => {
                    for tag in tags_in_attributes(html) {
                        self.collect_from_tag(&tag);
                    }
                    continue;
                }
                _ => continue,
            };
            let open_location = tag.open_location();
            if open_location.line > self.upper_bound.line as usize {
                return;
            }
            if open_location.line == self.upper_bound.line as usize {
                if open_location.char >= self.upper_bound.character as usize {
                    return;
                }
            }
            self.collect_from_tag(&tag);
        }
    }

    fn collect_from_tag(&mut self, tag: &SpmlTag) {
        if let TagAttributes::These(attributes) = tag.definition().attributes {
            for attribute in attributes {
                if let TagAttributeType::Identifier = attribute.r#type {
                    if let Some(ParsedAttribute::Valid(SpelAttribute {
                        value:
                            SpelAttributeValue {
                                spel:
                                    SpelAst::Identifier(SpelResult::Valid(Identifier::Name(Word {
                                        fragments,
                                    }))),
                                opening_quote_location,
                                ..
                            },
                        ..
                    })) = tag.spel_attribute(&attribute.name)
                    {
                        if fragments.len() == 1 {
                            if let WordFragment::String(StringLiteral { content, location }) =
                                &fragments[0]
                            {
                                if &**content == self.variable {
                                    let start = Position {
                                        line: opening_quote_location.line as u32
                                            + location.line() as u32,
                                        character: opening_quote_location.char as u32
                                            + location.char() as u32
                                            + 1,
                                    };
                                    self.definitions.push(lsp_types::Location {
                                        uri: self.uri.clone(),
                                        range: Range {
                                            start,
                                            end: Position {
                                                line: start.line,
                                                character: start.character + location.len() as u32,
                                            },
                                        },
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(body) = &tag.body() {
            for node in &body.nodes {
                let tag = match node {
                    Node::Tag(ParsedTag::Valid(tag)) => tag,
                    Node::Tag(ParsedTag::Erroneous(tag, _)) => tag,
                    _ => continue,
                };
                self.collect_from_tag(tag)
            }
        }
    }
}

pub(crate) fn definition(
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>, LsError> {
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
    let tag = match document.tree.node_at(cursor) {
        Some(Node::Tag(ParsedTag::Valid(tag))) => tag,
        Some(Node::Tag(ParsedTag::Erroneous(tag, _))) => tag,
        Some(Node::Html(ParsedHtml::Valid(html))) => {
            &(match find_node_in_attributes(html, cursor) {
                Some(tag) => tag,
                None => return Ok(None),
            })
        }
        Some(Node::Html(ParsedHtml::Erroneous(html, _))) => {
            &(match find_node_in_attributes(html, cursor) {
                Some(tag) => tag,
                None => return Ok(None),
            })
        }
        // Some(Node::Html(ParsedHtml::Erroneous(html, _))) => tag,
        _ => return Ok(None),
    };
    for (_, attribute) in tag.spel_attributes() {
        let attribute = match attribute {
            ParsedAttribute::Valid(attribute) => attribute,
            ParsedAttribute::Erroneous(attribute, _) => attribute,
            ParsedAttribute::Unparsable(_, _) => continue,
        };
        if !attribute.value.is_inside(&cursor) {
            continue;
        }
        let offset = Position {
            line: (attribute.value.opening_quote_location.line + 1) as u32,
            character: attribute.value.opening_quote_location.char as u32,
        };
        let node = match &attribute.value.spel {
            SpelAst::Comparable(SpelResult::Valid(comparable)) => {
                find_node_in_comparable(comparable, &cursor, &offset)
            }
            SpelAst::Condition(SpelResult::Valid(condition)) => {
                find_node_in_condition(condition, &cursor, &offset)
            }
            SpelAst::Expression(SpelResult::Valid(expression)) => {
                find_node_in_expression(expression, &cursor, &offset)
            }
            SpelAst::Identifier(SpelResult::Valid(identifier)) => {
                find_node_in_identifier(identifier, &cursor, &offset)
            }
            SpelAst::Object(SpelResult::Valid(object)) => {
                find_node_in_object(object, &cursor, &offset)
            }
            SpelAst::Query(SpelResult::Valid(query)) => find_node_in_query(query, &cursor, &offset),
            SpelAst::Regex(SpelResult::Valid(regex)) => find_node_in_regex(regex, &cursor, &offset),
            SpelAst::String(SpelResult::Valid(word)) => find_node_in_text(word, &cursor, &offset),
            SpelAst::Uri(SpelResult::Valid(Uri::Literal(uri))) => {
                let mut module = None;
                let module_attribute = match tag.spel_attribute("module") {
                    Some(ParsedAttribute::Valid(attribute)) => Some(attribute),
                    Some(ParsedAttribute::Erroneous(attribute, _)) => Some(attribute),
                    _ => None,
                };
                if let Some(SpelAttribute {
                    value:
                        SpelAttributeValue {
                            spel: SpelAst::String(SpelResult::Valid(Word { fragments })),
                            ..
                        },
                    ..
                }) = module_attribute
                // TODO: should use TagAttributeType::Uri { module_attribute} for this
                {
                    if let [WordFragment::String(StringLiteral { content, .. })] = &fragments[..] {
                        module = modules::find_module_by_name(&content)
                    }
                }
                module = module.or_else(|| {
                    modules::find_module_for_file(Path::new(
                        text_params.text_document.uri.path().as_str(),
                    ))
                });
                return Ok(module
                    .map(|module| module.path + &uri.to_string())
                    .filter(|file| Path::new(&file).exists())
                    .and_then(|file| format!("file://{}", &file).parse().ok())
                    .map(|uri| lsp_types::Location {
                        range: Range {
                            ..Default::default()
                        },
                        uri,
                    })
                    .map(GotoDefinitionResponse::Scalar));
            }
            SpelAst::Uri(SpelResult::Valid(uri)) => find_node_in_uri(uri, &cursor, &offset),
            _ => None,
        };
        if let Some(node) = node {
            let mut definitions = DefinitionsFinder::find(cursor, &document, file, node);
            return match definitions.len() {
                0 => Ok(None),
                1 => Ok(Some(GotoDefinitionResponse::Scalar(definitions.remove(0)))),
                _ => Ok(Some(GotoDefinitionResponse::Array(definitions))),
            };
        }
    }
    return Ok(None);
}

fn find_node_in_attributes(html: &HtmlNode, location: Position) -> Option<SpmlTag> {
    tags_in_attributes(html)
        .find(|tag| tag.open_location().start() < location && tag.close_location().end() > location)
}

fn tags_in_attributes(html: &HtmlNode) -> Box<dyn Iterator<Item = SpmlTag> + '_> {
    Box::new(
        html.attributes
            .iter()
            .filter_map(|attribute| match attribute.to_owned() {
                ParsedAttribute::Valid(attribute) => attribute.value,
                ParsedAttribute::Erroneous(attribute, _) => attribute.value,
                ParsedAttribute::Unparsable(_, _) => None,
            })
            .flat_map(|value| match value.content {
                HtmlAttributeValueContent::Tag(ParsedTag::Valid(tag)) => {
                    Box::new(iter::once(tag)) as Box<dyn Iterator<Item = SpmlTag>>
                }
                HtmlAttributeValueContent::Tag(ParsedTag::Erroneous(tag, _)) => {
                    Box::new(iter::once(tag)) as Box<dyn Iterator<Item = SpmlTag>>
                }
                HtmlAttributeValueContent::Fragmented(fragments) => Box::new(
                    fragments
                        .iter()
                        .filter_map(|fragment| match fragment {
                            HtmlAttributeValueFragment::Tag(ParsedTag::Valid(tag)) => {
                                Some(tag.clone())
                            }
                            HtmlAttributeValueFragment::Tag(ParsedTag::Erroneous(tag, _)) => {
                                Some(tag.clone())
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .into_iter(),
                )
                    as Box<dyn Iterator<Item = SpmlTag>>,
                _ => Box::new(iter::empty()),
            }),
    )
}

fn find_node_in_identifier(
    identifier: &Identifier,
    cursor: &Position,
    offset: &Position,
) -> Option<Variable> {
    match identifier {
        // TODO
        // Identifier::FieldAccess { identifier, field, dot_location } => {
        // }
        Identifier::Name(Word { fragments }) => {
            match fragments.len() {
                1 => match &fragments[0] {
                    WordFragment::String(_) => {
                        // this IS the definition
                    }
                    WordFragment::Interpolation(interpolation) => {
                        return find_node_in_object(&interpolation.content, cursor, offset);
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
                                return find_node_in_object(&interpolation.content, cursor, offset);
                            };
                        }
                    }
                }
            };
        }
        _ => (),
    };
    return None;
}

fn find_node_in_comparable(
    comparable: &Comparable,
    cursor: &Position,
    offset: &Position,
) -> Option<Variable> {
    return match comparable {
        Comparable::Condition(condition) => find_node_in_condition(condition, cursor, offset),
        Comparable::Expression(expression) => find_node_in_expression(expression, cursor, offset),
        Comparable::Function(function) => find_node_in_global_function(function, cursor, offset),
        Comparable::Object(interpolation) => {
            find_node_in_object(&interpolation.content, cursor, offset)
        }
        // Comparable::String(_) => todo!(),
        // Comparable::Null(_) => todo!(),
        _ => None,
    };
}
fn find_node_in_condition(
    condition: &Condition,
    cursor: &Position,
    offset: &Position,
) -> Option<Variable> {
    return match condition {
        Condition::Object(interpolation) => {
            find_node_in_object(&interpolation.content, cursor, offset)
        }
        Condition::Function(function) => find_node_in_global_function(function, cursor, offset),
        Condition::BinaryOperation {
            left,
            right,
            operator_location,
            ..
        } => match compare_cursor_to_location(&operator_location, cursor, offset) {
            Ordering::Less => find_node_in_condition(left, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => find_node_in_condition(right, cursor, offset),
        },
        Condition::BracketedCondition { condition, .. } => {
            find_node_in_condition(condition, cursor, offset)
        }
        Condition::NegatedCondition { condition, .. } => {
            find_node_in_condition(condition, cursor, offset)
        }
        Condition::Comparisson {
            left,
            right,
            operator_location,
            ..
        } => match compare_cursor_to_location(&operator_location, cursor, offset) {
            Ordering::Less => find_node_in_comparable(left, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => find_node_in_comparable(right, cursor, offset),
        },
        _ => None,
    };
}

fn find_node_in_expression(
    expression: &Expression,
    cursor: &Position,
    offset: &Position,
) -> Option<Variable> {
    return match expression {
        // Expression::Number(_) => todo!(),
        Expression::Function(function) => find_node_in_global_function(function, cursor, offset),
        Expression::Object(interpolation) => {
            find_node_in_object(&interpolation.content, cursor, offset)
        }
        Expression::SignedExpression { expression, .. } => {
            find_node_in_expression(expression, cursor, offset)
        }
        Expression::BracketedExpression { expression, .. } => {
            find_node_in_expression(expression, cursor, offset)
        }
        Expression::BinaryOperation {
            left,
            right,
            operator_location,
            ..
        } => match compare_cursor_to_location(&operator_location, cursor, offset) {
            Ordering::Less => find_node_in_expression(left, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => find_node_in_expression(right, cursor, offset),
        },
        Expression::Ternary {
            condition,
            left,
            right,
            question_mark_location,
            colon_location,
        } => match compare_cursor_to_location(&question_mark_location, cursor, offset) {
            Ordering::Less => find_node_in_condition(condition, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => {
                match compare_cursor_to_location(&colon_location, cursor, offset) {
                    Ordering::Less => find_node_in_expression(left, cursor, offset),
                    Ordering::Equal => None,
                    Ordering::Greater => find_node_in_expression(right, cursor, offset),
                }
            }
        },
        _ => None,
    };
}

fn find_node_in_object(object: &Object, cursor: &Position, offset: &Position) -> Option<Variable> {
    return match object {
        // Object::Anchor(_) => todo!(),
        Object::Function(function) => find_node_in_global_function(function, cursor, offset),
        Object::Name(Word { fragments }) => {
            match fragments.len() {
                1 => match &fragments[0] {
                    WordFragment::String(StringLiteral { content, .. }) => {
                        return Some(content.to_string());
                    }
                    WordFragment::Interpolation(interpolation) => {
                        return find_node_in_object(&interpolation.content, cursor, offset);
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
                                return find_node_in_object(&interpolation.content, cursor, offset);
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
                Ordering::Less => find_node_in_object(object, cursor, offset),
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
                Ordering::Less => find_node_in_object(object, cursor, offset),
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
            Ordering::Less => find_node_in_object(object, cursor, offset),
            Ordering::Equal => None,
            Ordering::Greater => find_node_in_expression(index, cursor, offset),
        },
        _ => None,
    };
}

fn find_node_in_global_function(
    function: &Function,
    cursor: &Position,
    offset: &Position,
) -> Option<Variable> {
    return match compare_cursor_to_location(&function.opening_bracket_location, cursor, offset) {
        Ordering::Less => None,
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
                            find_node_in_global_function(&function, cursor, offset)
                        }
                        // Argument::Null(_) => todo!(),
                        // Argument::Number(_) => todo!(),
                        Argument::Object(interpolation) => {
                            find_node_in_object(&interpolation.content, cursor, offset)
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

fn find_node_in_query(_query: &Query, _cursor: &Position, _offset: &Position) -> Option<Variable> {
    // TODO
    return None;
}

fn find_node_in_regex(_regex: &Regex, _cursor: &Position, _offset: &Position) -> Option<Variable> {
    // TODO
    return None;
}

fn find_node_in_text(text: &Word, cursor: &Position, offset: &Position) -> Option<Variable> {
    match text.fragments.len() {
        1 => match &text.fragments[0] {
            WordFragment::String(_) => {
                // there are no doc comments in spml
            }
            WordFragment::Interpolation(interpolation) => {
                return find_node_in_object(&interpolation.content, cursor, offset);
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
                        return find_node_in_object(&interpolation.content, cursor, offset);
                    };
                }
            }
        }
    };
    return None;
}

fn find_node_in_uri(_uri: &Uri, _cursor: &Position, _offset: &Position) -> Option<Variable> {
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
