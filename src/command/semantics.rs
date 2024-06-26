use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use lsp_server::ErrorCode;
use lsp_types::{SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensParams};
use tree_sitter::{Node, Point};

use crate::{
    document_store,
    grammar::TagDefinition,
    spel::ast::{
        Anchor, Argument, Comparable, Condition, Expression, Function, Identifier, Interpolation,
        Location, Null, Number, Object, Query, Regex, SignedNumber, SpelAst, SpelResult,
        StringLiteral, Uri, Word, WordFragment,
    },
};

use super::{
    super::capabilities::{TOKEN_MODIFIERS, TOKEN_TYPES},
    LsError,
};

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
        location: &Location,
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
        None => document_store::Document::from_uri(&uri)
            .map(|document| document_store::put(&uri, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", uri, err);
                return LsError {
                    message: format!("cannot read file {}", uri),
                    code: ErrorCode::RequestFailed,
                };
            }),
    }?;
    let tokenizer = &mut Tokenizer::new();
    index_document(
        document.tree.root_node(),
        &document.text,
        &document.spel,
        tokenizer,
    )
    .map_err(|err| {
        log::error!("semantic token parsing failed for {}: {}", uri, err);
        return LsError {
            message: format!("semantic token parsing failed for {}", uri),
            code: ErrorCode::RequestFailed,
        };
    })?;
    return Ok(tokenizer.collect());
}

fn index_document(
    root: Node,
    text: &String,
    spel: &HashMap<Point, SpelAst>,
    tokenizer: &mut Tokenizer,
) -> Result<()> {
    for node in root.children(&mut root.walk()) {
        match node.kind() {
            "page_header" | "import_header" | "taglib_header" | "html_doctype" | "text"
            | "comment" | "xml_entity" => continue,
            "html_tag" | "html_option_tag" | "html_void_tag" | "xml_comment" | "java_tag"
            | "script_tag" | "style_tag" => index_children(node, &text, spel, tokenizer)?,
            _ => match &TagDefinition::from_str(node.kind()) {
                Ok(tag) => index_tag(tag, node, &text, spel, tokenizer)?,
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
    tag: &TagDefinition,
    node: Node,
    text: &String,
    spel: &HashMap<Point, SpelAst>,
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
            "ERROR" | "html_tag" | "html_option_tag" => {
                index_children(child, text, spel, tokenizer)?
            }
            kind if kind.ends_with("_attribute") => {
                let value_node = match child.child(2).and_then(|child| child.child(1)) {
                    Some(node) => node,
                    _ => continue,
                };
                let offset = value_node.start_position();
                let mut token_collector =
                    SpelTokenCollector::new(tokenizer, offset.row as u32, offset.column as u32);
                match spel.get(&offset) {
                    Some(SpelAst::Comparable(SpelResult::Valid(comparable))) => {
                        index_comparable(&comparable, &mut token_collector)
                    }
                    Some(SpelAst::Condition(SpelResult::Valid(condition))) => {
                        index_condition(&condition, &mut token_collector)
                    }
                    Some(SpelAst::Expression(SpelResult::Valid(expression))) => {
                        index_expression(&expression, &mut token_collector)
                    }
                    Some(SpelAst::Identifier(SpelResult::Valid(identifier))) => {
                        index_identifier(&identifier, &mut token_collector)
                    }
                    Some(SpelAst::Object(SpelResult::Valid(object))) => {
                        index_object(&object, &mut token_collector)
                    }
                    Some(SpelAst::Query(SpelResult::Valid(query))) => {
                        index_query(&query, &mut token_collector)
                    }
                    Some(SpelAst::Regex(SpelResult::Valid(regex))) => {
                        index_regex(&regex, &mut token_collector)
                    }
                    Some(SpelAst::String(SpelResult::Valid(string))) => {
                        index_word(&string, &mut token_collector, None, &vec![])
                    }
                    Some(SpelAst::Uri(SpelResult::Valid(uri))) => {
                        index_uri(&uri, &mut token_collector)
                    }
                    _ => (),
                };
            }
            kind if kind.ends_with("_tag") => match &TagDefinition::from_str(kind) {
                Ok(child_tag) => index_tag(child_tag, child, text, spel, tokenizer)?,
                Err(err) => {
                    log::info!("expected sp or spt tag: {}", err);
                }
            },
            _ => index_children(child, text, spel, tokenizer)?,
        }
    }
    return Ok(());
}

fn index_children(
    node: Node,
    text: &String,
    spel: &HashMap<Point, SpelAst>,
    tokenizer: &mut Tokenizer,
) -> Result<()> {
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "text" | "java_tag" | "html_void_tag" => {}
            "ERROR" | "html_tag" | "html_option_tag" | "script_tag" | "style_tag" => {
                index_children(child, text, spel, tokenizer)?;
            }
            kind if kind.ends_with("_tag") => match &TagDefinition::from_str(kind) {
                Ok(child_tag) => index_tag(child_tag, child, text, spel, tokenizer)?,
                Err(err) => {
                    log::info!("expected sp or spt tag: {}", err);
                }
            },
            _ => index_children(child, text, spel, tokenizer)?,
        }
    }
    return Ok(());
}

