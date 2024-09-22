use std::collections::HashMap;

use lsp_server::ErrorCode;
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, Position, Range, TextEdit,
    Uri, WorkspaceEdit,
};

use crate::{
    capabilities::CodeActionImplementation,
    document_store,
    parser::{Node, SpIf, SpelAttribute, Tag},
    spel::ast::{
        Argument, Comparable, ComparissonOperator, Condition, Function, SpelAst, SpelResult,
    },
};

use super::LsError;

const DEFAULT_HEADER: &str = concat!(
    "<%@ page language=\"java\" pageEncoding=\"UTF-8\" contentType=\"text/html; charset=UTF-8\"\n",
    "%><%@ taglib uri=\"http://www.sitepark.com/taglibs/core\" prefix=\"sp\"\n",
    "%><%@ taglib tagdir=\"/WEB-INF/tags/spt\" prefix=\"spt\"\n",
    "%>\n"
);

pub(crate) fn action(params: CodeActionParams) -> Result<Vec<CodeActionOrCommand>, LsError> {
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
    let mut actions = Vec::new();
    if params
        .context
        .only
        .is_some_and(|kinds| kinds.contains(&CodeActionKind::SOURCE_FIX_ALL))
    {
        let edits = params
            .context
            .diagnostics
            .iter()
            .filter_map(|diagnostic| match diagnostic.code {
                Some(CodeActionImplementation::FIX_SPEL_SYNTAX_CODE) => diagnostic
                    .data
                    .as_ref()
                    .and_then(|data| serde_json::from_value::<Vec<TextEdit>>(data.to_owned()).ok()),
                _ => None,
            })
            .flatten()
            .collect::<Vec<TextEdit>>();
        if edits.len() > 0 {
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "fix all spel syntax errors".to_string(),
                kind: Some(CodeActionKind::SOURCE_FIX_ALL),
                edit: Some(WorkspaceEdit {
                    changes: Some(HashMap::from([(uri.clone(), edits)])),
                    ..WorkspaceEdit::default()
                }),
                ..CodeAction::default()
            }));
        }
    } else {
        for diagnostic in params.context.diagnostics {
            match diagnostic.code {
                Some(CodeActionImplementation::GENERATE_DEFAULT_HEADER_CODE) => {
                    actions.push(construct_generate_default_header(&uri))
                }
                Some(CodeActionImplementation::FIX_SPEL_SYNTAX_CODE) => {
                    diagnostic
                        .data
                        .and_then(|data| serde_json::from_value(data).ok())
                        .map(|edits| {
                            actions.push(construct_fix_spel_syntax(
                                &uri,
                                format!("quick-fix: {}", diagnostic.message),
                                edits,
                            ))
                        });
                }
                _ => (),
            }
        }
    }
    match document.tree.node_at(params.range.start) {
        Some(Node::Tag(Tag::SpIf(tag))) => {
            log::debug!("code-action triggered in sp:if");
            if let Some(action) = construct_name_to_condition(&uri, &tag) {
                log::debug!("build name-to-condition action");
                actions.push(action);
            }
            if let Some(action) = construct_condition_to_name(&uri, &tag) {
                log::debug!("build condition-to-name action");
                actions.push(action);
            }
        }
        _ => (),
    };
    return Ok(actions);
}

fn construct_generate_default_header<'a>(uri: &Uri) -> CodeActionOrCommand {
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

fn construct_fix_spel_syntax<'a>(
    uri: &Uri,
    title: String,
    edits: Vec<TextEdit>,
) -> CodeActionOrCommand {
    return CodeActionOrCommand::CodeAction(CodeAction {
        title,
        kind: Some(CodeActionImplementation::GenerateDefaultHeaders.to_kind()),
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(uri.clone(), edits)])),
            ..WorkspaceEdit::default()
        }),
        ..CodeAction::default()
    });
}

