use super::{LsError, ResponseErrorCode};
use crate::document_store;
use crate::grammar;
use crate::project;
use anyhow::Result;
use lsp_types::{Diagnostic, DiagnosticSeverity, DocumentDiagnosticParams, Position, Range};
use std::{collections::HashMap, path::Path, str::FromStr};
use tree_sitter::Node;

pub(crate) fn diagnostic(params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>, LsError> {
    let document = match document_store::get(&params.text_document.uri) {
        Some(document) => Ok(document),
        None => document_store::Document::new(&params.text_document.uri)
            .map(|document| document_store::put(&params.text_document.uri, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", params.text_document.uri, err);
                return LsError {
                    message: format!("cannot read file {}", params.text_document.uri),
                    code: ResponseErrorCode::RequestFailed,
                };
            }),
    }?;
    let working_directory = project::get_working_directory(&params.text_document.uri)
        .expect("cannot determine module - requires path to be <module>/src/main/webapp/");
    let mut diagnositcs: Vec<Diagnostic> = Vec::new();
    let root = document.tree.root_node();
    validate_document(root, &document.text, &mut diagnositcs, &working_directory).map_err(
        |err| LsError {
            message: format!("failed to validate document: {}", err),
            code: ResponseErrorCode::RequestFailed,
        },
    )?;
    return Ok(diagnositcs);
}

fn validate_document(
    root: Node,
    text: &String,
    diagnositcs: &mut Vec<Diagnostic>,
    working_directory: &project::WorkingDirectory,
) -> Result<()> {
    for node in root.children(&mut root.walk()) {
        match node.kind() {
            "page_header" | "import_header" | "taglib_header" | "html_doctype" | "text"
            | "comment" => continue,
            "ERROR" => diagnositcs.push(Diagnostic {
                source: Some("lspml".to_string()),
                message: format!(
                    "unexpected \"{}\"",
                    node.utf8_text(text.as_bytes()).unwrap()
                ),
                range: node_range(node),
                severity: Some(DiagnosticSeverity::ERROR),
                ..Default::default()
            }),
            "html_tag" | "html_option_tag" | "html_void_tag" | "xml_comment" | "java_tag"
            | "script_tag" | "style_tag" => {
                validate_children(node, &text, diagnositcs, working_directory)?
            }
            _ => {
                let _ = &grammar::Tag::from_str(node.kind()).and_then(|tag| {
                    validate_tag(
                        tag.properties(),
                        node,
                        &text,
                        diagnositcs,
                        working_directory,
                    )
                })?;
            }
        }
    }
    return Ok(());
}

fn validate_tag(
    tag: grammar::TagProperties,
    node: Node,
    text: &String,
    diagnositcs: &mut Vec<Diagnostic>,
    working_directory: &project::WorkingDirectory,
) -> Result<()> {
    let mut attributes: HashMap<String, String> = HashMap::new();
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "ERROR" => diagnositcs.push(Diagnostic {
                message: format!(
                    "unexpected \"{}\"",
                    child.utf8_text(text.as_bytes()).unwrap()
                ),
                severity: Some(DiagnosticSeverity::ERROR),
                range: node_range(child),
                source: Some("lspml".to_string()),
                ..Default::default()
            }),
            "text" => {
                // TODO: what tags can/cannot have text?
            }
            "html_tag" | "html_option_tag" | "html_void_tag" | "java_tag" | "script_tag"
            | "style_tag" => validate_children(child, text, diagnositcs, working_directory)?,
            kind if kind.ends_with("_attribute") => {
                let attribute = &kind[..kind.find("_attribute").unwrap()].to_string();
                if attributes.contains_key(attribute) {
                    diagnositcs.push(Diagnostic {
                        message: format!(
                            "duplicate {} attribute",
                            child.child(0).unwrap().utf8_text(text.as_bytes()).unwrap()
                        ),
                        severity: Some(DiagnosticSeverity::WARNING),
                        range: node_range(child),
                        source: Some("lspml".to_string()),
                        ..Default::default()
                    });
                } else {
                    let quoted_value = child
                        .child(2)
                        .unwrap()
                        .utf8_text(text.as_bytes())
                        .unwrap()
                        .to_string();
                    attributes.insert(
                        attribute.to_string(),
                        quoted_value[1..quoted_value.len() - 1].to_string(),
                    );
                }
            }
            kind if kind.ends_with("_tag") => {
                let child_tag = &grammar::Tag::from_str(kind).unwrap();
                if can_have_child(&tag, child_tag) {
                    validate_tag(
                        child_tag.properties(),
                        child,
                        text,
                        diagnositcs,
                        working_directory,
                    )?;
                } else {
                    diagnositcs.push(Diagnostic {
                        message: format!("unexpected {} tag", &kind[..kind.find("_tag").unwrap()]),
                        severity: Some(DiagnosticSeverity::WARNING),
                        range: node_range(child),
                        source: Some("lspml".to_string()),
                        ..Default::default()
                    });
                }
            }
            _ => validate_children(child, text, diagnositcs, working_directory)?,
        }
    }
    for rule in tag.attribute_rules {
        match rule {
            grammar::AttributeRule::Deprecated(name) if attributes.contains_key(*name) => {
                diagnositcs.push(Diagnostic {
                    message: format!("attribute {} is deprecated", name),
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
            }
            grammar::AttributeRule::AtleastOneOf(names)
                if !names.iter().any(|name| attributes.contains_key(*name)) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!(
                        "requires atleast one of these attributes: {}",
                        names.join(", ")
                    ),
                    severity: Some(DiagnosticSeverity::ERROR),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
            }
            grammar::AttributeRule::ExactlyOneOf(names) => {
                let present: Vec<&str> = names
                    .iter()
                    .map(|name| *name)
                    .filter(|name| attributes.contains_key(*name))
                    .collect();
                match present.len() {
                    0 => {
                        diagnositcs.push(Diagnostic {
                            message: format!(
                                "requires one of these attributes: {}",
                                names.join(", ")
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            range: node_range(node),
                            source: Some("lspml".to_string()),
                            ..Default::default()
                        });
                    }
                    1 => {}
                    _ => {
                        diagnositcs.push(Diagnostic {
                            message: format!(
                                "requires only one of these attributes: {}",
                                present.join(", ")
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            range: node_range(node),
                            source: Some("lspml".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
            grammar::AttributeRule::OnlyOneOf(names) => {
                let present: Vec<&str> = names
                    .iter()
                    .map(|name| *name)
                    .filter(|name| attributes.contains_key(*name))
                    .collect();
                if present.len() > 1 {
                    diagnositcs.push(Diagnostic {
                        message: format!(
                            "can only have one of these attributes: {}",
                            present.join(", ")
                        ),
                        severity: Some(DiagnosticSeverity::WARNING),
                        range: node_range(node),
                        source: Some("lspml".to_string()),
                        ..Default::default()
                    });
                }
            }
            grammar::AttributeRule::OnlyWith(name1, name2)
                if attributes.contains_key(*name1) && !attributes.contains_key(*name2) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!("attribute {} is useless without attribute {}", name1, name2),
                    severity: Some(DiagnosticSeverity::WARNING),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
            }
            grammar::AttributeRule::OnlyWithEither(name, names)
                if attributes.contains_key(*name)
                    && !names.iter().any(|name| attributes.contains_key(*name)) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!(
                        "attribute {} is useless without one of these attributes: {}",
                        name,
                        names.join(", ")
                    ),
                    severity: Some(DiagnosticSeverity::WARNING),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
            }
            grammar::AttributeRule::Required(name) if !attributes.contains_key(*name) => {
                diagnositcs.push(Diagnostic {
                    message: format!("missing required attribute {}", name),
                    severity: Some(DiagnosticSeverity::ERROR),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
            }
            grammar::AttributeRule::UriExists(uri_name, module_name) => {
                if let Some(uri) = attributes.get(*uri_name) {
                    let module = match attributes.get(*module_name).map(|str| str.as_str()) {
                        Some("${module.id}") | None => working_directory.module.as_str(),
                        Some(module) => module,
                    };
                    let file = format!(
                        "{}{}/src/main/webapp{}",
                        working_directory.path, module, uri
                    );
                    if !Path::new(&file).exists() {
                        diagnositcs.push(Diagnostic {
                            message: format!("included file {} does not exist", file),
                            severity: Some(DiagnosticSeverity::ERROR),
                            range: node_range(node),
                            source: Some("lspml".to_string()),
                            ..Default::default()
                        });
                    }
                    // else check if arguments are ok?
                }
            }
            _ => {}
        }
    }
    return Ok(());
}

fn can_have_child(tag: &grammar::TagProperties, child: &grammar::Tag) -> bool {
    return match &tag.children {
        grammar::TagChildren::Any => true,
        grammar::TagChildren::None => false,
        grammar::TagChildren::Scalar(tag) => child == tag,
        grammar::TagChildren::Vector(tags) => tags.iter().any(|tag| child == tag),
    };
}

fn validate_children(
    node: Node,
    text: &String,
    diagnositcs: &mut Vec<Diagnostic>,
    working_directory: &project::WorkingDirectory,
) -> Result<()> {
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "ERROR" => diagnositcs.push(Diagnostic {
                message: format!(
                    "unexpected \"{}\"",
                    child.utf8_text(text.as_bytes()).unwrap()
                ),
                severity: Some(DiagnosticSeverity::ERROR),
                range: node_range(child),
                source: Some("lspml".to_string()),
                ..Default::default()
            }),
            "text" => {
                // TODO: what tags can/cannot have text?
            }
            "html_tag" | "html_option_tag" | "html_void_tag" | "java_tag" | "script_tag"
            | "style_tag" => {
                validate_children(child, text, diagnositcs, working_directory)?;
            }
            kind if kind.ends_with("_tag") => {
                let child_tag = &grammar::Tag::from_str(kind).unwrap();
                validate_tag(
                    child_tag.properties(),
                    child,
                    text,
                    diagnositcs,
                    working_directory,
                )?;
            }
            _ => validate_children(child, text, diagnositcs, working_directory)?,
        }
    }
    return Ok(());
}

fn node_range(node: Node) -> Range {
    return Range {
        start: Position {
            line: node.start_position().row as u32,
            character: node.start_position().column as u32,
        },
        end: Position {
            line: node.end_position().row as u32,
            character: node.end_position().column as u32,
        },
    };
}
