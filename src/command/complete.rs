use super::{LsError, ResponseErrorCode};
use crate::document_store;
use crate::grammar;
use anyhow::Result;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, Documentation, InsertTextMode,
    MarkupContent, MarkupKind,
};
use std::{collections::HashMap, str::FromStr};
use tree_sitter::{Node, Point};

#[derive(PartialEq)]
enum TagParsePosition {
    Attributes,
    Children,
}

#[derive(PartialEq)]
enum CompletionType {
    Attributes,
    Tags,
}

pub(crate) fn complete(params: CompletionParams) -> Result<Vec<CompletionItem>, LsError> {
    let text_params = params.text_document_position;
    let document = match document_store::get(&text_params.text_document.uri) {
        Some(document) => Ok(document),
        None => document_store::Document::new(&text_params.text_document.uri)
            .map(|document| document_store::put(&text_params.text_document.uri, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", text_params.text_document.uri, err);
                return LsError {
                    message: format!("cannot read file {}", text_params.text_document.uri),
                    code: ResponseErrorCode::RequestFailed,
                };
            }),
    }?;
    let mut completions: Vec<CompletionItem> = Vec::new();
    let root = document.tree.root_node();
    search_completions_in_document(
        root,
        &document.text,
        &Point::new(
            text_params.position.line as usize,
            text_params.position.character as usize,
        ),
        &mut completions,
    )
    .map_err(|err| LsError {
        message: format!("failed to validate document: {}", err),
        code: ResponseErrorCode::RequestFailed,
    })?;
    return Ok(completions);
}

fn search_completions_in_document(
    root: Node,
    text: &String,
    cursor: &Point,
    completions: &mut Vec<CompletionItem>,
) -> Result<()> {
    for node in root.children(&mut root.walk()) {
        if node.end_position() < *cursor {
            // also skips over "implicitly" closed tags
            log::trace!("skip over {}", node.kind());
            continue;
        }
        if node.start_position() > *cursor {
            log::trace!("{} is beyond cursor", node.kind());
            break;
        }
        // we are in a nested node:
        match node.kind() {
            // ignore for now
            "page_header" | "import_header" | "taglib_header" | "text" | "comment" => return Ok(()),
            "ERROR" => {
                log::trace!("cursor {} is in ERROR", cursor);
                return Ok(());
            }
            "html_tag" | "html_option_tag" | "html_void_tag" | "java_tag" | "script_tag"
            | "style_tag" => return Ok(()), // validate_children(node, &text, cursor, completions)?,
            kind => {
                log::trace!(
                    "cursor {} is in tag {} ({} - {})",
                    cursor,
                    kind,
                    node.start_byte(),
                    node.end_position()
                );
                return grammar::Tag::from_str(kind).and_then(|tag| {
                    search_completions_in_tag(tag.properties(), node, &text, cursor, completions)
                });
            }
        }
    }
    // we are at document level - propose all "top level" tags.
    grammar::Tag::iter()
        .map(|tag| tag.properties())
        .map(|properties| CompletionItem {
            kind: Some(CompletionItemKind::METHOD),
            detail: properties.detail.map(|detail| detail.to_string()),
            documentation: properties.documentation.map(|detail| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: detail.to_string(),
                })
            }),
            insert_text: Some(format!("<{}", properties.name.to_string())),
            insert_text_mode: Some(InsertTextMode::AS_IS),
            ..Default::default()
        })
        .for_each(|completion| completions.push(completion));
    return Ok(());
}

