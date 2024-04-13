use super::{LsError, ResponseErrorCode};
use crate::{
    document_store::{self, Document},
    grammar, modules, parser,
};
use anyhow::Result;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionTextEdit, Documentation,
    MarkupContent, MarkupKind, Position, Range, TextDocumentPositionParams, TextEdit, Url,
};
use std::{collections::HashMap, str::FromStr};
use tree_sitter::{Node, Point};

#[derive(Debug, PartialEq)]
enum TagParsePosition {
    Attributes,
    Children,
}

#[derive(Debug, PartialEq)]
enum CompletionType {
    Attributes,
    Attribute(String),
    Tags,
}

#[derive(Debug)]
struct CompletionCollector<'a> {
    cursor: Point,
    file: &'a Url,
    document: &'a Document,
    completions: Vec<CompletionItem>,
}

impl CompletionCollector<'_> {
    fn new<'a>(
        params: &'a TextDocumentPositionParams,
        document: &'a Document,
    ) -> CompletionCollector<'a> {
        return CompletionCollector {
            cursor: Point::new(
                params.position.line as usize,
                params.position.character as usize,
            ),
            file: &params.text_document.uri,
            document,
            completions: Vec::new(),
        };
    }

    fn search_completions_in_document(self: &mut Self, root: Node) -> Result<()> {
        for node in root.children(&mut root.walk()) {
            // tree sitter puts an 'missing' node at the end of unclosed tags, so we cannot blindly
            // skip all nodes that end before the cursor
            if node.end_position() < self.cursor
                && (node.child_count() == 0
                    || !node
                        .child(node.child_count() - 1)
                        .is_some_and(|close_bracket| close_bracket.is_missing()))
            {
                continue;
            }
            if node.start_position() > self.cursor {
                break;
            }
            // we are in a nested node:
            match node.kind() {
                // ignore for now
                "page_header" | "import_header" | "taglib_header" | "text" | "comment" => {
                    return Ok(())
                }
                _ if node.is_error() => match node.child(0) {
                    Some(child) if child.kind().ends_with("_tag_open") => {
                        let kind = child.kind();
                        if kind == "html_tag_open" {
                            break;
                        }
                        let tag = &kind[..kind.len() - "_open".len()];
                        log::trace!(
                            "cursor {} is in ERROR which appears to be a {}",
                            self.cursor,
                            tag
                        );
                        return grammar::Tag::from_str(&tag).and_then(|tag| {
                            self.search_completions_in_tag(tag.properties(), node)
                        });
                    }
                    _ if node
                        .utf8_text(&self.document.text.as_bytes())
                        .map(|text| self.cut_text_up_to_cursor(node, text))
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
                                .and_then(|tag| tag.utf8_text(self.document.text.as_bytes()).ok())
                                .map(|tag| tag[1..].to_string() + ">")
                            {
                                Some(tag) => self.completions.push(CompletionItem {
                                    label: "</".to_string() + &tag,
                                    kind: Some(CompletionItemKind::SNIPPET),
                                    insert_text: Some(tag),
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
                    return self.search_completions_in_document(node);
                }
                kind => {
                    log::trace!(
                        "cursor {} is in tag {} ({} - {})",
                        self.cursor,
                        kind,
                        node.start_byte(),
                        node.end_position()
                    );
                    return grammar::Tag::from_str(kind)
                        .and_then(|tag| self.search_completions_in_tag(tag.properties(), node));
                }
            }
        }
        return self.complete_top_level_tags();
    }

    fn complete_top_level_tags(self: &mut Self) -> Result<()> {
        let line = self
            .document
            .text
            .lines()
            .nth(self.cursor.row)
            .map(|l| l.split_at(self.cursor.column).0)
            .unwrap_or("");
        let mut start = self.cursor.column as u32;
        for (i, c) in line.chars().rev().enumerate() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | ':' | '_' | '-' => continue,
                '<' => {
                    start -= i as u32 + 1;
                    break;
                }
                _ => break,
            }
        }
        let start = Position {
            line: self.cursor.row as u32,
            character: start,
        };
        let end = Position {
            line: self.cursor.row as u32,
            character: self.cursor.column as u32,
        };
        log::info!(
            "{:?} - {:?} (\"{}\")",
            start,
            end,
            line.split_at(end.character as usize)
                .0
                .split_at(start.character as usize)
                .1
        );
        grammar::TOP_LEVEL_TAGS
            .iter()
            .map(|tag| tag.properties())
            .map(|properties| CompletionItem {
                label: format!("<{}", properties.name),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: properties.detail.map(|detail| detail.to_string()),
                documentation: properties.documentation.map(|detail| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: detail.to_string(),
                    })
                }),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    new_text: format!("<{}", properties.name),
                    range: Range { start, end },
                })),
                // insert_text: Some(format!("<{}", properties.name)),
                ..Default::default()
            })
            .for_each(|completion| self.completions.push(completion));
        return Ok(());
    }

    fn search_completions_in_tag(
        self: &mut Self,
        tag: grammar::TagProperties,
        node: Node,
    ) -> Result<()> {
        let mut attributes: HashMap<String, String> = HashMap::new();
        let mut completion_type = CompletionType::Attributes;
        let mut position = TagParsePosition::Attributes;
        for child in node.children(&mut node.walk()) {
            if position == TagParsePosition::Children {
                // tree sitter puts an 'missing' node at the end of unclosed tags, so we cannot blindly
                // skip all nodes that end before the cursor
                if child.end_position() < self.cursor
                    && (child.child_count() == 0
                        || !child
                            .child(child.child_count() - 1)
                            .is_some_and(|close_bracket| close_bracket.is_missing()))
                {
                    continue;
                }
                if child.start_position() > self.cursor {
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
                            self.cursor,
                            tag
                        );
                        return grammar::Tag::from_str(&tag).and_then(|tag| {
                            self.search_completions_in_tag(tag.properties(), child)
                        });
                    }
                    _ if child
                        .utf8_text(self.document.text.as_bytes())
                        .map(|text| self.cut_text_up_to_cursor(child, text))
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
                                    tag = current
                                        .utf8_text(self.document.text.as_bytes())
                                        .ok()
                                        .map(|tag| &tag[1..])
                                }
                                "html_option_tag" => {
                                    tag = current
                                        .child(0)
                                        .and_then(|tag| {
                                            tag.utf8_text(self.document.text.as_bytes()).ok()
                                        })
                                        .map(|tag| &tag[1..])
                                }
                                kind if kind.ends_with("_tag_open") => {
                                    tag =
                                        grammar::Tag::from_str(&kind[..kind.len() - "_open".len()])
                                            .ok()
                                            .map(|tag| tag.properties().name)
                                }
                                _ => continue,
                            }
                            match tag.map(|tag| tag.to_string() + ">") {
                                Some(tag) => self.completions.push(CompletionItem {
                                    label: "</".to_string() + &tag,
                                    kind: Some(CompletionItemKind::SNIPPET),
                                    insert_text: Some(tag),
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
                    if self.cursor > child.start_position() {
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
                    return self.search_completions_in_tag(tag, child);
                }
                kind if kind.ends_with("_attribute") => {
                    let attribute = kind[..kind.len() - "_attribute".len()].to_string();
                    if child.start_position() < self.cursor && child.end_position() > self.cursor {
                        completion_type = CompletionType::Attribute(attribute.clone());
                    }
                    attributes.insert(
                        attribute,
                        parser::attribute_value_of(child, &self.document.text).to_string(),
                    );
                }
                kind if kind.ends_with("_tag") => {
                    return self.search_completions_in_tag(
                        grammar::Tag::from_str(kind)?.properties(),
                        child,
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
                return self.complete_attributes_of(tag, attributes);
            }
            CompletionType::Attribute(name) => {
                log::info!("complete values for attribute {} of {}", name, tag.name);
                return self.complete_values_of_attribute(tag, name, attributes);
            }
            CompletionType::Tags => {
                log::info!("complete child tags for {}", tag.name);
                // TODO: maybe also propose closing tag if not already present.
                return self.complete_children_of(tag);
            }
        };
    }

    fn cut_text_up_to_cursor<'a>(self: &Self, node: Node, text: &'a str) -> &'a str {
        let start = node.start_position();
        return match self.cursor.row.cmp(&start.row) {
            std::cmp::Ordering::Equal if self.cursor.column >= start.column => {
                &text[0..self.cursor.column - start.column]
            }
            std::cmp::Ordering::Less | std::cmp::Ordering::Equal => "",
            std::cmp::Ordering::Greater => {
                let expected_new_lines = self.cursor.row - start.row;
                let mut position = 0;
                for (index, line) in text.splitn(expected_new_lines + 1, '\n').enumerate() {
                    match index {
                        0 => position = line.len(),
                        n if n == expected_new_lines => {
                            return &text[0..position + 1 + self.cursor.column]
                        }
                        _ => position += 1 + line.len(),
                    }
                }
                return text;
            }
        };
    }

    fn complete_values_of_attribute(
        self: &mut Self,
        tag: grammar::TagProperties,
        attribute: String,
        attributes: HashMap<String, String>,
    ) -> Result<()> {
        for rule in tag.attribute_rules {
            match rule {
                grammar::AttributeRule::UriExists(uri, module) if *uri == attribute => {
                    let module = match attributes.get(*module).map(|str| str.as_str()) {
                        Some("${module.id}") | None => self
                            .file
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
                                self.completions.push(CompletionItem {
                                    label: name.clone() + "/",
                                    kind: Some(CompletionItemKind::FOLDER),
                                    insert_text: Some(name + "/"),
                                    ..Default::default()
                                })
                            } else if name.ends_with(".spml") {
                                self.completions.push(CompletionItem {
                                    label: name.clone(),
                                    kind: Some(CompletionItemKind::FILE),
                                    insert_text: Some(name),
                                    ..Default::default()
                                })
                            }
                        }
                    }
                    break;
                }
                grammar::AttributeRule::ValueOneOf(name, values)
                | grammar::AttributeRule::ValueOneOfCaseInsensitive(name, values)
                    if *name == attribute =>
                {
                    values.iter().for_each(|value| {
                        self.completions.push(CompletionItem {
                            label: value.to_string(),
                            kind: Some(CompletionItemKind::ENUM_MEMBER),
                            detail: None,
                            documentation: None,
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
        self: &mut Self,
        tag: grammar::TagProperties,
        attributes: HashMap<String, String>,
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
                    ..Default::default()
                })
                .for_each(|completion| self.completions.push(completion)),
            grammar::TagAttributes::None => {}
        };
        return Ok(());
    }

    fn complete_children_of(self: &mut Self, tag: grammar::TagProperties) -> Result<()> {
        match tag.children {
            grammar::TagChildren::Any => self.complete_top_level_tags()?,
            grammar::TagChildren::None => {}
            grammar::TagChildren::Scalar(tag) => self.completions.push(CompletionItem {
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
                    ..Default::default()
                })
                .for_each(|completion| self.completions.push(completion)),
        };
        return Ok(());
    }
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
    let root = document.tree.root_node();
    let mut completion_collector = CompletionCollector::new(&text_params, &document);
    completion_collector
        .search_completions_in_document(root)
        .map_err(|err| LsError {
            message: format!("failed to validate document: {}", err),
            code: ResponseErrorCode::RequestFailed,
        })?;
    return Ok(completion_collector.completions);
}
