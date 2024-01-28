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
        // tree sitter puts an 'missing' node at the end of unclosed tags, so we cannot blindly
        // skip all nodes that end before the cursor
        if node.end_position() < *cursor
            && (node.child_count() == 0
                || !node
                    .child(node.child_count() - 1)
                    .is_some_and(|close_bracket| close_bracket.is_missing()))
        {
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
            "html_tag" | "html_option_tag" | "html_void_tag" | "java_tag" | "script_tag"
            | "style_tag" => return Ok(()), // validate_children(node, &text, cursor, completions)?,
            _ if node.is_error() => match node.child(0) {
                Some(child) if child.kind().ends_with("_tag_open") => {
                    let kind = child.kind();
                    if kind == "html_tag_open" {
                        break;
                    }
                    let tag = &kind[..kind.len() - "_open".len()];
                    log::trace!(
                        "cursor {} is in ERROR which appears to be a {}",
                        cursor,
                        tag
                    );
                    return grammar::Tag::from_str(&tag).and_then(|tag| {
                        search_completions_in_tag(
                            tag.properties(),
                            node,
                            &text,
                            cursor,
                            completions,
                        )
                    });
                }
                _ => log::trace!("cursor {} is in ERROR without children", cursor),
            },
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
            // tree sitter puts an 'missing' node at the end of unclosed tags, so we cannot blindly
            // skip all nodes that end before the cursor
            if child.end_position() < *cursor
                && (child.child_count() == 0
                    || !child
                        .child(child.child_count() - 1)
                        .is_some_and(|close_bracket| close_bracket.is_missing()))
            {
                continue;
            }
            if *cursor < child.start_position() {
                break;
            }
            completion_type = CompletionType::Tags;
        }
        match child.kind() {
            _ if child.is_error() => match child.child(0) {
                Some(child) if child.kind().ends_with("_tag_open") => {
                    let kind = child.kind();
                    if kind == "html_tag_open" {
                        break;
                    }
                    let tag = &kind[..kind.len() - "_open".len()];
                    log::trace!(
                        "cursor {} is in ERROR which appears to be a {}",
                        cursor,
                        tag
                    );
                    return grammar::Tag::from_str(&tag).and_then(|tag| {
                        search_completions_in_tag(
                            tag.properties(),
                            child,
                            &text,
                            cursor,
                            completions,
                        )
                    });
                },
                _ => {
                    log::trace!("cursor {} is in ERROR without children", cursor);
                    break;
                },
            }
            ">" => {
                if child.is_missing() {
                    log::trace!("\">\" is missing in {}", node.kind());
                    continue;
                }
                if *cursor > child.start_position() {
                    completion_type = CompletionType::Tags;
                }
                position = TagParsePosition::Children;
            }
            "self_closing_tag_end" => {
                if child.is_missing() {
                    log::trace!("\"/>\" is missing in {}", node.kind());
                    continue;
                }
                log::trace!("reached end of {}", tag.name);
                break;
            }
            kind if kind.ends_with("_tag_close") => {
                log::trace!("reached end of {}", tag.name);
                break;
            }
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
                    .expect(
                        format!(
                            "attribute {:?} of {:?} did not have a attribute-value child",
                            child, node
                        )
                        .as_str(),
                    )
                    .utf8_text(text.as_bytes())
                    .expect(
                        format!(
                            "attribute-value in {:?} of {:?} did not contain text",
                            child, node
                        )
                        .as_str(),
                    )
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
                grammar::TagAttributes::These(possible) => possible
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
                grammar::TagAttributes::None => {}
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
