use std::collections::HashMap;

use lsp_types::{
    CodeAction, CodeActionOrCommand, CodeActionParams, Position, Range, TextEdit, Url,
    WorkspaceEdit,
};
use tree_sitter::{Node, Point};

use crate::{
    command::ResponseErrorCode,
    document_store::{self, Document},
    spel::{
        ast::{Argument, Comparable, ComparissonOperator, Condition, ConditionAst, Function},
        parser::Parser,
    },
    CodeActionImplementation,
};

use super::LsError;

const DEFAULT_HEADER: &str = concat!(
    "<%@ page language=\"java\" pageEncoding=\"UTF-8\" contentType=\"text/html; charset=UTF-8\"\n",
    "%><%@ taglib uri=\"http://www.sitepark.com/taglibs/core\" prefix=\"sp\"\n",
    "%><%@ taglib tagdir=\"/WEB-INF/tags/spt\" prefix=\"spt\"\n",
    "%><%@ taglib tagdir=\"/WEB-INF/tags/tag\" prefix=\"tag\"\n",
    "%>\n"
);

pub(crate) fn action(params: CodeActionParams) -> Result<Vec<CodeActionOrCommand>, LsError> {
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
    let mut actions = Vec::new();
    let diagnostics = params.context.diagnostics;
    if diagnostics.len() > 0 {
        log::debug!("code action request carried diagnostics: {:?}", diagnostics);
        match diagnostics[0].code {
            Some(CodeActionImplementation::GENERATE_DEFAULT_HEADER_CODE) => {
                actions.push(construct_generate_default_header(&uri))
            }
            _ => {}
        }
    }
    let node = document.tree.root_node().descendant_for_point_range(
        Point {
            row: params.range.start.line as usize,
            column: params.range.start.character as usize,
        },
        Point {
            row: params.range.end.line as usize,
            column: params.range.end.character as usize,
        },
    );
    match node {
        Some(node) => match node.kind() {
            "if_tag_open" => {
                let attributes = collect_attributes(node);
                if let Some(action) = construct_name_to_condition(&document, &uri, &attributes) {
                    actions.push(action);
                }
                if let Some(action) = construct_condition_to_name(&document, &uri, &attributes) {
                    actions.push(action);
                }
            }
            _ => {}
        },
        None => {}
    };
    return Ok(actions);
}

fn collect_attributes<'a>(mut node: Node<'a>) -> HashMap<&'a str, Node<'a>> {
    let mut attributes = HashMap::new();
    loop {
        if let Some(sibling) = node
            .next_sibling()
            .filter(|n| n.kind().ends_with("_attribute"))
        {
            if let Some(value) = sibling.child(0) {
                attributes.insert(value.kind(), sibling);
                node = sibling;
                continue;
            }
        }
        return attributes;
    }
}

fn construct_generate_default_header<'a>(uri: &Url) -> CodeActionOrCommand {
    let document_start = Position {
        line: 0,
        character: 0,
    };
    return CodeActionOrCommand::CodeAction(CodeAction {
        title: "generate default header".to_string(),
        kind: Some(CodeActionImplementation::GenerateDefaultHeaders.to_kind()),
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(
                uri.clone(),
                vec![TextEdit {
                    range: Range {
                        start: document_start,
                        end: document_start,
                    },
                    new_text: DEFAULT_HEADER.to_string(),
                }],
            )])),
            ..WorkspaceEdit::default()
        }),
        ..CodeAction::default()
    });
}

fn construct_name_to_condition<'a>(
    document: &Document,
    uri: &Url,
    attributes: &HashMap<&'a str, Node<'a>>,
) -> Option<CodeActionOrCommand> {
    if let Some(name_node) = attributes.get("name") {
        if let Some((operator, value_node)) = attributes.iter().find_map(|(k, v)| match *k {
            k @ ("gt" | "gte" | "lt" | "lte" | "eq" | "neq" | "isNull") => Some((k, v)),
            _ => None,
        }) {
            if let Some(name) = name_node
                .named_child(0)
                .and_then(|n| n.named_child(0))
                .and_then(|n| n.utf8_text(document.text.as_bytes()).ok())
            {
                if let Some(value) = value_node
                    .named_child(0)
                    .and_then(|n| n.named_child(0))
                    .and_then(|n| n.utf8_text(document.text.as_bytes()).ok())
                {
                    let new_text = match operator {
                        "isNull" if value == "true" => {
                            format!("condition=\"isNull(${{{}}})\"", name)
                        }
                        "isNull" if value == "false" => {
                            format!("condition=\"!isNull(${{{}}})\"", name)
                        }
                        "isNull" => format!("condition=\"isNull(${{{}}}) == {}\"", name, value),
                        "gt" => format!("condition=\"${{{}}} > {}\"", name, value),
                        "gte" => format!("condition=\"${{{}}} >= {}\"", name, value),
                        "lt" => format!("condition=\"${{{}}} < {}\"", name, value),
                        "lte" => format!("condition=\"${{{}}} <= {}\"", name, value),
                        "neq" => format!("condition=\"${{{}}} != {}\"", name, value),
                        _ => format!("condition=\"${{{}}} == {}\"", name, value),
                    };
                    return Some(CodeActionOrCommand::CodeAction(CodeAction {
                        title: format!("transform \"name\" and \"{}\" to \"condition\"", operator),
                        kind: Some(CodeActionImplementation::NameToCondition.to_kind()),
                        edit: Some(WorkspaceEdit {
                            changes: Some(HashMap::from([(
                                uri.clone(),
                                vec![
                                    TextEdit {
                                        range: Range {
                                            start: Position {
                                                line: value_node.start_position().row as u32,
                                                character: value_node.start_position().column
                                                    as u32
                                                    - 1,
                                            },
                                            end: point_to_position(&value_node.end_position()),
                                        },
                                        new_text: "".to_string(),
                                    },
                                    TextEdit {
                                        range: node_range(&name_node),
                                        new_text,
                                    },
                                ],
                            )])),
                            ..WorkspaceEdit::default()
                        }),
                        ..CodeAction::default()
                    }));
                }
            }
        };
    }
    return None;
}

