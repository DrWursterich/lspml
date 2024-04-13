use std::{collections::HashMap, path::Path, str::FromStr};

use anyhow::Result;
use lsp_types::{
    Diagnostic, DiagnosticSeverity, DiagnosticTag, DocumentDiagnosticParams, Position, Range, Url,
};
use tree_sitter::Node;

use crate::{
    document_store, grammar, modules, parser, spel::parser::Parser, CodeActionImplementation,
};

use super::{LsError, ResponseErrorCode};

pub(crate) fn diagnostic(params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>, LsError> {
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
    let mut diagnositcs: Vec<Diagnostic> = Vec::new();
    let root = document.tree.root_node();
    validate_document(root, &document.text, &mut diagnositcs, &uri).map_err(|err| LsError {
        message: format!("failed to validate document: {}", err),
        code: ResponseErrorCode::RequestFailed,
    })?;
    return Ok(diagnositcs);
}

fn validate_document(
    root: Node,
    text: &String,
    diagnositcs: &mut Vec<Diagnostic>,
    file: &Url,
) -> Result<()> {
    validate_header(root, text, diagnositcs)?;
    for node in root.children(&mut root.walk()) {
        match node.kind() {
            "page_header" | "import_header" | "taglib_header" | "html_doctype" | "text"
            | "comment" | "xml_entity" => continue,
            "ERROR" => diagnositcs.push(Diagnostic {
                source: Some("lspml".to_string()),
                message: format!("unexpected \"{}\"", node.utf8_text(text.as_bytes())?),
                range: node_range(node),
                severity: Some(DiagnosticSeverity::ERROR),
                ..Default::default()
            }),
            "html_tag" | "html_option_tag" | "html_void_tag" | "xml_comment" | "java_tag"
            | "script_tag" | "style_tag" => validate_children(node, &text, diagnositcs, file)?,
            _ => match &grammar::Tag::from_str(node.kind()) {
                Ok(tag) => validate_tag(tag.properties(), node, &text, diagnositcs, file),
                Err(err) => {
                    log::info!(
                        "error while trying to interprete node \"{}\" as tag: {}",
                        node.kind(),
                        err
                    );
                    continue;
                }
            }?,
        }
    }
    return Ok(());
}

fn validate_header(root: Node, _text: &String, diagnositcs: &mut Vec<Diagnostic>) -> Result<()> {
    if root.kind() != "document" {
        let document_start = Position {
            line: 0,
            character: 0,
        };
        diagnositcs.push(Diagnostic {
            source: Some("lspml".to_string()),
            message: format!(
                "missing atleast one header. Try generating one with the \"{}\" code-action",
                CodeActionImplementation::GenerateDefaultHeaders
            ),
            range: Range {
                start: document_start,
                end: document_start,
            },
            code: Some(CodeActionImplementation::GENERATE_DEFAULT_HEADER_CODE),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
    }
    return Ok(());
}

fn validate_tag(
    tag: grammar::TagProperties,
    node: Node,
    text: &String,
    diagnositcs: &mut Vec<Diagnostic>,
    file: &Url,
) -> Result<()> {
    if tag.deprecated {
        diagnositcs.push(Diagnostic {
            message: format!("{} tag is deprecated", tag.name),
            severity: Some(DiagnosticSeverity::INFORMATION),
            range: node_range(node),
            source: Some("lspml".to_string()),
            tags: Some(vec![DiagnosticTag::DEPRECATED]),
            ..Default::default()
        });
    }
    let mut attributes: HashMap<String, String> = HashMap::new();
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            // may need to check on kind of missing child
            _ if child.is_missing() => diagnositcs.push(Diagnostic {
                message: format!("{} is never closed", node.kind()),
                severity: Some(DiagnosticSeverity::ERROR),
                range: node_range(node),
                source: Some("lspml".to_string()),
                ..Default::default()
            }),
            _ if child.is_error() => diagnositcs.push(Diagnostic {
                message: format!("unexpected \"{}\"", child.utf8_text(text.as_bytes())?),
                severity: Some(DiagnosticSeverity::ERROR),
                range: node_range(child),
                source: Some("lspml".to_string()),
                ..Default::default()
            }),
            "html_void_tag" | "java_tag" | "script_tag" | "style_tag" => {}
            "html_tag" | "html_option_tag" => validate_children(child, text, diagnositcs, file)?,
            kind if kind.ends_with("_attribute") => {
                let attribute = parser::attribute_name_of(child, text).to_string();
                let value = parser::attribute_value_of(child, text).to_string();
                if let grammar::TagAttributes::These(definitions) = tag.attributes {
                    if let Some(definition) = definitions
                        .iter()
                        .find(|definition| definition.name == attribute)
                    {
                        let value_node = match child.child(2).and_then(|child| child.child(1)) {
                            Some(node) => node,
                            _ => continue,
                        };
                        let parser = &mut Parser::new(value_node.utf8_text(&text.as_bytes())?);
                        match definition.r#type {
                            grammar::TagAttributeType::Comparable => {
                                match parser.parse_comparable() {
                                    Ok(_result) => {}
                                    Err(err) => {
                                        log::error!(
                                            "parse comparable \"{}\" failed: {}",
                                            value_node.utf8_text(&text.as_bytes())?,
                                            err
                                        );
                                        diagnositcs.push(Diagnostic {
                                            message: format!("invalid comparable: {}", err),
                                            severity: Some(DiagnosticSeverity::ERROR),
                                            range: node_range(value_node),
                                            source: Some("lspml".to_string()),
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                            grammar::TagAttributeType::Condition => {
                                match parser.parse_condition_ast() {
                                    Ok(_result) => {}
                                    Err(err) => {
                                        log::error!(
                                            "parse condition \"{}\" failed: {}",
                                            value_node.utf8_text(&text.as_bytes())?,
                                            err
                                        );
                                        diagnositcs.push(Diagnostic {
                                            message: format!("invalid condition: {}", err),
                                            severity: Some(DiagnosticSeverity::ERROR),
                                            range: node_range(value_node),
                                            source: Some("lspml".to_string()),
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                            grammar::TagAttributeType::Expression => {
                                match parser.parse_expression_ast() {
                                    Ok(_result) => {}
                                    Err(err) => {
                                        log::error!(
                                            "parse expression \"{}\" failed: {}",
                                            value_node.utf8_text(&text.as_bytes())?,
                                            err
                                        );
                                        diagnositcs.push(Diagnostic {
                                            message: format!("invalid expression: {}", err),
                                            severity: Some(DiagnosticSeverity::ERROR),
                                            range: node_range(value_node),
                                            source: Some("lspml".to_string()),
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                            grammar::TagAttributeType::Identifier => {
                                match parser.parse_identifier() {
                                    Ok(_result) => {}
                                    Err(err) => {
                                        log::error!(
                                            "parse identifier \"{}\" failed: {}",
                                            value_node.utf8_text(&text.as_bytes())?,
                                            err
                                        );
                                        diagnositcs.push(Diagnostic {
                                            message: format!("invalid identifier: {}", err),
                                            severity: Some(DiagnosticSeverity::ERROR),
                                            range: node_range(value_node),
                                            source: Some("lspml".to_string()),
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                            grammar::TagAttributeType::Object => match parser.parse_object_ast() {
                                Ok(_result) => {}
                                Err(err) => {
                                    log::error!(
                                        "parse object \"{}\" failed: {}",
                                        value_node.utf8_text(&text.as_bytes())?,
                                        err
                                    );
                                    diagnositcs.push(Diagnostic {
                                        message: format!("invalid object: {}", err),
                                        severity: Some(DiagnosticSeverity::ERROR),
                                        range: node_range(value_node),
                                        source: Some("lspml".to_string()),
                                        ..Default::default()
                                    });
                                }
                            },
                            grammar::TagAttributeType::Regex => match parser.parse_regex() {
                                Ok(_result) => {}
                                Err(err) => {
                                    log::error!(
                                        "parse regex \"{}\" failed: {}",
                                        value_node.utf8_text(&text.as_bytes())?,
                                        err
                                    );
                                    diagnositcs.push(Diagnostic {
                                        message: format!("invalid regex: {}", err),
                                        severity: Some(DiagnosticSeverity::ERROR),
                                        range: node_range(value_node),
                                        source: Some("lspml".to_string()),
                                        ..Default::default()
                                    });
                                }
                            },
                            grammar::TagAttributeType::String => match parser.parse_text() {
                                Ok(_result) => {}
                                Err(err) => {
                                    log::error!(
                                        "parse text \"{}\" failed: {}",
                                        value_node.utf8_text(&text.as_bytes())?,
                                        err
                                    );
                                    diagnositcs.push(Diagnostic {
                                        message: format!("invalid text: {}", err),
                                        severity: Some(DiagnosticSeverity::ERROR),
                                        range: node_range(value_node),
                                        source: Some("lspml".to_string()),
                                        ..Default::default()
                                    });
                                }
                            },
                            grammar::TagAttributeType::Query => {}
                            grammar::TagAttributeType::Uri => match parser.parse_uri() {
                                Ok(_result) => {}
                                Err(err) => {
                                    log::error!(
                                        "parse uri \"{}\" failed: {}",
                                        value_node.utf8_text(&text.as_bytes())?,
                                        err
                                    );
                                    diagnositcs.push(Diagnostic {
                                        message: format!("invalid uri: {}", err),
                                        severity: Some(DiagnosticSeverity::ERROR),
                                        range: node_range(value_node),
                                        source: Some("lspml".to_string()),
                                        ..Default::default()
                                    });
                                }
                            },
                        }
                    };
                }
                if attributes.contains_key(&attribute) {
                    diagnositcs.push(Diagnostic {
                        message: format!("duplicate {} attribute", attribute),
                        severity: Some(DiagnosticSeverity::WARNING),
                        range: node_range(child),
                        source: Some("lspml".to_string()),
                        ..Default::default()
                    });
                } else {
                    attributes.insert(attribute, value);
                }
            }
            kind if kind.ends_with("_tag") => match &grammar::Tag::from_str(kind) {
                Ok(child_tag) => {
                    if can_have_child(&tag, child_tag) {
                        validate_tag(child_tag.properties(), child, text, diagnositcs, file)?;
                    } else {
                        diagnositcs.push(Diagnostic {
                            message: format!(
                                "unexpected {} tag",
                                &kind[..kind.find("_tag").unwrap()]
                            ),
                            severity: Some(DiagnosticSeverity::WARNING),
                            range: node_range(child),
                            source: Some("lspml".to_string()),
                            ..Default::default()
                        });
                    }
                }
                Err(err) => {
                    log::info!("expected sp or spt tag: {}", err);
                }
            },
            _ => validate_children(child, text, diagnositcs, file)?,
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
                    tags: Some(vec![DiagnosticTag::DEPRECATED]),
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
            grammar::AttributeRule::ExactlyOneOfOrBody(names) => {
                let present: Vec<&str> = names
                    .iter()
                    .map(|name| *name)
                    .filter(|name| attributes.contains_key(*name))
                    .collect();
                let has_body = node
                    .child(node.child_count() - 1)
                    .is_some_and(|tag| tag.kind().ends_with("_tag_close"));
                match present.len() {
                    0 if !has_body => {
                        diagnositcs.push(Diagnostic {
                            message: format!(
                                "requires either a tag-body or one of these attributes: {}",
                                names.join(", ")
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            range: node_range(node),
                            source: Some("lspml".to_string()),
                            ..Default::default()
                        });
                    }
                    0 if has_body => {}
                    1 if !has_body => {}
                    _ => {
                        diagnositcs.push(Diagnostic {
                            message: format!(
                                "requires either a tag-body or only one of these attributes: {}",
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
            grammar::AttributeRule::ExactlyOrBody(name)
                if attributes.contains_key(*name)
                    != node
                        .child(node.child_count() - 1)
                        .is_some_and(|tag| tag.kind().ends_with("_tag_close")) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!("requires either a tag-body or the attribute {}", name,),
                    severity: Some(DiagnosticSeverity::ERROR),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
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
            grammar::AttributeRule::OnlyOneOfOrBody(names) => {
                let present: Vec<&str> = names
                    .iter()
                    .map(|name| *name)
                    .filter(|name| attributes.contains_key(*name))
                    .collect();
                let has_body = node
                    .child(node.child_count() - 1)
                    .is_some_and(|tag| tag.kind().ends_with("_tag_close"));
                if has_body {
                    if present.len() > 0 {
                        diagnositcs.push(Diagnostic {
                            message: format!(
                                "can only have either a tag-body or one of these attributes: {}",
                                present.join(", ")
                            ),
                            severity: Some(DiagnosticSeverity::WARNING),
                            range: node_range(node),
                            source: Some("lspml".to_string()),
                            ..Default::default()
                        });
                    }
                } else if present.len() > 1 {
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
            grammar::AttributeRule::OnlyOrBody(name)
                if attributes.contains_key(*name)
                    && node
                        .child(node.child_count() - 1)
                        .is_some_and(|tag| tag.kind().ends_with("_tag_close")) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!("can only have either a tag-body or the {} attribute", name),
                    severity: Some(DiagnosticSeverity::WARNING),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
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
            grammar::AttributeRule::OnlyWithEitherOrBody(name, names)
                if attributes.contains_key(*name)
                    && !names.iter().any(|name| attributes.contains_key(*name))
                    && !node
                        .child(node.child_count() - 1)
                        .is_some_and(|tag| tag.kind().ends_with("_tag_close")) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!(
                        "attribute {} is useless without either a tag-body or one of these attributes: {}",
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
                    if uri.contains("${") {
                        continue;
                    }
                    let module_value = attributes.get(*module_name).map(|str| str.as_str());
                    let module = match module_value {
                        Some("${module.id}") | None => file
                            .to_file_path()
                            .ok()
                            .and_then(|file| modules::find_module_for_file(file.as_path())),
                        Some(module) => modules::find_module_by_name(module),
                    };
                    match module {
                        Some(module) => {
                            let file = format!("{}{}", module.path, uri);
                            if !Path::new(&file).exists() {
                                diagnositcs.push(Diagnostic {
                                    message: format!("included file {} does not exist", file),
                                    severity: Some(DiagnosticSeverity::ERROR),
                                    range: node_range(node),
                                    source: Some("lspml".to_string()),
                                    ..Default::default()
                                });
                            }
                        }
                        None => {
                            diagnositcs.push(Diagnostic {
                                message: match module_value {
                                    Some("${module.id}") | None => {
                                        "current module not listed in module-file".to_string()
                                    }
                                    Some(name) => {
                                        format!("module \"{}\" not listed in module-file", name)
                                    }
                                },
                                severity: Some(DiagnosticSeverity::HINT),
                                range: node_range(node),
                                source: Some("lspml".to_string()),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
            grammar::AttributeRule::ValueOneOf(name, values)
                if attributes
                    .get(*name)
                    .is_some_and(|v| !values.contains(&v.as_str())) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!(
                        "attribute {} should be one of these values: [{}]",
                        name,
                        values.join(", ")
                    ),
                    severity: Some(DiagnosticSeverity::ERROR),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
            }
            grammar::AttributeRule::ValueOneOfCaseInsensitive(name, values)
                if attributes
                    .get(*name)
                    .is_some_and(|v| !values.contains(&v.to_uppercase().as_str())) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!(
                        "attribute {} should be one of these values: [{}]",
                        name,
                        values.join(", ")
                    ),
                    severity: Some(DiagnosticSeverity::ERROR),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
            }
            grammar::AttributeRule::OnlyWithValue(name, attribute, value)
                if attributes.contains_key(*name)
                    && !attributes.get(*attribute).is_some_and(|v| v == value) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!(
                        "attribute {} is useless without attribute {} containing the value {}",
                        name, attribute, value
                    ),
                    severity: Some(DiagnosticSeverity::ERROR),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
            }
            grammar::AttributeRule::OnlyWithEitherValue(name, attribute, values)
                if attributes.contains_key(*name)
                    && !attributes
                        .get(*attribute)
                        .is_some_and(|v| values.contains(&v.as_str())) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!(
                        "attribute {} is useless without attribute {} containing one of these values: [{}]",
                        name, attribute, values.join(", ")
                    ),
                    severity: Some(DiagnosticSeverity::ERROR),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
            }
            grammar::AttributeRule::BodyOnlyWithEitherValue(attribute, values)
                if node
                    .child(node.child_count() - 1)
                    .is_some_and(|tag| tag.kind().ends_with("_tag_close"))
                    && !attributes
                        .get(*attribute)
                        .is_some_and(|v| values.contains(&v.as_str())) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!(
                        "tag-body is useless without attribute {} containing one of these values: [{}]",
                        attribute, values.join(", ")
                    ),
                    severity: Some(DiagnosticSeverity::WARNING),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
            }
            grammar::AttributeRule::RequiredWithValue(name, attribute, value)
                if attributes.get(*attribute).is_some_and(|v| v == value)
                    && !attributes.contains_key(*name) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!(
                        "attribute {} is required when attribute {} is {}",
                        name, attribute, value
                    ),
                    severity: Some(DiagnosticSeverity::ERROR),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
            }
            grammar::AttributeRule::RequiredOrBodyWithValue(name, attribute, value)
                if attributes.get(*attribute).is_some_and(|v| v == value) =>
            {
                let has_attribute = attributes.contains_key(*name);
                let has_body = node
                    .child(node.child_count() - 1)
                    .is_some_and(|tag| tag.kind().ends_with("_tag_close"));
                if !has_attribute && !has_body {
                    diagnositcs.push(Diagnostic {
                        message: format!(
                            "either attribute {} or a tag-body is required when attribute {} is \"{}\"",
                            name,
                            attribute,
                            value
                        ),
                        severity: Some(DiagnosticSeverity::ERROR),
                        range: node_range(node),
                        source: Some("lspml".to_string()),
                        ..Default::default()
                    });
                } else if has_attribute && has_body {
                    diagnositcs.push(Diagnostic {
                        message: format!(
                            "exactly one of attribute {} or a tag-body is required when attribute {} is \"{}\"",
                            name,
                            attribute,
                            value
                        ),
                        severity: Some(DiagnosticSeverity::ERROR),
                        range: node_range(node),
                        source: Some("lspml".to_string()),
                        ..Default::default()
                    });
                }
            }
            grammar::AttributeRule::RequiredWithEitherValue(name, attribute, values)
                if attributes
                    .get(*attribute)
                    .is_some_and(|v| values.contains(&v.as_str()))
                    && !attributes.contains_key(*name) =>
            {
                diagnositcs.push(Diagnostic {
                    message: format!(
                        "attribute {} is required when attribute {} is either of [{}]",
                        name,
                        attribute,
                        values.join(", ")
                    ),
                    severity: Some(DiagnosticSeverity::ERROR),
                    range: node_range(node),
                    source: Some("lspml".to_string()),
                    ..Default::default()
                });
            }
            grammar::AttributeRule::ExactlyOneOfOrBodyWithValue(names, attribute, value)
                if attributes.get(*attribute).is_some_and(|v| v == value) =>
            {
                let present: Vec<&str> = names
                    .iter()
                    .map(|name| *name)
                    .filter(|name| attributes.contains_key(*name))
                    .collect();
                let has_body = node
                    .child(node.child_count() - 1)
                    .is_some_and(|tag| tag.kind().ends_with("_tag_close"));
                match present.len() {
                    0 if !has_body => {
                        diagnositcs.push(Diagnostic {
                            message: format!(
                                "when attribute {} is \"{}\" either a tag-body or exactly one of these attributes is required: [{}]",
                                attribute, value, names.join(", ")
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            range: node_range(node),
                            source: Some("lspml".to_string()),
                            ..Default::default()
                        });
                    }
                    0 if has_body => {}
                    1 if !has_body => {}
                    _ => {
                        diagnositcs.push(Diagnostic {
                            message: format!(
                                "when attribute {} is \"{}\" only one of a tag-body and these attributes is required: [{}]",
                                attribute, value, names.join(", ")
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            range: node_range(node),
                            source: Some("lspml".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
            grammar::AttributeRule::ExactlyOneOfOrBodyWithEitherValue(names, attribute, values)
                if attributes
                    .get(*attribute)
                    .is_some_and(|v| values.contains(&v.as_str())) =>
            {
                let present: Vec<&str> = names
                    .iter()
                    .map(|name| *name)
                    .filter(|name| attributes.contains_key(*name))
                    .collect();
                let has_body = node
                    .child(node.child_count() - 1)
                    .is_some_and(|tag| tag.kind().ends_with("_tag_close"));
                match present.len() {
                    0 if !has_body => {
                        diagnositcs.push(Diagnostic {
                            message: format!(
                                "when attribute {} is either of [{}] either a tag-body or exactly one of these attributes is required: [{}]",
                                attribute, values.join(", "), names.join(", ")
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            range: node_range(node),
                            source: Some("lspml".to_string()),
                            ..Default::default()
                        });
                    }
                    0 if has_body => {}
                    1 if !has_body => {}
                    _ => {
                        diagnositcs.push(Diagnostic {
                            message: format!(
                                "when attribute {} is either of [{}] only one of a tag-body and these attributes is required: [{}]",
                                attribute, values.join(", "), names.join(", ")
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            range: node_range(node),
                            source: Some("lspml".to_string()),
                            ..Default::default()
                        });
                    }
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
    file: &Url,
) -> Result<()> {
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "ERROR" => diagnositcs.push(Diagnostic {
                message: format!("unexpected \"{}\"", child.utf8_text(text.as_bytes())?),
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
                validate_children(child, text, diagnositcs, file)?;
            }
            kind if kind.ends_with("_tag") => match &grammar::Tag::from_str(kind) {
                Ok(child_tag) => {
                    validate_tag(child_tag.properties(), child, text, diagnositcs, file)?
                }
                Err(err) => {
                    log::info!("expected sp or spt tag: {}", err);
                }
            },
            _ => validate_children(child, text, diagnositcs, file)?,
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
