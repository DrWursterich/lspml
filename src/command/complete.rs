use super::{LsError, ResponseErrorCode};
use crate::document_store;
use crate::grammar;
use crate::modules;
use crate::parser;
use anyhow::Result;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, Documentation, InsertTextMode,
    MarkupContent, MarkupKind, Url,
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
    Attribute(String),
    Tags,
}

pub(crate) fn complete(params: CompletionParams) -> Result<Vec<CompletionItem>, LsError> {
    let text_params = params.text_document_position;
    let uri = &text_params.text_document.uri;
    let document = match document_store::get(uri) {
        Some(document) => Ok(document),
        None => document_store::Document::new(uri)
            .map(|document| document_store::put(uri, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", uri, err);
                return LsError {
                    message: format!("cannot read file {}", uri),
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
        uri,
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
    file: &Url,
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
            continue;
        }
        if node.start_position() > *cursor {
            break;
        }
        // we are in a nested node:
        match node.kind() {
            // ignore for now
            "page_header" | "import_header" | "taglib_header" | "text" | "comment" => return Ok(()),
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
                            file,
                            completions,
                        )
                    });
                }
                _ if node
                    .utf8_text(&text.as_bytes())
                    .map(|text| cut_text_up_to_cursor(node, text, *cursor))
                    .is_ok_and(|text| text == "/" || text.ends_with("</")) =>
                {
                    let mut current = node;
                    loop {
                        match current.prev_sibling().or_else(|| current.parent()) {
                            Some(next) => current = next,
                            None => return Ok(()),
                        };
                        let tag;
                        match current.kind() {
                            "html_tag_open" => tag = Some(current),
                            "html_option_tag" => tag = current.child(0),
                            _ => continue,
                        }
                        match tag
                            .and_then(|tag| tag.utf8_text(text.as_bytes()).ok())
                            .map(|tag| tag[1..].to_string() + ">")
                        {
                            Some(tag) => completions.push(CompletionItem {
                                label: "</".to_string() + &tag,
                                kind: Some(CompletionItemKind::SNIPPET),
                                detail: None,
                                documentation: None,
                                insert_text: Some(tag),
                                insert_text_mode: Some(InsertTextMode::AS_IS),
                                ..Default::default()
                            }),
                            None => {}
                        };
                        break;
                    }
                    return Ok(());
                }
                _ => {}
            },
            // is there a way to "include" other lsps?
            "java_tag" | "script_tag" | "style_tag" | "html_void_tag" => {
                log::info!(
                    "cursor seems to be inside {}, for which completion is not supported",
                    node.kind()
                );
            }
            "html_tag" | "html_option_tag" => {
                return search_completions_in_document(node, text, cursor, file, completions);
            }
            kind => {
                log::trace!(
                    "cursor {} is in tag {} ({} - {})",
                    cursor,
                    kind,
                    node.start_byte(),
                    node.end_position()
                );
                return grammar::Tag::from_str(kind).and_then(|tag| {
                    search_completions_in_tag(
                        tag.properties(),
                        node,
                        &text,
                        cursor,
                        file,
                        completions,
                    )
                });
            }
        }
    }
    return complete_top_level_tags(completions);
}