fn search_completions_in_tag(
    tag: grammar::TagProperties,
    node: Node,
    text: &String,
    cursor: &Point,
    completions: &mut Vec<CompletionItem>,
) -> Result<()> {
    let mut attributes: HashMap<String, String> = HashMap::new();
    let mut completion_type = CompletionType::Attributes;
    let mut position = TagParsePosition::Attributes;
    for child in node.children(&mut node.walk()) {
        if position == TagParsePosition::Children {
            if *cursor >= child.end_position() {
                log::trace!("skip over {} child {}", tag.name, node.kind());
                continue;
            }
            if *cursor < child.start_position() {
                break;
            }
            completion_type = CompletionType::Tags;
        }
        match child.kind() {
            ">" => {
                if *cursor > child.start_position() {
                    log::trace!(
                        "since cursor at {} is greater than '>' ({}) we should complete tags",
                        cursor,
                        child.start_position()
                    );
                    completion_type = CompletionType::Tags;
                } else {
                    log::trace!(
                        "since cursor at {} is less than '>' ({}) we should complete attributes",
                        cursor,
                        child.start_position()
                    );
                }
                position = TagParsePosition::Children;
            }
            "self_closing_tag_end" => {
                if child.is_missing() {
                    log::trace!("found missing '/>' in {}", tag.name);
                    break;
                }
                log::trace!("reached end of {}", tag.name);
                break;
            }
            kind if kind.ends_with("_tag_close") => {
                log::trace!("reached end of {}", tag.name);
                break;
            }
            "ERROR" => {}
            "text" => {
                // TODO: what tags can/cannot have text?
            }
            "html_tag" | "html_option_tag" | "html_void_tag" | "java_tag" | "script_tag"
            | "style_tag" => {
                log::info!(
                    "cursor seems to be inside a special tag ({}), which is not yet implemented",
                    node.kind()
                );
                // should carry over the possible children of the current tag
                // validate_children(child, text, cursor, completions)?;
            }
            kind if kind.ends_with("_attribute") => {
                position = TagParsePosition::Attributes;
                let quoted_value = child
                    .child(0)
                    .unwrap()
                    .utf8_text(text.as_bytes())
                    .unwrap()
                    .to_string();
                attributes.insert(
                    kind[..kind.find("_attribute").unwrap()].to_string(),
                    quoted_value[1..quoted_value.len()].to_string(),
                );
            }
            kind if kind.ends_with("_tag") => {
                return search_completions_in_tag(
                    grammar::Tag::from_str(kind)?.properties(),
                    child,
                    text,
                    cursor,
                    completions,
                );
            }
            kind => {
                log::info!("ignore node {}", kind);
                // validate_children(child, text, cursor, completions)?;
            }
        }
    }
    match completion_type {
        CompletionType::Attributes => {
            // complete all attributes that the tag permitts, that are not already present and are
            // not in conflict with any present ones. also propose "/>" and/or ">" if not already
            // present.
            log::info!("complete attributes for {}", tag.name);
            match tag.attributes {
                grammar::TagAttributes::These(possible)
                | grammar::TagAttributes::TheseAndDynamic(possible) => possible
                    .iter()
                    .filter(|attribute| !attributes.contains_key(attribute.name))
                    .map(|attribute| CompletionItem {
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: attribute.detail.map(|detail| detail.to_string()),
                        documentation: attribute.documentation.map(|detail| {
                            Documentation::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: detail.to_string(),
                            })
                        }),
                        insert_text: Some(format!("{}=\"", attribute.name.to_string())),
                        insert_text_mode: Some(InsertTextMode::AS_IS),
                        ..Default::default()
                    })
                    .for_each(|completion| completions.push(completion)),
                grammar::TagAttributes::None | grammar::TagAttributes::OnlyDynamic => {}
            };
        }
        CompletionType::Tags => {
            // complete all possible child tags of tag. also propose cosing tag if not already
            // present.
            log::info!("complete child tags for {}", tag.name);
            match tag.children {
                grammar::TagChildren::Any => complete_all_tags(completions)?,
                grammar::TagChildren::None => {}
                grammar::TagChildren::Scalar(tag) => completions.push(CompletionItem {
                    kind: Some(CompletionItemKind::METHOD),
                    detail: tag.properties().detail.map(|detail| detail.to_string()),
                    documentation: tag.properties().documentation.map(|detail| {
                        Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: detail.to_string(),
                        })
                    }),
                    insert_text: Some(format!("<{}", tag.properties().name.to_string())),
                    insert_text_mode: Some(InsertTextMode::AS_IS),
                    ..Default::default()
                }),
                grammar::TagChildren::Vector(tags) => tags
                    .iter()
                    .map(|tag| tag.properties())
                    .map(|properties| CompletionItem {
                        kind: Some(CompletionItemKind::METHOD),
                        detail: properties.detail.map(|detail| detail.to_string()),
                        documentation: properties.documentation.map(|detail| {
                            Documentation::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: detail.to_string(),
                            })
                        }),
                        insert_text: Some(format!("<{}", properties.name.to_string())),
                        insert_text_mode: Some(InsertTextMode::AS_IS),
                        ..Default::default()
                    })
                    .for_each(|completion| completions.push(completion)),
            }
        }
    };
    return Ok(());
}

fn complete_all_tags(completions: &mut Vec<CompletionItem>) -> Result<()> {
    grammar::Tag::iter()
        .map(|tag| tag.properties())
        .map(|properties| CompletionItem {
            kind: Some(CompletionItemKind::METHOD),
            detail: properties.detail.map(|detail| detail.to_string()),
            documentation: properties.documentation.map(|detail| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: detail.to_string(),
                })
            }),
            insert_text: Some(format!("<{}", properties.name.to_string())),
            insert_text_mode: Some(InsertTextMode::AS_IS),
            ..Default::default()
        })
        .for_each(|completion| completions.push(completion));
    return Ok(());
}