fn construct_name_to_condition<'a>(uri: &Uri, if_tag: &SpIf) -> Option<CodeActionOrCommand> {
    let name_attribute = match &if_tag.name_attribute {
        Some(v) => v,
        _ => return None,
    };
    log::debug!("got name_attribute");
    let (operator, value_attribute) = match first_comparable_if_attribute(if_tag) {
        Some((o, v)) => (o, v),
        _ => return None,
    };
    log::debug!("got operator and value_attribute");
    let name = match &name_attribute.value.spel {
        SpelAst::Object(SpelResult::Valid(o)) => o,
        _ => return None,
    };
    log::debug!("got name");
    let value = match &value_attribute.value.spel {
        SpelAst::Comparable(SpelResult::Valid(c)) => c.to_string(),
        SpelAst::Condition(SpelResult::Valid(c)) => c.to_string(),
        SpelAst::String(SpelResult::Valid(c)) => format!("'{}'", c),
        x => {
            log::debug!("unexpected value: {:?}", x);
            return None;
        }
    };
    log::debug!("got value");
    let new_condition = match operator {
        "isNull" if value == "true" => format!("isNull(${{{}}})", name),
        "isNull" if value == "false" => format!("!isNull(${{{}}})", name),
        "isNull" => format!("isNull(${{{}}}) == {}", name, value),
        "gt" => format!("${{{}}} > {}", name, value),
        "gte" => format!("${{{}}} >= {}", name, value),
        "lt" => format!("${{{}}} < {}", name, value),
        "lte" => format!("${{{}}} <= {}", name, value),
        "neq" => format!("${{{}}} != {}", name, value),
        _ => format!("${{{}}} == {}", name, value),
    };
    log::debug!("got new_condition");
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
                                line: value_attribute.key_location.line as u32,
                                character: value_attribute.key_location.char as u32 - 1,
                            },
                            end: Position {
                                line: value_attribute.value.closing_quote_location.line as u32,
                                character: (value_attribute.value.closing_quote_location.char
                                    + value_attribute.value.closing_quote_location.length)
                                    as u32,
                            },
                        },
                        new_text: "".to_string(),
                    },
                    TextEdit {
                        range: Range {
                            start: Position {
                                line: name_attribute.key_location.line as u32,
                                character: name_attribute.key_location.char as u32,
                            },
                            end: Position {
                                line: name_attribute.value.closing_quote_location.line as u32,
                                character: (name_attribute.value.closing_quote_location.char
                                    + name_attribute.value.closing_quote_location.length)
                                    as u32,
                            },
                        },
                        new_text: format!("condition=\"{}\"", new_condition),
                    },
                ],
            )])),
            ..WorkspaceEdit::default()
        }),
        ..CodeAction::default()
    }));
}

fn first_comparable_if_attribute(if_tag: &SpIf) -> Option<(&str, &SpelAttribute)> {
    match &if_tag.gt_attribute {
        Some(attribute) => return Some(("gt", attribute)),
        None => (),
    };
    match &if_tag.gte_attribute {
        Some(attribute) => return Some(("gte", attribute)),
        None => (),
    };
    match &if_tag.lt_attribute {
        Some(attribute) => return Some(("lt", attribute)),
        None => (),
    };
    match &if_tag.lte_attribute {
        Some(attribute) => return Some(("lte", attribute)),
        None => (),
    };
    match &if_tag.eq_attribute {
        Some(attribute) => return Some(("eq", attribute)),
        None => (),
    };
    match &if_tag.neq_attribute {
        Some(attribute) => return Some(("neq", attribute)),
        None => (),
    };
    match &if_tag.isNull_attribute {
        Some(attribute) => return Some(("isNull", attribute)),
        None => (),
    };
    return None;
}

fn construct_condition_to_name<'a>(uri: &Uri, if_tag: &SpIf) -> Option<CodeActionOrCommand> {
    let condition_attribute = match &if_tag.condition_attribute {
        Some(v) => v,
        None => return None,
    };
    let condition = match &condition_attribute.value.spel {
        SpelAst::Condition(SpelResult::Valid(c)) => c,
        _ => return None,
    };
    match condition {
        Condition::Comparisson {
            left,
            operator,
            right,
            ..
        } => {
            let operator_name;
            let new_text;
            match (&**left, &**right) {
                (Comparable::Object(inner), _) => {
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
                }
                (_, Comparable::Object(inner)) => {
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
                }
                _ => return None,
            };
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
                            range: Range {
                                start: Position {
                                    line: condition_attribute.key_location.line as u32,
                                    character: condition_attribute.key_location.char as u32,
                                },
                                end: Position {
                                    line: condition_attribute.value.closing_quote_location.line
                                        as u32,
                                    character: (condition_attribute
                                        .value
                                        .closing_quote_location
                                        .char
                                        + condition_attribute.value.closing_quote_location.length)
                                        as u32,
                                },
                            },
                            new_text,
                        }],
                    )])),
                    ..WorkspaceEdit::default()
                }),
                ..CodeAction::default()
            }));
        }
        condition => {
            return parse_is_null(&condition).map(|(name, value)| {
                CodeActionOrCommand::CodeAction(CodeAction {
                    title: "transform \"condition\" to \"name\" and \"isNull\"".to_string(),
                    kind: Some(CodeActionImplementation::ConditionToName.to_kind()),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(
                            uri.clone(),
                            vec![TextEdit {
                                range: Range {
                                    start: Position {
                                        line: condition_attribute.key_location.line as u32,
                                        character: condition_attribute.key_location.char as u32,
                                    },
                                    end: Position {
                                        line: condition_attribute.value.closing_quote_location.line
                                            as u32,
                                        character: (condition_attribute
                                            .value
                                            .closing_quote_location
                                            .char
                                            + condition_attribute
                                                .value
                                                .closing_quote_location
                                                .length)
                                            as u32,
                                    },
                                },
                                new_text: format!("name=\"{}\" isNull=\"{}\"", name, value),
                            }],
                        )])),
                        ..WorkspaceEdit::default()
                    }),
                    ..CodeAction::default()
                })
            })
        }
    }
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
