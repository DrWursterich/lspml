use std::{cmp::Ordering, fs, iter::Iterator};

use anyhow::Result;
use lsp_server::ErrorCode;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionTextEdit, Documentation,
    MarkupContent, MarkupKind, Position, Range, TextDocumentPositionParams, TextEdit, Url,
};

use crate::{
    document_store::{self, Document},
    grammar::{self, TagAttribute, TagAttributeType, TagAttributes, TagChildren, TagDefinition},
    modules::{self, Module},
    parser::{DocumentNode, Node, ParsableTag, SpelAttribute, Tag, TagBody},
    spel::ast::{SpelAst, SpelResult, StringLiteral, Uri, Word, WordFragment},
};

use super::LsError;

#[derive(Debug)]
struct CompletionCollector<'a> {
    cursor: Position,
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
            cursor: params.position,
            file: &params.text_document.uri,
            document,
            completions: Vec::new(),
        };
    }

    fn search_completions_in_document(&mut self) {
        let mut nodes = &self.document.tree.nodes;
        let mut current = None;
        loop {
            if let Some(node) = find_tag_at(nodes, self.cursor) {
                current = Some(node);
                if let Node::Tag(tag) = node {
                    if let Some(body) = tag.body() {
                        nodes = &body.nodes;
                        continue;
                    }
                }
            }
            break;
        }
        match current {
            Some(Node::Tag(tag)) => self.search_completions_in_tag(tag),
            _ => self.complete_top_level_tags(),
        }
    }

    fn complete_top_level_tags(&mut self) {
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

    // fn complete_closing_tag(&mut self, mut current: Node) {
    //     loop {
    //         match current.prev_sibling().or_else(|| current.parent()) {
    //             Some(next) => current = next,
    //             None => return,
    //         };
    //         let tag = match current.kind() {
    //             "html_tag_open" => current
    //                 .utf8_text(self.document.text.as_bytes())
    //                 .ok()
    //                 .map(|tag| &tag[1..]),
    //             "html_option_tag" => current
    //                 .child(0)
    //                 .and_then(|tag| tag.utf8_text(self.document.text.as_bytes()).ok())
    //                 .map(|tag| &tag[1..]),
    //             kind if kind.ends_with("_tag_open") => {
    //                 TagDefinition::from_str(&kind[..kind.len() - "_open".len()])
    //                     .ok()
    //                     .map(|tag| tag.name)
    //             }
    //             _ => continue,
    //         };
    //         if let Some(tag) = tag.map(|tag| tag.to_string() + ">") {
    //             self.completions.push(CompletionItem {
    //                 label: "</".to_string() + &tag,
    //                 kind: Some(CompletionItemKind::SNIPPET),
    //                 insert_text: Some(tag),
    //                 ..Default::default()
    //             });
    //         };
    //         return;
    //     }
    // }

    fn determine_tag_range(&self) -> Range {
        let line = self
            .document
            .text
            .lines()
            .nth(self.cursor.line as usize)
            .map(|l| l.split_at(self.cursor.character as usize).0)
            .unwrap_or("");
        let mut start = self.cursor.character;
        for (i, c) in line.chars().rev().enumerate() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | ':' | '_' | '-' => continue,
                '<' => start -= (i as u32) + 1,
                _ => (),
            }
            break;
        }
        return Range {
            start: Position {
                line: self.cursor.line as u32,
                character: start,
            },
            end: Position {
                line: self.cursor.line as u32,
                character: self.cursor.character as u32,
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

    fn search_completions_in_tag(&mut self, tag: &Tag) {
        if let Some(body) = tag.body() {
            if self.is_cursor_in_body(body) {
                return self.complete_children_of(tag);
            }
        }
        for (name, attribute) in &tag.spel_attributes() {
            if is_in_attribute_value(attribute, &self.cursor) {
                let definition = tag.definition();
                let attribute_type = name
                    .strip_suffix("_attribute")
                    .and_then(|name| definition.attributes.get_by_name(name));
                return match &attribute.spel {
                    // SpelAst::Comparable(_) => (),
                    // SpelAst::Condition(_) => (),
                    // SpelAst::Expression(_) => (),
                    // SpelAst::Identifier(_) => (),
                    // SpelAst::Object(_) => (),
                    // SpelAst::Query(_) => (),
                    // SpelAst::Regex(_) => (),
                    SpelAst::String(_)
                        if attribute_type
                            .is_some_and(|a| matches!(&a.r#type, TagAttributeType::Module)) =>
                    {
                        modules::all_modules().iter().for_each(|(name, _)| {
                            self.completions.push(CompletionItem {
                                label: name.to_owned(),
                                kind: Some(CompletionItemKind::MODULE),
                                insert_text: Some(name.to_owned()),
                                ..Default::default()
                            })
                        });
                    }
                    SpelAst::String(_) => {
                        for rule in tag.definition().attribute_rules {
                            let name = name.strip_suffix("_attribute").unwrap_or(name);
                            match rule {
                                grammar::AttributeRule::ValueOneOf(attribute_name, values)
                                | grammar::AttributeRule::ValueOneOfCaseInsensitive(
                                    attribute_name,
                                    values,
                                ) if attribute_name == &name => {
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
                                _ => (),
                            };
                        }
                    }
                    SpelAst::Uri(SpelResult::Valid(uri)) => {
                        let mut module = None;
                        if let Some(TagAttribute {
                            r#type: TagAttributeType::Uri { module_attribute },
                            ..
                        }) = attribute_type
                        {
                            if let Some(SpelAttribute {
                                spel: SpelAst::String(SpelResult::Valid(Word { fragments })),
                                ..
                            }) = tag.spel_attribute(&module_attribute)
                            {
                                if fragments.len() == 1 {
                                    if let WordFragment::String(StringLiteral { content, .. }) =
                                        &fragments[0]
                                    {
                                        module = modules::find_module_by_name(content);
                                    }
                                }
                            }
                        }
                        let module = module.or_else(|| {
                            self.file
                                .to_file_path()
                                .ok()
                                .and_then(|file| modules::find_module_for_file(file.as_path()))
                        });
                        if let Some(module) = &module {
                            if let Err(err) = self.complete_uri(uri, module) {
                                log::error!("failed to complete uri: {}", err);
                            }
                        }
                    }
                    _ => (),
                };
            }
        }
        return self.complete_attributes_of(tag);
    }

    fn is_cursor_in_body(&self, body: &TagBody) -> bool {
        if (self.cursor.line as usize) < body.open_location.line {
            return false;
        }
        if (self.cursor.line as usize) == body.open_location.line {
            if (self.cursor.character as usize) <= body.open_location.char {
                return false;
            }
        }
        return true;
    }

    fn complete_uri(&mut self, uri: &Uri, module: &Module) -> Result<()> {
        let path = uri.to_string();
        for entry in fs::read_dir(module.path.clone() + path.as_str())? {
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
        return Ok(());
    }

    fn complete_attributes_of(&mut self, tag: &Tag) {
        if let TagAttributes::These(possible) = tag.definition().attributes {
            possible
                .iter()
                .filter(|attribute| tag.spel_attribute(attribute.name).is_none())
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
                .for_each(|completion| self.completions.push(completion));
        };
    }

    fn complete_children_of(&mut self, tag: &Tag) {
        match tag.definition().children {
            TagChildren::Any => self.complete_top_level_tags(),
            TagChildren::None => (),
            TagChildren::Scalar(tag) => self.complete_tag(tag),
            TagChildren::Vector(tags) => self.complete_tags(tags.iter()),
        };
    }
}

pub(crate) fn find_tag_at(nodes: &Vec<Node>, cursor: Position) -> Option<&Node> {
    for node in nodes {
        let range = node.range();
        if cursor > range.end {
            continue;
        }
        if cursor < range.start {
            break;
        }
        return Some(node);
    }
    return None;
}

fn is_in_attribute_value(attribute: &SpelAttribute, position: &Position) -> bool {
    let opening_line = attribute
        .opening_quote_location
        .line
        .cmp(&(position.line as usize));
    let opening_char = attribute
        .opening_quote_location
        .char
        .cmp(&(position.character as usize));
    match (opening_line, opening_char) {
        (Ordering::Less, _) | (Ordering::Equal, Ordering::Less) => (),
        _ => return false,
    }
    let closing_line = attribute
        .closing_quote_location
        .line
        .cmp(&(position.line as usize));
    let closing_char = attribute
        .closing_quote_location
        .char
        .cmp(&(position.character as usize));
    return match (closing_line, closing_char) {
        (Ordering::Greater, _)
        | (Ordering::Equal, Ordering::Greater)
        | (Ordering::Equal, Ordering::Equal) => true,
        _ => false,
    };
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
    let mut completion_collector = CompletionCollector::new(&text_params, &document);
    completion_collector.search_completions_in_document();
    return Ok(completion_collector.completions);
}

// #[cfg(test)]
// mod tests {
//     use lsp_types::{
//         CompletionParams, PartialResultParams, Position, TextDocumentIdentifier,
//         TextDocumentPositionParams, Url, WorkDoneProgressParams,
//     };

//     use crate::document_store::Document;

//     use super::CompletionCollector;

//     #[test]
//     fn test_completion_for_attributes_in_nested_tag() {
//         let document_content = concat!(
//             "<%@ page language=\"java\" pageEncoding=\"UTF-8\" contentType=\"text/html; charset=UTF-8\"%>\n",
//             "<sp:include module=\"test-module\" uri=\"/functions/doSomething.spml\">\n",
//             "\t<sp:argument \n",
//             "</sp:include>\n");
//         let params: CompletionParams = CompletionParams {
//             text_document_position: TextDocumentPositionParams {
//                 text_document: TextDocumentIdentifier {
//                     uri: Url::parse("file:///some/test/file.spml").unwrap(),
//                 },
//                 position: Position {
//                     line: 2,
//                     character: 14,
//                 },
//             },
//             work_done_progress_params: WorkDoneProgressParams {
//                 work_done_token: None,
//             },
//             partial_result_params: PartialResultParams {
//                 partial_result_token: None,
//             },
//             context: None,
//         };

//         let document = Document::new(document_content.to_string()).unwrap();
//         let mut completion_collector =
//             CompletionCollector::new(&params.text_document_position, &document);
//         completion_collector.search_completions_in_document();
//         let result = completion_collector.completions;

//         assert_eq!(
//             result
//                 .iter()
//                 .map(|c| c.label.clone())
//                 .collect::<Vec<String>>(),
//             vec![
//                 "condition",
//                 "default",
//                 "expression",
//                 "locale",
//                 "name",
//                 "object",
//                 "value",
//             ]
//         );
//     }
// }