fn index_identifier(identifier: &Identifier, token_collector: &mut SpelTokenCollector) {
    match identifier {
        Identifier::Name(name) => index_word(
            name,
            token_collector,
            Some(SemanticTokenType::VARIABLE),
            &vec![],
        ),
        Identifier::FieldAccess {
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

fn index_object(object: &Object, token_collector: &mut SpelTokenCollector) {
    match object {
        Object::Anchor(anchor) => index_anchor(&anchor, token_collector),
        Object::Function(function) => index_function(function, token_collector),
        Object::Name(name) => {
            index_word(
                name,
                token_collector,
                Some(SemanticTokenType::VARIABLE),
                &vec![],
            );
        }
        Object::String(string) => index_string(string, token_collector),
        Object::FieldAccess {
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
        Object::MethodAccess {
            object,
            dot_location,
            function,
        } => {
            index_object(object, token_collector);
            token_collector.add(dot_location, &SemanticTokenType::OPERATOR, &vec![]);
            index_function(function, token_collector);
        }
        Object::ArrayAccess {
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

fn index_anchor(anchor: &Anchor, token_collector: &mut SpelTokenCollector) {
    token_collector.add(
        &anchor.opening_bracket_location,
        &SemanticTokenType::OPERATOR,
        &vec![],
    );
    index_word(
        &anchor.name,
        token_collector,
        Some(SemanticTokenType::ENUM_MEMBER),
        &vec![],
    );
    token_collector.add(
        &anchor.closing_bracket_location,
        &SemanticTokenType::OPERATOR,
        &vec![],
    );
}

fn index_string(string: &StringLiteral, token_collector: &mut SpelTokenCollector) {
    token_collector.add(&string.location, &SemanticTokenType::STRING, &vec![]);
}

fn index_null(null: &Null, token_collector: &mut SpelTokenCollector) {
    token_collector.add(&null.location, &SemanticTokenType::ENUM_MEMBER, &vec![])
}

fn index_number(number: &Number, token_collector: &mut SpelTokenCollector) {
    token_collector.add(&number.location, &SemanticTokenType::NUMBER, &vec![]);
}

fn index_signed_number(number: &SignedNumber, token_collector: &mut SpelTokenCollector) {
    token_collector.add(&number.sign_location, &SemanticTokenType::OPERATOR, &vec![]);
    index_number(&number.number, token_collector);
}

fn index_expression(expression: &Expression, token_collector: &mut SpelTokenCollector) {
    match expression {
        Expression::Function(function) => index_function(&function, token_collector),
        Expression::Null(null) => index_null(&null, token_collector),
        Expression::Number(number) => index_number(&number, token_collector),
        Expression::Object(interpolation) => {
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
        Expression::SignedExpression {
            expression,
            sign_location,
            ..
        } => {
            token_collector.add(&sign_location, &SemanticTokenType::OPERATOR, &vec![]);
            index_expression(expression, token_collector);
        }
        Expression::BinaryOperation {
            left,
            right,
            operator_location: operation_location,
            ..
        } => {
            index_expression(left, token_collector);
            token_collector.add(operation_location, &SemanticTokenType::OPERATOR, &vec![]);
            index_expression(right, token_collector);
        }
        Expression::BracketedExpression {
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
        Expression::Ternary {
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

fn index_condition(condition: &Condition, token_collector: &mut SpelTokenCollector) {
    match condition {
        Condition::True { location } | Condition::False { location } => {
            token_collector.add(location, &SemanticTokenType::ENUM_MEMBER, &vec![])
        }
        Condition::Object(location) => {
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
        Condition::Function(function) => index_function(function, token_collector),
        Condition::BinaryOperation {
            left,
            right,
            operator_location,
            ..
        } => {
            index_condition(left, token_collector);
            token_collector.add(operator_location, &SemanticTokenType::OPERATOR, &vec![]);
            index_condition(right, token_collector);
        }
        Condition::BracketedCondition {
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
        Condition::NegatedCondition {
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
        Condition::Comparisson {
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

fn index_function(function: &Function, token_collector: &mut SpelTokenCollector) {
    token_collector.add(&function.name_location, &SemanticTokenType::METHOD, &vec![]);
    token_collector.add(
        &function.opening_bracket_location,
        &SemanticTokenType::OPERATOR,
        &vec![],
    );
    for arg in function.arguments.iter() {
        match &arg.argument {
            Argument::Anchor(anchor) => index_anchor(&anchor, token_collector),
            Argument::Function(function) => index_function(&function, token_collector),
            Argument::Null(null) => index_null(&null, token_collector),
            Argument::Number(number) => index_number(&number, token_collector),
            Argument::Object(interpolation) => index_interpolation(&interpolation, token_collector),
            Argument::SignedNumber(number) => index_signed_number(&number, token_collector),
            Argument::String(string) => index_string(&string, token_collector),
            Argument::True { location } | Argument::False { location } => {
                token_collector.add(location, &SemanticTokenType::ENUM_MEMBER, &vec![])
            }
        }
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

fn index_uri(uri: &Uri, token_collector: &mut SpelTokenCollector) {
    match uri {
        Uri::Literal(literal) => {
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
        Uri::Object(object) => index_interpolation(object, token_collector),
    };
}

fn index_regex(regex: &Regex, token_collector: &mut SpelTokenCollector) {
    token_collector.add(
        &regex.location,
        // &SemanticTokenType::ENUM_MEMBER,
        &SemanticTokenType::REGEXP,
        &vec![],
    );
}

fn index_query(_query: &Query, _token_collector: &mut SpelTokenCollector) {
    // TODO:!
}

fn index_word(
    word: &Word,
    token_collector: &mut SpelTokenCollector,
    token_type: Option<SemanticTokenType>,
    token_modifiers: &Vec<SemanticTokenModifier>,
) {
    for fragment in &word.fragments {
        match fragment {
            WordFragment::String(StringLiteral { location, .. }) => {
                if let Some(token_type) = &token_type {
                    token_collector.add(&location, token_type, token_modifiers);
                }
            }
            WordFragment::Interpolation(interpolation) => {
                index_interpolation(&interpolation, token_collector)
            }
        }
    }
}

fn index_comparable(comparable: &Comparable, token_collector: &mut SpelTokenCollector) {
    match comparable {
        Comparable::Condition(condition) => index_condition(&condition, token_collector),
        Comparable::Expression(expression) => index_expression(&expression, token_collector),
        Comparable::Function(function) => index_function(&function, token_collector),
        Comparable::Object(interpolation) => {
            index_interpolation(interpolation, token_collector);
        }
        Comparable::String(string) => index_string(&string, token_collector),
        Comparable::Null(null) => index_null(&null, token_collector),
    }
}

fn index_interpolation(interpolation: &Interpolation, token_collector: &mut SpelTokenCollector) {
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
        capabilities::TOKEN_TYPES,
        command::semantics::{SpelTokenCollector, Tokenizer},
        spel::ast::{Location, Object, StringLiteral, Word, WordFragment},
    };

    #[test]
    fn test_index_single_object() {
        let tokenizer = &mut Tokenizer::new();
        let root_object = Object::Name(Word {
            fragments: vec![WordFragment::String(StringLiteral {
                content: "_someVariable".to_string(),
                location: Location::VariableLength {
                    char: 0,
                    line: 0,
                    length: 13,
                },
            })],
        });
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
        let root_object = Object::Name(Word {
            fragments: vec![WordFragment::String(StringLiteral {
                content: "_someVariable".to_string(),
                location: Location::VariableLength {
                    char: 0,
                    line: 0,
                    length: 13,
                },
            })],
        });
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
