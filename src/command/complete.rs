use super::{LsError, ResponseErrorCode};
use crate::document_store;
use crate::grammar;
use crate::parser;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, Documentation, InsertTextMode,
};

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
    let (node, previous) =
        parser::find_current_and_previous_nodes(&document.tree, text_params.position).ok_or_else(
            || LsError {
                message: format!(
                    "could not determine node in {} at line {}, character {}",
                    text_params.text_document.uri,
                    text_params.position.line,
                    text_params.position.character
                ),
                code: ResponseErrorCode::RequestFailed,
            },
        )?;
    return match node.kind() {
        "text" | "document" => Ok(grammar::SpTag::iter()
            .map(|tag| tag.properties())
            .map(|properties| CompletionItem {
                kind: Some(CompletionItemKind::METHOD),
                detail: properties.detail,
                documentation: properties.documentation,
                insert_text: Some(properties.name),
                insert_text_mode: Some(InsertTextMode::AS_IS),
                ..Default::default()
            })
            .collect()),
        "include_tag" => match previous.map(|p| p.kind()) {
            Some(">") | Some("argument_tag") => Ok(vec![CompletionItem {
                kind: Some(CompletionItemKind::METHOD),
                detail: grammar::SpTag::Argument.properties().detail,
                documentation: grammar::SpTag::Argument.properties().documentation,
                insert_text: Some(grammar::SpTag::Argument.properties().name),
                insert_text_mode: Some(InsertTextMode::AS_IS),
                ..Default::default()
            }]),
            Some("argument_tag_open") => Ok(vec![CompletionItem {
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(String::from("Attribute(String)")),
                documentation: Some(Documentation::String(String::from(
                    "the name of the argument",
                ))),
                insert_text: Some(String::from("name=\"")),
                insert_text_mode: Some(InsertTextMode::AS_IS),
                ..Default::default()
            }]),
            Some("name_attribute") => {
                match previous.and_then(|p| p.prev_sibling()).map(|p| p.kind()) {
                    Some("argument_tag_open") => Ok(vec![CompletionItem {
                        kind: Some(CompletionItemKind::FIELD),
                        detail: Some(String::from("Attribute(Object)")),
                        documentation: Some(Documentation::String(String::from(
                            "the interpreted value of the argument",
                        ))),
                        insert_text: Some(String::from("object=\"")),
                        insert_text_mode: Some(InsertTextMode::AS_IS),
                        ..Default::default()
                    }]),
                    _ => Ok(Vec::new()),
                }
            }
            _ => Ok(Vec::new()),
        },
        "string" => match previous.map(|p| p.kind()) {
            Some("name=") => match previous
                .and_then(|p| p.parent())
                .and_then(|p| p.prev_sibling())
                .map(|p| p.kind())
            {
                Some("argument_tag_open") => Ok(vec![
                    CompletionItem {
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(String::from("Argument(ID)")),
                        documentation: Some(Documentation::String(String::from(
                            "the itemScope to do something for",
                        ))),
                        insert_text: Some(String::from("itemScope\"")),
                        insert_text_mode: Some(InsertTextMode::AS_IS),
                        ..Default::default()
                    },
                    CompletionItem {
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(String::from("Argument(Map)")),
                        documentation: Some(Documentation::String(String::from(
                            "options to configure the process of doing something",
                        ))),
                        insert_text: Some(String::from("options\"")),
                        insert_text_mode: Some(InsertTextMode::AS_IS),
                        ..Default::default()
                    },
                ]),
                _ => Ok(Vec::new()),
            },
            _ => Ok(Vec::new()),
        },
        // "start_tag" =>
        // "attribute" =>
        // "attribute_name" =>
        // "quoted_attribute_value" =>
        // "attribute_value" =>
        // "raw_text" =>
        // "end_tag" =>
        // "self_closing_tag" =>
        // "error" =>
        // "expression_statement" =>
        // "member_expression" =>
        // "object" =>
        // "property" =>
        _ => Ok(Vec::new()),
    };
}
