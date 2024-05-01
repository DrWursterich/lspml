use lsp_server::ErrorCode;
use lsp_types::{GotoDefinitionParams, Position, Range, Url};
use std::{cmp::Ordering, path::Path};
use tree_sitter::Point;

use crate::{
    document_store::{self, Document},
    grammar::{TagAttributeType, TagAttributes},
    modules,
    parser::{DocumentNode, Node, ParsableTag, SpelAttribute, Tag},
    spel::ast::{
        Argument, Comparable, Condition, Expression, Function, Identifier, Location, Object, Query,
        Regex, SpelAst, SpelResult, StringLiteral, Uri, Word, WordFragment,
    },
};

use super::LsError;

type Variable = String;

struct DefinitionsFinder<'a> {
    document: &'a Document,
    uri: &'a Url,
    variable: Variable,
    upper_bound: Position,
    definitions: Vec<lsp_types::Location>,
}

impl DefinitionsFinder<'_> {
    fn find(
        upper_bound: Position,
        document: &Document,
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
            if let Node::Tag(tag) = node {
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
    }

    fn collect_from_tag(&mut self, tag: &Tag) {
        if let TagAttributes::These(attributes) = tag.definition().attributes {
            for attribute in attributes {
                if let TagAttributeType::Identifier = attribute.r#type {
                    if let Some(SpelAttribute {
                        spel:
                            SpelAst::Identifier(SpelResult::Valid(Identifier::Name(Word { fragments }))),
                        opening_quote_location,
                        ..
                    }) = tag.spel_attribute(&attribute.name)
                    {
                        if fragments.len() == 1 {
                            if let WordFragment::String(StringLiteral { content, location }) =
                                &fragments[0]
                            {
                                if *content == self.variable {
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
                if let Node::Tag(tag) = node {
                    self.collect_from_tag(tag)
                };
            }
        }
    }
}

pub(crate) fn definition(
    params: GotoDefinitionParams,
) -> Result<Option<Vec<lsp_types::Location>>, LsError> {
    let text_params = params.text_document_position_params;
    let file = &text_params.text_document.uri;
    let document = match document_store::get(file) {
        Some(document) => Ok(document),
        None => document_store::Document::from_uri(file)
            .map(|document| document_store::put(file, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", file, err);
                return LsError {
                    message: format!("cannot read file {}", file),
                    code: ErrorCode::RequestFailed,
                };
            }),
    }?;
    let cursor = text_params.position;
    let mut nodes = &document.tree.nodes;
    let mut current = None;
    loop {
        if let Some(node) = find_tag_at(nodes, cursor) {
            current = Some(node);
            if let Node::Tag(tag) = node {
                if let Some(body) = tag.body() {
                    nodes = &body.nodes;
                    continue;
                }
            }
        }
        break;
    }
    if let Some(Node::Tag(tag)) = current {
        for (_, attribute) in tag.spel_attributes() {
            if is_in_attribute_value(attribute, &cursor) {
                let offset = Point {
                    row: attribute.opening_quote_location.line + 1,
                    column: attribute.opening_quote_location.char,
                };
                let node = match &attribute.spel {
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
                    SpelAst::Query(SpelResult::Valid(query)) => {
                        find_node_in_query(query, &cursor, &offset)
                    }
                    SpelAst::Regex(SpelResult::Valid(regex)) => {
                        find_node_in_regex(regex, &cursor, &offset)
                    }
                    SpelAst::String(SpelResult::Valid(word)) => {
                        find_node_in_text(word, &cursor, &offset)
                    }
                    SpelAst::Uri(SpelResult::Valid(Uri::Literal(uri))) => {
                        let mut module = None;
                        if let Some(SpelAttribute {
                            spel: SpelAst::String(SpelResult::Valid(Word { fragments })),
                            ..
                        }) = tag.spel_attribute("module")
                        // TODO: should use TagAttributeType::Uri { module_attribute} for this
                        {
                            if let [WordFragment::String(StringLiteral { content, .. })] =
                                &fragments[..]
                            {
                                module = modules::find_module_by_name(&content)
                            }
                        }
                        module =
                            module.or_else(|| {
                                text_params.text_document.uri.to_file_path().ok().and_then(
                                    |module| modules::find_module_for_file(module.as_path()),
                                )
                            });
                        return Ok(module
                            .map(|module| module.path + &uri.to_string())
                            .filter(|file| Path::new(&file).exists())
                            .and_then(|file| Url::parse(format!("file://{}", &file).as_str()).ok())
                            .map(|uri| {
                                vec![lsp_types::Location {
                                    range: Range {
                                        ..Default::default()
                                    },
                                    uri,
                                }]
                            }));
                    }
                    SpelAst::Uri(SpelResult::Valid(uri)) => find_node_in_uri(uri, &cursor, &offset),
                    _ => None,
                };
                if let Some(node) = node {
                    let definitions = DefinitionsFinder::find(cursor, &document, file, node);
                    return match definitions.len() {
                        0 => Ok(None),
                        _ => Ok(Some(definitions)),
                    };
                }
            }
        }
    }
    return Ok(None);
}

pub(crate) fn find_tag_at(nodes: &Vec<Node>, cursor: Position) -> Option<&Node> {
    for node in nodes {
        let range = node.range();
        if cursor > range.end {
            continue;
        }
        if cursor < range.start {
            break;
        }
        return Some(node);
    }
    return None;
}

fn is_in_attribute_value(attribute: &SpelAttribute, position: &Position) -> bool {
    let opening_line = attribute
        .opening_quote_location
        .line
        .cmp(&(position.line as usize));
    let opening_char = attribute
        .opening_quote_location
        .char
        .cmp(&(position.character as usize));
    match (opening_line, opening_char) {
        (Ordering::Less, _) | (Ordering::Equal, Ordering::Less) => (),
        _ => return false,
    }
    let closing_line = attribute
        .closing_quote_location
        .line
        .cmp(&(position.line as usize));
    let closing_char = attribute
        .closing_quote_location
        .char
        .cmp(&(position.character as usize));
    return match (closing_line, closing_char) {
        (Ordering::Greater, _) | (Ordering::Equal, Ordering::Greater) => true,
        _ => false,
    };
}

fn find_node_in_identifier(
    identifier: &Identifier,
    cursor: &Position,
    offset: &Point,
) -> Option<Variable> {
    // TODO
    match identifier {
        Identifier::Name(Word { fragments }) => {
            match fragments.len() {
                1 => match &fragments[0] {
                    WordFragment::String(_) => {
                        // there are no doc comments in spml
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
    offset: &Point,
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
    offset: &Point,
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
    offset: &Point,
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

fn find_node_in_object(object: &Object, cursor: &Position, offset: &Point) -> Option<Variable> {
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
    offset: &Point,
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

fn find_node_in_query(_query: &Query, _cursor: &Position, _offset: &Point) -> Option<Variable> {
    // TODO
    return None;
}

fn find_node_in_regex(_regex: &Regex, _cursor: &Position, _offset: &Point) -> Option<Variable> {
    // TODO
    return None;
}

fn find_node_in_text(_text: &Word, _cursor: &Position, _offset: &Point) -> Option<Variable> {
    // TODO
    return None;
}

fn find_node_in_uri(_uri: &Uri, _cursor: &Position, _offset: &Point) -> Option<Variable> {
    // TODO
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
