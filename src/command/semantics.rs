use std::cmp::Ordering;

use anyhow::Result;
use lsp_server::ErrorCode;
use lsp_types::{SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensParams};

use super::LsError;
use grammar::AttributeRule;
use parser::{
    HtmlAttributeValueContent, HtmlAttributeValueFragment, HtmlNode, Node, ParsableTag,
    ParsedAttribute, ParsedHtml, ParsedTag, SpmlTag, Tree,
};
use spel::ast::{
    Anchor, Argument, Comparable, Condition, Expression, Function, Identifier, Interpolation,
    Location, Null, Number, Object, Query, Regex, SignedNumber, SpelAst, SpelResult, StringLiteral,
    Uri, Word, WordFragment,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct UnprocessedSemanticToken {
    pub char: u32,
    pub line: u32,
    pub length: u32,
    pub token_type: u32,
    pub token_modifiers_bitset: u32,
}

impl PartialOrd for UnprocessedSemanticToken {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let cmp = self.line.cmp(&other.line);
        if let Ordering::Equal = cmp {
            return Some(self.char.cmp(&other.char));
        }
        return Some(cmp);
    }
}

impl Ord for UnprocessedSemanticToken {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp = self.line.cmp(&other.line);
        if let Ordering::Equal = cmp {
            return self.char.cmp(&other.char);
        }
        return cmp;
    }
}

#[derive(Debug, PartialEq)]
struct Tokenizer {
    tokens: Vec<UnprocessedSemanticToken>,
}

#[derive(Debug, PartialEq)]
struct SpelTokenCollector<'a> {
    tokenizer: &'a mut Tokenizer,
    offset_line: u32,
    offset_char: u32,
}

impl Tokenizer {
    fn new() -> Self {
        return Tokenizer { tokens: vec![] };
    }

    fn add(
        &mut self,
        char: u32,
        line: u32,
        length: u32,
        r#type: &SemanticTokenType,
        modifiers: &Vec<SemanticTokenModifier>,
    ) {
        self.tokens.push(UnprocessedSemanticToken {
            char,
            line,
            length,
            token_type: capabilities::TOKEN_TYPES
                .iter()
                .enumerate()
                .find_map(|(index, token_type)| match token_type == r#type {
                    true => Some(index as u32),
                    false => None,
                })
                .expect(&format!("no token type \"{}\" found", r#type.as_str())),
            token_modifiers_bitset: capabilities::TOKEN_MODIFIERS
                .iter()
                .enumerate()
                .filter_map(|(index, modifier)| match modifiers.contains(modifier) {
                    true => Some(1 << index as u32),
                    false => None,
                })
                .sum::<u32>(),
        });
    }

    fn collect(&self) -> Vec<SemanticToken> {
        let mut unprocessed = self.tokens.to_vec();
        unprocessed.sort();
        let mut processed = Vec::new();
        let mut cursor_line = 0;
        let mut cursor_char = 0;
        for item in unprocessed {
            let delta_line = item.line - cursor_line;
            let delta_start = match delta_line {
                0 => item.char - cursor_char,
                _ => item.char,
            };
            processed.push(SemanticToken {
                delta_line,
                delta_start,
                length: item.length,
                token_type: item.token_type,
                token_modifiers_bitset: item.token_modifiers_bitset,
            });
            cursor_line = item.line;
            cursor_char = item.char;
        }
        return processed;
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
                log::error!("failed to read {:?}: {}", uri, err);
                return LsError {
                    message: format!("cannot read file {:?}", uri),
                    code: ErrorCode::RequestFailed,
                };
            }),
    }?;
    let tokenizer = &mut Tokenizer::new();
    index_document(&document.tree, tokenizer);
    return Ok(tokenizer.collect());
}

fn index_document(tree: &Tree, tokenizer: &mut Tokenizer) {
    return index_nodes(&tree.nodes, tokenizer);
}

fn index_tag(tag: &SpmlTag, tokenizer: &mut Tokenizer) {
    if tag.definition().deprecated {
        tokenizer.add(
            tag.open_location().char as u32,
            tag.open_location().line as u32,
            tag.open_location().length as u32,
            &SemanticTokenType::MACRO,
            &vec![SemanticTokenModifier::DEPRECATED],
        );
    }
    for (name, attribute) in tag.spel_attributes() {
        let attribute = match attribute {
            ParsedAttribute::Valid(attribute) => attribute,
            ParsedAttribute::Erroneous(attribute, _) => attribute,
            ParsedAttribute::Unparsable(_, _) => continue,
        };
        if tag
            .definition()
            .attribute_rules
            .iter()
            .any(|rule| match rule {
                AttributeRule::Deprecated(attribute) => return *attribute == name,
                _ => false,
            })
        {
            tokenizer.add(
                attribute.key.location.char as u32,
                attribute.key.location.line as u32,
                attribute.key.location.length as u32,
                &SemanticTokenType::MACRO,
                &vec![SemanticTokenModifier::DEPRECATED],
            );
        }
        let offset = &attribute.value.opening_quote_location;
        let mut token_collector =
            SpelTokenCollector::new(tokenizer, offset.line as u32, offset.char as u32 + 1);
        match &attribute.value.spel {
            SpelAst::Comparable(SpelResult::Valid(comparable)) => {
                index_comparable(&comparable, &mut token_collector)
            }
            SpelAst::Condition(SpelResult::Valid(condition)) => {
                index_condition(&condition, &mut token_collector)
            }
            SpelAst::Expression(SpelResult::Valid(expression)) => {
                index_expression(&expression, &mut token_collector)
            }
            SpelAst::Identifier(SpelResult::Valid(identifier)) => {
                index_identifier(&identifier, &mut token_collector)
            }
            SpelAst::Object(SpelResult::Valid(object)) => {
                index_object(&object, &mut token_collector)
            }
            SpelAst::Query(SpelResult::Valid(query)) => index_query(&query, &mut token_collector),
            SpelAst::Regex(SpelResult::Valid(regex)) => index_regex(&regex, &mut token_collector),
            SpelAst::String(SpelResult::Valid(string)) => {
                index_word(&string, &mut token_collector, None, &vec![])
            }
            SpelAst::Uri(SpelResult::Valid(uri)) => index_uri(&uri, &mut token_collector),
            _ => (),
        };
    }
    if let Some(body) = tag.body() {
        index_nodes(&body.nodes, tokenizer);
    }
}