fn complete_top_level_tags(completions: &mut Vec<CompletionItem>) -> Result<()> {
    grammar::TOP_LEVEL_TAGS
        .iter()
        .map(|tag| tag.properties())
        .map(|properties| CompletionItem {
            label: "<".to_string() + properties.name,
            kind: Some(CompletionItemKind::METHOD),
            detail: properties.detail.map(|detail| detail.to_string()),
            documentation: properties.documentation.map(|detail| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: detail.to_string(),
                })
            }),
            insert_text: Some("<".to_string() + properties.name),
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
    file: &Url,
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
            if child.start_position() > *cursor {
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
                            file,
                            completions,
                        )
                    });
                }
                _ if child
                    .utf8_text(&text.as_bytes())
                    .map(|text| cut_text_up_to_cursor(child, text, *cursor))
                    .is_ok_and(|text| text == "/" || text.ends_with("</")) =>
                {
                    let mut current = child;
                    loop {
                        match current.prev_sibling().or_else(|| current.parent()) {
                            Some(next) => current = next,
                            None => return Ok(()),
                        };
                        let tag;
                        match current.kind() {
                            "html_tag_open" => {
                                tag = current.utf8_text(text.as_bytes()).ok().map(|tag| &tag[1..])
                            }
                            "html_option_tag" => {
                                tag = current
                                    .child(0)
                                    .and_then(|tag| tag.utf8_text(text.as_bytes()).ok())
                                    .map(|tag| &tag[1..])
                            }
                            kind if kind.ends_with("_tag_open") => {
                                tag = grammar::Tag::from_str(&kind[..kind.len() - "_open".len()])
                                    .ok()
                                    .map(|tag| tag.properties().name)
                            }
                            _ => continue,
                        }
                        match tag.map(|tag| tag.to_string() + ">") {
                            Some(tag) => completions.push(CompletionItem {
                                label: "</".to_string() + &tag,
                                kind: Some(CompletionItemKind::SNIPPET),
                                detail: None,
                                documentation: None,
                                insert_text: Some(tag),
                                insert_text_mode: Some(InsertTextMode::AS_IS),
                                ..Default::default()
                            }),
                            None => {}
                        };
                        break;
                    }
                    return Ok(());
                }
                _ => {}
            },
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
                break;
            }
            kind if kind.ends_with("_tag_close") => {
                break;
            }
            // is there a way to "include" other lsps?
            "java_tag" | "script_tag" | "style_tag" | "html_void_tag" => {
                log::info!(
                    "cursor seems to be inside {}, for which completion is not supported",
                    node.kind()
                );
            }
            "html_tag" | "html_option_tag" => {
                if child.child_count() == 0 {
                    log::info!(
                        "cursor seems to be inside {}, for which completion is not supported",
                        node.kind()
                    );
                    continue;
                }
                // search in the child tag and complete the children possible in the current tag
                log::info!("search {} children in {}", tag.name, node.kind());
                return search_completions_in_tag(tag, child, text, cursor, file, completions);
            }
            kind if kind.ends_with("_attribute") => {
                let attribute = kind[..kind.len() - "_attribute".len()].to_string();
                if &child.start_position() < cursor && &child.end_position() > cursor {
                    completion_type = CompletionType::Attribute(attribute.clone());
                }
                attributes.insert(
                    attribute,
                    parser::attribute_value_of(child, text).to_string(),
                );
            }
            kind if kind.ends_with("_tag") => {
                return search_completions_in_tag(
                    grammar::Tag::from_str(kind)?.properties(),
                    child,
                    text,
                    cursor,
                    file,
                    completions,
                );
            }
            kind => {
                log::info!("ignore node {}", kind);
            }
        }
    }
    match completion_type {
        CompletionType::Attributes => {
            log::info!("complete attributes for {}", tag.name);
            return complete_attributes_of(tag, attributes, completions);
        }
        CompletionType::Attribute(name) => {
            log::info!("complete values for attribute {} of {}", name, tag.name);
            return complete_values_of_attribute(tag, name, attributes, file, completions);
        }
        CompletionType::Tags => {
            log::info!("complete child tags for {}", tag.name);
            // TODO: maybe also propose cosing tag if not already present.
            return complete_children_of(tag, completions);
        }
    };
}

fn cut_text_up_to_cursor<'a>(node: Node, text: &'a str, cursor: Point) -> &'a str {
    if cursor.row <= node.start_position().row {
        return &text[0..cursor.column - node.start_position().column];
    }
    let expected_new_lines = cursor.row - node.start_position().row;
    let mut position = 0;
    for (index, line) in text.splitn(expected_new_lines + 1, '\n').enumerate() {
        match index {
            0 => position = line.len(),
            n if n == expected_new_lines => return &text[0..position + 1 + cursor.column],
            _ => position += 1 + line.len(),
        }
    }
    return text;
}

