use std::{fs, iter::Iterator, str::FromStr};

use anyhow::Result;
use lsp_server::ErrorCode;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionTextEdit, Documentation,
    MarkupContent, MarkupKind, Position, Range, TextDocumentPositionParams, TextEdit, Uri as Url,
};

use lspml_grammar::{
    self, AttributeRule, TagAttribute, TagAttributeType, TagAttributes, TagChildren, TagDefinition,
};
use lspml_spel::ast::{SpelAst, SpelResult, StringLiteral, Uri, Word, WordFragment};
use lspml_spml::{
    AttributeValue, ErrorNode, Node, ParsableTag, ParsedAttribute, ParsedTag, SpelAttribute,
    SpelAttributeValue, SpmlTag, TagBody, TagError,
};

use super::LsError;

#[derive(Debug)]
struct CompletionCollector<'a> {
    cursor: Position,
    file: &'a Url,
    document: &'a lspml_document_store::Document,
    completions: Vec<CompletionItem>,
}

impl<'a> CompletionCollector<'a> {
    fn new(
        params: &'a TextDocumentPositionParams,
        document: &'a lspml_document_store::Document,
    ) -> CompletionCollector<'a> {
        CompletionCollector {
            cursor: params.position,
            file: &params.text_document.uri,
            document,
            completions: Vec::new(),
        }
    }

    fn search_completions_in_document(&mut self) {
        match self.document.tree.node_at(self.cursor) {
            Some(Node::Tag(tag)) if let ParsedTag::Valid(tag) = &**tag => {
                self.search_completions_in_tag(tag)
            }
            Some(Node::Tag(tag)) if let ParsedTag::Erroneous(tag, errors) = &**tag => {
                for error in errors {
                    match error {
                        TagError::Superfluous(_, location) if location.contains(&self.cursor) => {
                            return self.complete_attributes_of(tag);
                        }
                        TagError::Missing(text, _) => {
                            self.completions.push(CompletionItem {
                                detail: None,
                                documentation: None,
                                insert_text: Some(text.to_string()),
                                kind: Some(CompletionItemKind::SNIPPET),
                                ..Default::default()
                            });
                        }
                        err => log::debug!("unhandled tag error {:?} in {:?}", err, tag),
                    }
                }
                self.search_completions_in_tag(tag);
                // "/>" e.g. is always added as missing right after the last non-whitespace
                // character, terminating it's parent node. that means that our cursor can never be
                // placed after all attributes and still be considered inside the tag node. the
                // only case we could ever trigger this is by moving the cursor in front of an
                // attribute - where it could be arguably okay to suggest "/>" - but that is not
                // the intended usecase here.
                // in order to change this we would have to instead check the closest node before
                // the cursor for missing operators (maybe even skipping text nodes?) in addition
                // to what we're completing originally...
                //
                // if let Some(Node::Tag(ParsedTag::Erroneous(tag, errors))) =
                //     self.document.tree.closest_node_prior_to(self.cursor)
                // {
                //     for error in errors {
                //         if let TagError::Missing(text, _) = error {
                //             self.completions.push(CompletionItem {
                //                 label: format!(
                //                     "close {} tag with \"{}\"",
                //                     tag.definition().name,
                //                     text
                //                 ),
                //                 kind: Some(CompletionItemKind::OPERATOR), // ?
                //                 insert_text: Some(text.to_string()),
                //                 ..Default::default()
                //             })
                //         }
                //     }
                // }
            }
            Some(Node::Tag(tag)) if let ParsedTag::Unparsable(text, location) = &**tag => {
                self.completions.push(CompletionItem {
                    label: format!("unparsable \"{}\"", text),
                    kind: Some(CompletionItemKind::TEXT),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                        new_text: text.to_string(),
                        range: location.range(),
                    })),
                    ..Default::default()
                })
            }
            Some(mut node @ Node::Error(ErrorNode { content, range })) => {
                let mut closest_tag = None;
                loop {
                    (node, closest_tag) = match self.document.tree.parent_of(node) {
                        Some(parent @ Node::Tag(tag)) if let ParsedTag::Valid(tag) = &**tag => {
                            (parent, Some(tag))
                        }
                        Some(parent @ Node::Tag(tag))
                            if let ParsedTag::Erroneous(tag, errors) = &**tag =>
                        {
                            let new_text = errors.iter().find_map(|error| match error {
                                TagError::Missing(text, _) if text.starts_with(&**content) => {
                                    Some(text)
                                }
                                _ => None,
                            });
                            if let Some(new_text) = new_text {
                                return self.completions.push(CompletionItem {
                                    label: new_text.to_string(),
                                    kind: Some(CompletionItemKind::KEYWORD),
                                    insert_text: Some(new_text.to_string()),
                                    text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                                        new_text: new_text.to_string(),
                                        range: *range,
                                    })),
                                    ..Default::default()
                                });
                            }
                            (parent, Some(tag))
                        }
                        Some(parent) => (parent, None),
                        None => break,
                    };
                }
                if self.complete_attributes_of_unterminated_tag() {
                    return;
                }
                if let Some(tag) = closest_tag {
                    self.complete_attributes_of(tag);
                } else {
                    self.complete_top_level_tags();
                }
            }
            _ => {
                if !self.complete_attributes_of_unterminated_tag() {
                    self.complete_top_level_tags();
                }
            }
        }
    }

    /// tree-sitter does not consider an unterminated start tag a tag if the cursor is placed
    /// behind it. `<sp:if ` for example becomes an error node `<sp:if`, which ends before the
    /// cursor, or gets merged into one large error node together with its surrounding tags.
    /// Therefore this inspects the text in front of the cursor instead and completes the
    /// attributes of the tag, if that text ends in an unterminated spml start tag.
    fn complete_attributes_of_unterminated_tag(&mut self) -> bool {
        match self
            .text_before_cursor()
            .and_then(find_unterminated_start_tag)
        {
            Some((definition, present)) => {
                self.complete_attributes(&definition, |name| present.contains(&name));
                true
            }
            None => false,
        }
    }

    fn text_before_cursor(&self) -> Option<&'a str> {
        let text: &'a str = &self.document.text;
        let mut line_start = 0;
        for _ in 0..self.cursor.line {
            line_start += text[line_start..].find('\n')? + 1;
        }
        let line_end = text[line_start..]
            .find('\n')
            .map_or(text.len(), |index| line_start + index);
        let cursor = line_start + self.cursor.character as usize;
        match cursor <= line_end {
            true => text.get(..cursor),
            false => None,
        }
    }

    fn complete_top_level_tags(&mut self) {
        self.complete_tags(lspml_grammar::TOP_LEVEL_TAGS.iter());
    }

    fn complete_tags<'t>(&mut self, tags: impl Iterator<Item = &'t TagDefinition>) {
        let range = self.determine_tag_range();
        tags.map(|tag| Self::tag_to_completion(tag, range))
            .for_each(|completion| self.completions.push(completion));
    }

    fn complete_tag(&mut self, tag: &TagDefinition) {
        self.completions
            .push(Self::tag_to_completion(tag, self.determine_tag_range()));
    }

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
        Range {
            start: Position {
                line: self.cursor.line,
                character: start,
            },
            end: Position {
                line: self.cursor.line,
                character: self.cursor.character,
            },
        }
    }

    fn tag_to_completion(tag: &TagDefinition, range: Range) -> CompletionItem {
        let new_text = format!("<{}", tag.name);
        CompletionItem {
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
        }
    }

    fn search_completions_in_tag(&mut self, tag: &SpmlTag) {
        if let Some(body) = tag.body()
            && self.is_cursor_in_body(body)
        {
            if self.complete_attributes_of_unterminated_tag() {
                return;
            }
            return self.complete_children_of(tag);
        }
        for (name, attribute) in &tag.spel_attributes() {
            let attribute = match attribute {
                ParsedAttribute::Valid(attribute) => attribute,
                ParsedAttribute::Erroneous(attribute, _) => attribute,
                ParsedAttribute::Unparsable(_, _) => continue,
            };
            if attribute.value.is_inside(&self.cursor) {
                let definition = tag.definition();
                let attribute_type = name
                    .strip_suffix("_attribute")
                    .and_then(|name| definition.attributes.get_by_name(name));
                return match &attribute.value.spel {
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
                        lspml_modules::all_modules().iter().for_each(|(name, _)| {
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
                                AttributeRule::ValueOneOf(attribute_name, values)
                                | AttributeRule::ValueOneOfCaseInsensitive(
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
                            let module_attribute = match tag.spel_attribute(module_attribute) {
                                Some(ParsedAttribute::Valid(attribute)) => Some(attribute),
                                Some(ParsedAttribute::Erroneous(attribute, _)) => Some(attribute),
                                _ => None,
                            };
                            if let Some(SpelAttribute {
                                value:
                                    SpelAttributeValue {
                                        spel: SpelAst::String(SpelResult::Valid(Word { fragments })),
                                        ..
                                    },
                                ..
                            }) = module_attribute
                                && fragments.len() == 1
                                && let WordFragment::String(StringLiteral { content, .. }) =
                                    &fragments[0]
                            {
                                module = lspml_modules::find_module_by_name(content);
                            }
                        }
                        let module = module.or_else(|| {
                            lspml_document_store::uri_to_path(self.file)
                                .ok()
                                .and_then(|path| lspml_modules::find_module_for_file(&path))
                        });
                        if let Some(module) = &module
                            && let Err(err) = self.complete_uri(uri, module)
                        {
                            log::error!("failed to complete uri: {}", err);
                        }
                    }
                    _ => (),
                };
            }
        }
        self.complete_attributes_of(tag)
    }

    fn is_cursor_in_body(&self, body: &TagBody) -> bool {
        if (self.cursor.line as usize) < body.open_location.line {
            return false;
        }
        if (self.cursor.line as usize) == body.open_location.line
            && (self.cursor.character as usize) <= body.open_location.char
        {
            return false;
        }
        true
    }

    fn complete_uri(&mut self, uri: &Uri, module: &lspml_modules::Module) -> Result<()> {
        let path = uri.to_string();
        for entry in fs::read_dir(module.path.clone() + path.as_str())? {
            let entry = entry?;
            let name = if path.is_empty() {
                "/".to_string() + entry.file_name().to_str().unwrap()
            } else {
                entry.file_name().to_str().unwrap().to_string()
            };
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
        Ok(())
    }

    fn complete_attributes_of(&mut self, tag: &SpmlTag) {
        self.complete_attributes(&tag.definition(), |name| tag.spel_attribute(name).is_some());
    }

    fn complete_attributes(
        &mut self,
        definition: &TagDefinition,
        is_present: impl Fn(&str) -> bool,
    ) {
        if let TagAttributes::These(possible) = definition.attributes {
            possible
                .iter()
                .filter(|attribute| !is_present(attribute.name))
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

    fn complete_children_of(&mut self, tag: &SpmlTag) {
        match tag.definition().children {
            TagChildren::Any => self.complete_top_level_tags(),
            TagChildren::None => (),
            TagChildren::Scalar(tag) => self.complete_tag(tag),
            TagChildren::Vector(tags) => self.complete_tags(tags.iter()),
        };
    }
}

/// Finds the spml start tag that `text` ends in, if that tag is not terminated yet and `text`
/// ends either in between its attributes or within a partially typed attribute name.
/// Returns the definition of that tag together with the names of its attributes.
fn find_unterminated_start_tag(text: &str) -> Option<(TagDefinition, Vec<&str>)> {
    let is_name_char = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ':');
    let (name_end, kind) = text.rmatch_indices('<').find_map(|(index, _)| {
        let rest = &text[index + 1..];
        let name = &rest[..rest.find(|c| !is_name_char(c)).unwrap_or(rest.len())];
        let kind = match name.split_once(':') {
            Some(("sp", name)) => format!("{}_tag", name),
            Some(("spt", name)) => format!("spt_{}_tag", name),
            _ => return None,
        };
        Some((index + 1 + name.len(), kind))
    })?;
    let definition = TagDefinition::from_str(&kind).ok()?;
    let mut rest = &text[name_end..];
    // without whitespace the tag name itself is still being typed
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let mut attributes = Vec::new();
    loop {
        rest = rest.trim_start();
        let (name, remainder) =
            rest.split_at(rest.find(|c| !is_name_char(c)).unwrap_or(rest.len()));
        if remainder.is_empty() {
            return Some((definition, attributes));
        }
        if name.is_empty() {
            return None;
        }
        let value = remainder.trim_start().strip_prefix('=')?.trim_start();
        let quote = value.chars().next().filter(|c| matches!(c, '"' | '\''))?;
        let value_length = value[1..].find(quote)?;
        attributes.push(name);
        rest = &value[value_length + 2..];
    }
}

pub(crate) fn complete(params: CompletionParams) -> Result<Vec<CompletionItem>, LsError> {
    let text_params = params.text_document_position;
    let uri = &text_params.text_document.uri;
    let document = match lspml_document_store::get(uri) {
        Some(document) => Ok(document),
        None => lspml_document_store::Document::from_uri(uri)
            .map(|document| lspml_document_store::put(uri, document))
            .map_err(|err| {
                log::error!("failed to read {:?}: {}", uri, err);
                LsError {
                    message: format!("cannot read file {:?}", uri),
                    code: ErrorCode::RequestFailed,
                }
            }),
    }?;
    let mut completion_collector = CompletionCollector::new(&text_params, &document);
    completion_collector.search_completions_in_document();
    Ok(completion_collector.completions)
}

#[cfg(test)]
mod tests {
    use lsp_types::{
        CompletionItem, CompletionItemKind, Position, TextDocumentIdentifier,
        TextDocumentPositionParams,
    };

    use super::{CompletionCollector, find_unterminated_start_tag};

    const HEADER: &str = concat!(
        "<%@ page language=\"java\" pageEncoding=\"UTF-8\" contentType=\"text/html; charset=UTF-8\"\n",
        "%><%@ taglib uri=\"http://www.sitepark.com/taglibs/core\" prefix=\"sp\"\n",
        "%><%@ taglib tagdir=\"/WEB-INF/tags/spt\" prefix=\"spt\"\n",
        "%><%@ taglib tagdir=\"/WEB-INF/tags/tag\" prefix=\"tag\"\n",
        "%>\n",
    );

    const SP_IF_ATTRIBUTES: [&str; 13] = [
        "condition",
        "contains",
        "eq",
        "gt",
        "gte",
        "ic",
        "isNull",
        "locale",
        "lt",
        "lte",
        "match",
        "name",
        "neq",
    ];

    /// completes at the position of `|` in `body`, which is appended to a jsp header.
    fn complete(body: &str) -> Vec<CompletionItem> {
        let text = format!("{}{}", HEADER, body);
        let cursor = text.find('|').expect("body should contain a cursor marker");
        let before_cursor = &text[..cursor];
        let position = Position {
            line: before_cursor.matches('\n').count() as u32,
            character: before_cursor.rsplit('\n').next().unwrap_or("").len() as u32,
        };
        let document = lspml_document_store::Document::new(text.replacen('|', "", 1)).unwrap();
        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: "file:///some/test/file.spml".parse().unwrap(),
            },
            position,
        };
        let mut completion_collector = CompletionCollector::new(&params, &document);
        completion_collector.search_completions_in_document();
        completion_collector.completions
    }

    fn attributes(completions: &[CompletionItem]) -> Vec<&str> {
        completions
            .iter()
            .filter(|completion| completion.kind == Some(CompletionItemKind::PROPERTY))
            .map(|completion| completion.label.as_str())
            .collect()
    }

    fn tags(completions: &[CompletionItem]) -> Vec<&str> {
        completions
            .iter()
            .filter(|completion| completion.label.starts_with('<'))
            .map(|completion| completion.label.as_str())
            .collect()
    }

    fn sp_if_attributes_except(present: &[&str]) -> Vec<&'static str> {
        SP_IF_ATTRIBUTES
            .into_iter()
            .filter(|attribute| !present.contains(attribute))
            .collect()
    }

    fn assert_completes_attributes(body: &str, expected: &[&str]) {
        let completions = complete(body);
        assert_eq!(
            attributes(&completions),
            expected,
            "attributes for {:?}",
            body
        );
        assert_eq!(
            tags(&completions),
            Vec::<&str>::new(),
            "tags for {:?}",
            body
        );
    }

    fn assert_completes_tags(body: &str) {
        let completions = complete(body);
        assert_eq!(
            attributes(&completions),
            Vec::<&str>::new(),
            "attributes for {:?}",
            body
        );
        assert!(
            tags(&completions).contains(&"<sp:print"),
            "tags for {:?}",
            body
        );
    }

    #[test]
    fn test_completion_for_attributes_of_unterminated_tag() {
        assert_completes_attributes("<sp:if |", &SP_IF_ATTRIBUTES);
        assert_completes_attributes("<sp:if \t |", &SP_IF_ATTRIBUTES);
        assert_completes_attributes("<sp:if\n\t|", &SP_IF_ATTRIBUTES);
        assert_completes_attributes("<sp:print |", &attributes(&complete("<sp:print |/>")));
    }

    #[test]
    fn test_completion_for_partially_typed_attribute_of_unterminated_tag() {
        assert_completes_attributes("<sp:if n|", &SP_IF_ATTRIBUTES);
        assert_completes_attributes("<sp:if na|", &SP_IF_ATTRIBUTES);
    }

    #[test]
    fn test_completion_for_attributes_of_unterminated_tag_followed_by_content() {
        assert_completes_attributes("<sp:if |\n<sp:print name=\"x\"/>\n", &SP_IF_ATTRIBUTES);
        assert_completes_attributes("<sp:if n|\n<sp:print name=\"x\"/>\n", &SP_IF_ATTRIBUTES);
        assert_completes_attributes(
            "<sp:print |\n<div></div>\n",
            &attributes(&complete("<sp:print |/>")),
        );
        assert_completes_attributes("<div>\n<sp:if |\n</div>\n", &SP_IF_ATTRIBUTES);
    }

    #[test]
    fn test_completion_for_attributes_of_unterminated_tag_with_attributes() {
        let expected = sp_if_attributes_except(&["name"]);
        assert_completes_attributes("<sp:if name=\"x\" |", &expected);
        assert_completes_attributes("<sp:if name=\"x\" n|", &expected);
        assert_completes_attributes("<sp:if name=\"x\"\n    |", &expected);
        assert_completes_attributes("<sp:if name=\"x\"\n    |\n<div></div>\n", &expected);
        assert_completes_attributes("<sp:if name=\"x\" |\n<sp:print name=\"x\"/>\n", &expected);
        assert_completes_attributes(
            "<sp:if name=\"x\" eq='y > z' |",
            &sp_if_attributes_except(&["name", "eq"]),
        );
    }

    #[test]
    fn test_completion_for_attributes_in_nested_tag() {
        assert_completes_attributes(
            concat!(
                "<sp:include module=\"test-module\" uri=\"/functions/doSomething.spml\">\n",
                "\t<sp:argument |\n",
                "</sp:include>\n",
            ),
            &[
                "condition",
                "default",
                "expression",
                "locale",
                "name",
                "object",
                "value",
            ],
        );
        assert_completes_attributes(
            "<sp:print name=\"y\">\n<sp:if |\n</sp:print>\n",
            &SP_IF_ATTRIBUTES,
        );
        assert_completes_attributes(
            "<sp:if name=\"y\">\n<sp:print name=\"y\"/>\n<sp:if n|\n</sp:if>\n",
            &SP_IF_ATTRIBUTES,
        );
    }

    #[test]
    fn test_completion_for_attributes_does_not_include_those_after_the_cursor() {
        assert_completes_attributes(
            "<sp:if name=\"x\" | eq=\"y\">\n</sp:if>\n",
            &sp_if_attributes_except(&["name", "eq"]),
        );
    }

    #[test]
    fn test_completion_still_suggests_missing_tag_end() {
        let completions = complete("<sp:if name=\"x\" |");
        assert!(
            completions
                .iter()
                .any(|completion| completion.insert_text.as_deref() == Some("/>"))
        );
    }

    #[test]
    fn test_completion_for_tags_in_body() {
        assert_completes_tags("<sp:if name=\"x\">\n|");
        assert_completes_tags("<sp:if name=\"x\">\n|\n</sp:if>\n");
        assert_completes_tags("<sp:if name=\"x\">\n<sp:print name=\"y\"/>\n|\n</sp:if>\n");
    }

    #[test]
    fn test_completion_for_tags_after_terminated_tag() {
        assert_completes_tags("<sp:print name=\"x\"/> |");
        assert_completes_tags("<sp:print name=\"x\"/>\n|");
        assert_completes_tags("<sp:print name=\"x\"/>\n|\n<div></div>\n");
        assert_completes_tags("<a href=\"<sp:print name=\"x\"/>\"> |");
    }

    #[test]
    fn test_completion_for_tags_while_typing_tag_name() {
        assert_completes_tags("<sp:if|");
        assert_completes_tags("<sp:i|");
        assert_completes_tags("<sp:if name=\"x\">\n<sp:pri|\n</sp:if>\n");
    }

    #[test]
    fn test_completion_for_attribute_values() {
        let completions = complete("<spt:worklist command=\"|\"/>\n");
        let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(labels, vec!["create", "update"]);
        assert_eq!(
            attributes(&complete("<sp:if name=\"x\" eq=\"|")),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn test_completion_for_closing_tag_does_not_suggest_attributes() {
        assert_eq!(
            attributes(&complete("<sp:if name=\"x\">\n</|")),
            Vec::<&str>::new()
        );
        assert_eq!(
            attributes(&complete("<sp:if name=\"x\">\n</sp:i|")),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn test_find_unterminated_start_tag() {
        let find = |text| {
            find_unterminated_start_tag(text)
                .map(|(definition, attributes)| (definition.name, attributes))
        };
        assert_eq!(find("<sp:if "), Some(("sp:if", vec![])));
        assert_eq!(find("<sp:if na"), Some(("sp:if", vec![])));
        assert_eq!(find("<spt:date "), Some(("spt:date", vec![])));
        assert_eq!(
            find("<sp:if name=\"a<b>c\"\n\tcondition='x' n"),
            Some(("sp:if", vec!["name", "condition"])),
        );
        assert_eq!(find("<sp:if"), None);
        assert_eq!(find("<sp:if name"), Some(("sp:if", vec![])));
        assert_eq!(find("<sp:if name "), None);
        assert_eq!(find("<sp:if name="), None);
        assert_eq!(find("<sp:if name=\"x"), None);
        assert_eq!(find("<sp:if name=\"x\">"), None);
        assert_eq!(find("<sp:if name=\"x\"/> "), None);
        assert_eq!(find("<sp:unknown "), None);
        assert_eq!(find("<tag:custom "), None);
        assert_eq!(find("<div "), None);
        assert_eq!(find(""), None);
    }
}