fn construct_condition_to_name<'a>(
    document: &Document,
    uri: &Url,
    attributes: &HashMap<&'a str, Node<'a>>,
) -> Option<CodeActionOrCommand> {
    if let Some(condition_node) = attributes.get("condition") {
        if let Some(condition) = condition_node
            .named_child(0)
            .and_then(|n| n.named_child(0))
            .and_then(|n| n.utf8_text(document.text.as_bytes()).ok())
        {
            let parser = &mut Parser::new(condition);
            match parser.parse_condition_ast() {
                Ok(ConditionAst {
                    root:
                        Condition::Comparisson {
                            left,
                            operator,
                            right,
                            ..
                        },
                }) => {
                    let operator_name;
                    let new_text;
                    if let Comparable::Object(inner) = *left {
                        operator_name = match operator {
                            ComparissonOperator::Equal => "eq",
                            ComparissonOperator::Unequal => "neq",
                            ComparissonOperator::GreaterThan => "gt",
                            ComparissonOperator::GreaterThanOrEqual => "gte",
                            ComparissonOperator::LessThan => "lt",
                            ComparissonOperator::LessThanOrEqual => "lte",
                        };
                        new_text = format!(
                            "name=\"{}\" {}=\"{}\"",
                            inner.content, operator_name, *right
                        );
                    } else if let Comparable::Object(inner) = *right {
                        operator_name = match operator {
                            ComparissonOperator::Equal => "eq",
                            ComparissonOperator::Unequal => "neq",
                            ComparissonOperator::GreaterThan => "lt",
                            ComparissonOperator::GreaterThanOrEqual => "lte",
                            ComparissonOperator::LessThan => "gt",
                            ComparissonOperator::LessThanOrEqual => "gte",
                        };
                        new_text =
                            format!("name=\"{}\" {}=\"{}\"", inner.content, operator_name, *left);
                    } else {
                        return None;
                    }
                    return Some(CodeActionOrCommand::CodeAction(CodeAction {
                        title: format!(
                            "transform \"condition\" to \"name\" and \"{}\"",
                            operator_name
                        ),
                        kind: Some(CodeActionImplementation::ConditionToName.to_kind()),
                        edit: Some(WorkspaceEdit {
                            changes: Some(HashMap::from([(
                                uri.clone(),
                                vec![TextEdit {
                                    range: node_range(&condition_node),
                                    new_text,
                                }],
                            )])),
                            ..WorkspaceEdit::default()
                        }),
                        ..CodeAction::default()
                    }));
                }
                Ok(ConditionAst { root }) => {
                    return parse_is_null(&root).map(|(name, value)| {
                        CodeActionOrCommand::CodeAction(CodeAction {
                            title: "transform \"condition\" to \"name\" and \"isNull\"".to_string(),
                            kind: Some(CodeActionImplementation::ConditionToName.to_kind()),
                            edit: Some(WorkspaceEdit {
                                changes: Some(HashMap::from([(
                                    uri.clone(),
                                    vec![TextEdit {
                                        range: node_range(&condition_node),
                                        new_text: format!("name=\"{}\" isNull=\"{}\"", name, value),
                                    }],
                                )])),
                                ..WorkspaceEdit::default()
                            }),
                            ..CodeAction::default()
                        })
                    })
                }
                _ => return None,
            };
        }
    }
    return None;
}

fn parse_is_null(root: &Condition) -> Option<(String, String)> {
    if let Some(argument) = is_null_argument(root) {
        return Some((argument, "true".to_string()));
    }
    match root {
        Condition::NegatedCondition { condition, .. } => {
            is_null_argument(condition).map(|argument| (argument, "false".to_string()))
        }
        Condition::Comparisson {
            left,
            // TODO: negate opposite value if operator is unequal
            operator: _operator @ /*(*/ ComparissonOperator::Equal, /*| ComparissonOperator::Unequal)*/
            right,
            ..
        } => match &**left {
            Comparable::Condition(condition) => {
                is_null_argument(&*condition).map(|argument| (argument, right.to_string()))
            }
            _ => match &**right {
                Comparable::Condition(condition) => {
                    is_null_argument(&*condition).map(|argument| (argument, left.to_string()))
                }
                _ => None,
            },
        },
        _ => None,
    }
}

fn is_null_argument(root: &Condition) -> Option<String> {
    match root {
        Condition::Function(Function {
            name, arguments, ..
        }) if arguments.len() == 1 && name == "isNull" => match &arguments[0].argument {
            Argument::Object(interpolation) => Some(interpolation.content.to_string()),
            argument => Some(argument.to_string()),
        },
        _ => None,
    }
}

fn node_range(node: &Node<'_>) -> Range {
    return Range {
        start: point_to_position(&node.start_position()),
        end: point_to_position(&node.end_position()),
    };
}

fn point_to_position(point: &Point) -> Position {
    return Position {
        line: point.row as u32,
        character: point.column as u32,
    };
}