fn complete_values_of_attribute(
    tag: grammar::TagProperties,
    attribute: String,
    attributes: HashMap<String, String>,
    file: &Url,
    completions: &mut Vec<CompletionItem>,
) -> Result<()> {
    for rule in tag.attribute_rules {
        match rule {
            grammar::AttributeRule::UriExists(uri, module) if *uri == attribute => {
                let module = match attributes.get(*module).map(|str| str.as_str()) {
                    Some("${module.id}") | None => file
                        .to_file_path()
                        .ok()
                        .and_then(|file| modules::find_module_for_file(file.as_path())),
                    Some(module) => modules::find_module_by_name(module),
                };
                if let Some(module) = module {
                    let path = attributes
                        .get(&attribute)
                        .and_then(|path| path.rfind("/").map(|index| &path[..index]))
                        .unwrap_or("");
                    for entry in std::fs::read_dir(module.path + path)? {
                        let entry = entry?;
                        let name;
                        if path.len() == 0 {
                            name = "/".to_string() + entry.file_name().to_str().unwrap();
                        } else {
                            name = entry.file_name().to_str().unwrap().to_string();
                        }
                        if entry.path().is_dir() {
                            completions.push(CompletionItem {
                                label: name.clone() + "/",
                                kind: Some(CompletionItemKind::FOLDER),
                                detail: None,
                                documentation: None,
                                insert_text: Some(name + "/"),
                                insert_text_mode: Some(InsertTextMode::AS_IS),
                                ..Default::default()
                            })
                        } else if name.ends_with(".spml") {
                            completions.push(CompletionItem {
                                label: name.clone(),
                                kind: Some(CompletionItemKind::FILE),
                                detail: None,
                                documentation: None,
                                insert_text: Some(name),
                                insert_text_mode: Some(InsertTextMode::AS_IS),
                                ..Default::default()
                            })
                        }
                    }
                }
                break;
            }
            grammar::AttributeRule::ValueOneOf(name, values) if *name == attribute => {
                values.iter().for_each(|value| {
                    completions.push(CompletionItem {
                        label: value.to_string(),
                        kind: Some(CompletionItemKind::ENUM_MEMBER),
                        detail: None,
                        documentation: None,
                        insert_text: Some(value.to_string()),
                        insert_text_mode: Some(InsertTextMode::AS_IS),
                        ..Default::default()
                    })
                });
                break;
            }
            _ => {}
        };
    }
    return Ok(());
}

fn complete_attributes_of(
    tag: grammar::TagProperties,
    attributes: HashMap<String, String>,
    completions: &mut Vec<CompletionItem>,
) -> Result<()> {
    match tag.attributes {
        grammar::TagAttributes::These(possible) => possible
            .iter()
            .filter(|attribute| !attributes.contains_key(attribute.name))
            .map(|attribute| CompletionItem {
                label: attribute.name.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: attribute.detail.map(|detail| detail.to_string()),
                documentation: attribute.documentation.map(|detail| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: detail.to_string(),
                    })
                }),
                insert_text: Some(attribute.name.to_string() + "=\""),
                insert_text_mode: Some(InsertTextMode::AS_IS),
                ..Default::default()
            })
            .for_each(|completion| completions.push(completion)),
        grammar::TagAttributes::None => {}
    };
    return Ok(());
}

fn complete_children_of(
    tag: grammar::TagProperties,
    completions: &mut Vec<CompletionItem>,
) -> Result<()> {
    match tag.children {
        grammar::TagChildren::Any => complete_top_level_tags(completions)?,
        grammar::TagChildren::None => {}
        grammar::TagChildren::Scalar(tag) => completions.push(CompletionItem {
            label: "<".to_string() + tag.properties().name,
            kind: Some(CompletionItemKind::METHOD),
            detail: tag.properties().detail.map(|detail| detail.to_string()),
            documentation: tag.properties().documentation.map(|detail| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: detail.to_string(),
                })
            }),
            insert_text: Some("<".to_string() + tag.properties().name),
            insert_text_mode: Some(InsertTextMode::AS_IS),
            ..Default::default()
        }),
        grammar::TagChildren::Vector(tags) => tags
            .iter()
            .map(|tag| tag.properties())
            .map(|properties| CompletionItem {
                label: "<".to_string() + properties.name,
                kind: Some(CompletionItemKind::METHOD),
                detail: properties.detail.map(|detail| detail.to_string()),
                documentation: properties.documentation.map(|detail| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: detail.to_string(),
                    })
                }),
                insert_text: Some("<".to_string() + properties.name),
                insert_text_mode: Some(InsertTextMode::AS_IS),
                ..Default::default()
            })
            .for_each(|completion| completions.push(completion)),
    };
    return Ok(());
}
