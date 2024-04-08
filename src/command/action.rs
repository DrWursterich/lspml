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
        ast::{Comparable, ComparissonOperator, Condition, ConditionAst},
        parser::Parser,
    },
    CODE_ACTIONS,
};

use super::LsError;

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
    let mut actions = Vec::new();
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
        match node.next_sibling().filter(|n| n.kind() != ">") {
            Some(sibling) => {
                attributes.insert(sibling.child(0).unwrap().kind(), sibling);
                node = sibling;
            }
            None => break,
        };
    }
    return attributes;
}

fn construct_name_to_condition<'a>(
    document: &Document,
    uri: &Url,
    attributes: &HashMap<&'a str, Node<'a>>,
) -> Option<CodeActionOrCommand> {
    if let Some(name_node) = attributes.get("name") {
        if let Some((operator, value_node)) = attributes.iter().find_map(|(k, v)| match *k {
            k @ ("gt" | "gte" | "lt" | "lte" | "eq" | "neq") => Some((k, v)),
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
                    return Some(CodeActionOrCommand::CodeAction(CodeAction {
                        title: format!("transform \"name\" and \"{}\" to \"condition\"", operator),
                        kind: Some(CODE_ACTIONS[0].clone()),
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
                                            end: Position {
                                                line: value_node.end_position().row as u32,
                                                character: value_node.end_position().column as u32,
                                            },
                                        },
                                        new_text: "".to_string(),
                                    },
                                    TextEdit {
                                        range: Range {
                                            start: Position {
                                                line: name_node.start_position().row as u32,
                                                character: name_node.start_position().column as u32,
                                            },
                                            end: Position {
                                                line: name_node.end_position().row as u32,
                                                character: name_node.end_position().column as u32,
                                            },
                                        },
                                        new_text: format!(
                                            "condition=\"${{{}}} {} {}\"",
                                            name,
                                            match operator {
                                                "gt" => ">",
                                                "gte" => ">=",
                                                "lt" => "<",
                                                "lte" => "<=",
                                                "neq" => "!=",
                                                _ => "==",
                                            },
                                            value
                                        ),
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
            if let Ok(ConditionAst {
                root:
                    Condition::Comparisson {
                        left,
                        operator,
                        right,
                        ..
                    },
            }) = parser.parse_condition_ast()
            {
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
                    kind: Some(CODE_ACTIONS[1].clone()),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(
                            uri.clone(),
                            vec![TextEdit {
                                range: Range {
                                    start: Position {
                                        line: condition_node.start_position().row as u32,
                                        character: condition_node.start_position().column as u32,
                                    },
                                    end: Position {
                                        line: condition_node.end_position().row as u32,
                                        character: condition_node.end_position().column as u32,
                                    },
                                },
                                new_text,
                            }],
                        )])),
                        ..WorkspaceEdit::default()
                    }),
                    ..CodeAction::default()
                }));
            };
        }
    }
    return None;
}
