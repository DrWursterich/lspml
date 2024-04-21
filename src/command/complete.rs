use super::LsError;
use crate::{
    document_store::{self, Document},
    grammar::{self, TagChildren, TagDefinition},
    modules, parser,
};
use anyhow::Result;
use lsp_server::ErrorCode;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionTextEdit, Documentation,
    MarkupContent, MarkupKind, Position, Range, TextDocumentPositionParams, TextEdit, Url,
};
use std::{cmp::Ordering, collections::HashMap, iter::Iterator, str::FromStr};
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
            match self.compare_node_to_cursor(node) {
                Ordering::Less => continue,
                Ordering::Equal => (),
                Ordering::Greater => break,
            };
            match node.kind() {
                "page_header" | "import_header" | "taglib_header" | "text" | "comment" => {
                    return Ok(()) // ignore for now
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
                        return TagDefinition::from_str(&tag)
                            .and_then(|tag| self.search_completions_in_tag(tag, node));
                    }
                    _ if node
                        .utf8_text(&self.document.text.as_bytes())
                        .map(|text| self.cut_text_up_to_cursor(node, text))
                        .is_ok_and(|text| text == "/" || text.ends_with("</")) =>
                    {
                        return Ok(self.complete_closing_tag(node));
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
                    return TagDefinition::from_str(kind)
                        .and_then(|tag| self.search_completions_in_tag(tag, node));
                }
            }
        }
        return Ok(self.complete_top_level_tags());
    }

    fn complete_top_level_tags(self: &mut Self) {
        self.complete_tags(grammar::TOP_LEVEL_TAGS.iter());
    }

    fn complete_tags<'a>(&mut self, tags: impl Iterator<Item = &'a TagDefinition>) {
        let range = self.determine_tag_range();
        tags.map(|tag| Self::tag_to_completion(tag, range))
            .for_each(|completion| self.completions.push(completion));
    }

    fn complete_tag<'a>(&mut self, tag: &TagDefinition) {
        self.completions
            .push(Self::tag_to_completion(tag, self.determine_tag_range()));
    }

    fn complete_closing_tag(&mut self, mut current: Node) {
        loop {
            match current.prev_sibling().or_else(|| current.parent()) {
                Some(next) => current = next,
                None => return,
            };
            let tag = match current.kind() {
                "html_tag_open" => current
                    .utf8_text(self.document.text.as_bytes())
                    .ok()
                    .map(|tag| &tag[1..]),
                "html_option_tag" => current
                    .child(0)
                    .and_then(|tag| tag.utf8_text(self.document.text.as_bytes()).ok())
                    .map(|tag| &tag[1..]),
                kind if kind.ends_with("_tag_open") => {
                    TagDefinition::from_str(&kind[..kind.len() - "_open".len()])
                        .ok()
                        .map(|tag| tag.name)
                }
                _ => continue,
            };
            if let Some(tag) = tag.map(|tag| tag.to_string() + ">") {
                self.completions.push(CompletionItem {
                    label: "</".to_string() + &tag,
                    kind: Some(CompletionItemKind::SNIPPET),
                    insert_text: Some(tag),
                    ..Default::default()
                });
            };
            return;
        }
    }

    fn determine_tag_range(&self) -> Range {
        let line = self
            .document
            .text
            .lines()
            .nth(self.cursor.row)
            .map(|l| l.split_at(self.cursor.column).0)
            .unwrap_or("");
        let mut start = self.cursor.column;
        for (i, c) in line.chars().rev().enumerate() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | ':' | '_' | '-' => continue,
                '<' => start -= i + 1,
                _ => (),
            }
            break;
        }
        return Range {
            start: Position {
                line: self.cursor.row as u32,
                character: start as u32,
            },
            end: Position {
                line: self.cursor.row as u32,
                character: self.cursor.column as u32,
            },
        };
    }

    fn tag_to_completion(tag: &TagDefinition, range: Range) -> CompletionItem {
        let new_text = format!("<{}", tag.name);
        return CompletionItem {
            label: new_text.clone(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: tag.detail.map(|detail| detail.to_string()),
            documentation: tag.documentation.map(|detail| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: detail.to_string(),
                })
            }),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit { new_text, range })),
            ..Default::default()
        };
    }

    fn search_completions_in_tag(self: &mut Self, tag: TagDefinition, node: Node) -> Result<()> {
        let mut attributes: HashMap<String, String> = HashMap::new();
        let mut completion_type = CompletionType::Attributes;
        let mut position = TagParsePosition::Attributes;
        for child in node.children(&mut node.walk()) {
            if position == TagParsePosition::Children {
                match self.compare_node_to_cursor(child) {
                    Ordering::Less => continue,
                    Ordering::Equal => completion_type = CompletionType::Tags,
                    Ordering::Greater => break,
                };
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
                        return TagDefinition::from_str(&tag)
                            .and_then(|tag| self.search_completions_in_tag(tag, child));
                    }
                    _ if child
                        .utf8_text(self.document.text.as_bytes())
                        .map(|text| self.cut_text_up_to_cursor(child, text))
                        .is_ok_and(|text| text == "/" || text.ends_with("</")) =>
                    {
                        return Ok(self.complete_closing_tag(child));
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
                    log::info!("search {} children in {}", tag.name, node.kind());
                    return self.search_completions_in_tag(tag, child);
                }
                kind if kind.ends_with("_tag_close") => {
                    break;
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
                    return self.search_completions_in_tag(TagDefinition::from_str(kind)?, child);
                }
                kind => {
                    log::info!("ignore node {}", kind);
                }
            }
        }
        return Ok(match completion_type {
            CompletionType::Attributes => {
                log::info!("complete attributes for {}", tag.name);
                self.complete_attributes_of(tag, attributes)
            }
            CompletionType::Attribute(name) => {
                log::info!("complete values for attribute {} of {}", name, tag.name);
                self.complete_values_of_attribute(tag, name, attributes)?
            }
            CompletionType::Tags => {
                log::info!("complete child tags for {}", tag.name);
                // TODO: maybe also propose closing tag if not already present.
                self.complete_children_of(tag)
            }
        });
    }

    fn compare_node_to_cursor(&self, node: Node) -> Ordering {
        // tree sitter puts a 'missing' node at the end of unclosed tags, so we cannot blindly skip
        // all nodes that end before the cursor
        if node.end_position() < self.cursor
            && (node.child_count() == 0
                || !node
                    .child(node.child_count() - 1)
                    .is_some_and(|close_bracket| close_bracket.is_missing()))
        {
            // continue;
            return Ordering::Less;
        }
        if node.start_position() > self.cursor {
            // break;
            return Ordering::Greater;
        }
        return Ordering::Equal;
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
        tag: TagDefinition,
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
        tag: TagDefinition,
        attributes: HashMap<String, String>,
    ) {
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
    }

    fn complete_children_of(self: &mut Self, tag: TagDefinition) {
        match tag.children {
            TagChildren::Any => self.complete_top_level_tags(),
            TagChildren::None => (),
            TagChildren::Scalar(tag) => self.complete_tag(tag),
            TagChildren::Vector(tags) => self.complete_tags(tags.iter()),
        };
    }
}

pub(crate) fn complete(params: CompletionParams) -> Result<Vec<CompletionItem>, LsError> {
    let text_params = params.text_document_position;
    let uri = &text_params.text_document.uri;
    let document = match document_store::get(uri) {
        Some(document) => Ok(document),
        None => document_store::Document::from_uri(uri)
            .map(|document| document_store::put(uri, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", uri, err);
                return LsError {
                    message: format!("cannot read file {}", uri),
                    code: ErrorCode::RequestFailed,
                };
            }),
    }?;
    let root = document.tree.root_node();
    let mut completion_collector = CompletionCollector::new(&text_params, &document);
    completion_collector
        .search_completions_in_document(root)
        .map_err(|err| LsError {
            message: format!("failed to validate document: {}", err),
            code: ErrorCode::RequestFailed,
        })?;
    return Ok(completion_collector.completions);
}