fn index_html(html: &HtmlNode, tokenizer: &mut Tokenizer) {
    for attribute in &html.attributes {
        let value = match attribute {
            ParsedAttribute::Valid(attribute) => &attribute.value,
            ParsedAttribute::Erroneous(attribute, _) => &attribute.value,
            _ => continue,
        };
        let content = match value {
            Some(value) => &value.content,
            None => continue,
        };
        match content {
            HtmlAttributeValueContent::Tag(tag) => index_parsed_spml_tag(tag, tokenizer),
            HtmlAttributeValueContent::Fragmented(fragments) => {
                for fragment in fragments {
                    if let HtmlAttributeValueFragment::Tag(tag) = fragment {
                        index_parsed_spml_tag(tag, tokenizer);
                    }
                }
            }
            _ => continue,
        }
    }
    if let Some(body) = html.body() {
        index_nodes(&body.nodes, tokenizer);
    }
}

fn index_nodes(nodes: &Vec<Node>, tokenizer: &mut Tokenizer) {
    for node in nodes {
        match node {
            Node::Tag(tag) => index_parsed_spml_tag(tag, tokenizer),
            Node::Html(ParsedHtml::Valid(html)) => index_html(html, tokenizer),
            Node::Html(ParsedHtml::Erroneous(html, _)) => index_html(html, tokenizer),
            _ => (),
        };
    }
}

fn index_parsed_spml_tag(tag: &ParsedTag<SpmlTag>, tokenizer: &mut Tokenizer) {
    match tag {
        ParsedTag::Valid(tag) => index_tag(tag, tokenizer),
        ParsedTag::Erroneous(tag, _) => index_tag(tag, tokenizer),
        _ => (),
    };
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

    use super::{SpelTokenCollector, Tokenizer};
    use spel::ast::{Location, Object, StringLiteral, Word, WordFragment};

    #[test]
    fn test_index_single_object() {
        let tokenizer = &mut Tokenizer::new();
        let root_object = Object::Name(Word {
            fragments: vec![WordFragment::String(StringLiteral {
                content: "_someVariable".into(),
                location: Location::VariableLength {
                    char: 0,
                    line: 0,
                    length: 13,
                },
            })],
        });
        super::index_object(
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
                token_type: capabilities::TOKEN_TYPES
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
                content: "_someVariable".into(),
                location: Location::VariableLength {
                    char: 0,
                    line: 0,
                    length: 13,
                },
            })],
        });
        super::index_object(
            &root_object,
            &mut SpelTokenCollector {
                tokenizer,
                offset_line: 8,
                offset_char: 14,
            },
        );
        super::index_object(
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
                    token_type: capabilities::TOKEN_TYPES
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
                    token_type: capabilities::TOKEN_TYPES
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

    #[test]
    fn test_index_out_of_order() {
        let tokenizer = &mut Tokenizer::new();
        tokenizer.add(0, 0, 3, &SemanticTokenType::OPERATOR, &vec![]);
        tokenizer.add(14, 0, 3, &SemanticTokenType::OPERATOR, &vec![]);
        tokenizer.add(4, 0, 9, &SemanticTokenType::VARIABLE, &vec![]);
        assert_eq!(
            tokenizer.collect(),
            vec![
                SemanticToken {
                    delta_start: 0,
                    delta_line: 0,
                    length: 3,
                    token_type: capabilities::TOKEN_TYPES
                        .iter()
                        .enumerate()
                        .find_map(|(index, token_type)| {
                            match *token_type == SemanticTokenType::OPERATOR {
                                true => Some(index as u32),
                                false => None,
                            }
                        })
                        .expect("no operator token exists"),
                    token_modifiers_bitset: 0,
                },
                SemanticToken {
                    delta_start: 4,
                    delta_line: 0,
                    length: 9,
                    token_type: capabilities::TOKEN_TYPES
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
                    delta_start: 10,
                    delta_line: 0,
                    length: 3,
                    token_type: capabilities::TOKEN_TYPES
                        .iter()
                        .enumerate()
                        .find_map(|(index, token_type)| {
                            match *token_type == SemanticTokenType::OPERATOR {
                                true => Some(index as u32),
                                false => None,
                            }
                        })
                        .expect("no operator token exists"),
                    token_modifiers_bitset: 0,
                }
            ],
        );
    }
}
