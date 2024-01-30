use super::{LsError, ResponseErrorCode};
use crate::document_store;
use crate::modules;
use crate::parser;
use lsp_types::{GotoDefinitionParams, Location, Position, Range, Url};
use std::{path::Path, result::Result};
use tree_sitter::{Query, QueryCursor};

/**
 * variables (check)
 * includes (check)
 * imports
 * object params and functions? (would probably have to jump into java sources..)
 */
pub(crate) fn definition(params: GotoDefinitionParams) -> Result<Option<Location>, LsError> {
    let text_params = params.text_document_position_params;
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
    let (node, _) = parser::find_current_and_previous_nodes(&document.tree, text_params.position)
        .ok_or_else(|| LsError {
        message: format!(
            "could not determine node in {} at line {}, character {}",
            text_params.text_document.uri,
            text_params.position.line,
            text_params.position.character
        ),
        code: ResponseErrorCode::RequestFailed,
    })?;
    return match node.kind() {
        // check if string is evaluated ?
        "string" => match node.parent().map(|p| p.kind()) {
            Some("uri_attribute")
                if node
                    .parent()
                    .and_then(|parent| parent.parent())
                    .is_some_and(|tag| tag.kind() == "include_tag") =>
            {
                node.utf8_text(document.text.as_bytes())
                    .map_err(|err| LsError {
                        message: format!("error while reading file: {}", err),
                        code: ResponseErrorCode::RequestFailed,
                    })
                    .map(|path| {
                        node
                            .parent()
                            .and_then(|p| p.parent())
                            .and_then(|p| {
                                p.children(&mut document.tree.walk())
                                    .find(|node| node.kind() == "module_attribute")
                            })
                            .map(|attribute| parser::attribute_value_of(attribute, &document.text))
                            .filter(|module| *module != "${module.id}")
                            .and_then(modules::find_module_by_name)
                            .or_else(|| {
                                text_params.text_document.uri.to_file_path().ok().and_then(
                                    |module| modules::find_module_for_file(module.as_path()),
                                )
                            })
                            .map(|module| module.path + &path[1..path.len() - 1])
                            .filter(|file| Path::new(&file).exists())
                            .map(|file| Location {
                                range: Range {
                                    ..Default::default()
                                },
                                uri: Url::parse(format!("file://{}", &file).as_str()).unwrap(),
                            })
                    })
            }
            Some("name_attribute")
                if node
                    .parent()
                    .and_then(|parent| parent.parent())
                    .is_some_and(|tag| tag.kind() == "argument_tag") =>
            {
                // goto param definition in included file
                return Ok(None);
            }
            Some(kind) if kind.ends_with("_attribute") => {
                let variable = &node.utf8_text(document.text.as_bytes()).unwrap();
                let variable = &variable[1..variable.len() - 1];
                return match Query::new(
                    tree_sitter_spml::language(),
                    &create_definition_query(variable).as_str(),
                ) {
                    Ok(query) => Ok(QueryCursor::new()
                        .matches(&query, document.tree.root_node(), document.text.as_bytes())
                        .into_iter()
                        .flat_map(|m| m.captures.iter())
                        .map(|c| c.node)
                        .min_by(|a, b| a.start_position().cmp(&b.start_position()))
                        .map(|result| Location {
                            range: Range {
                                start: Position {
                                    line: result.start_position().row as u32,
                                    character: result.start_position().column as u32 + 1,
                                },
                                end: Position {
                                    line: result.end_position().row as u32,
                                    character: result.end_position().column as u32 - 1,
                                },
                            },
                            uri: text_params.text_document.uri,
                        })),
                    Err(err) => {
                        log::error!("error in definition query of {}: {}", variable, err);
                        return Err(LsError {
                            message: format!("error in definition query of {}: {}", variable, err),
                            code: ResponseErrorCode::RequestFailed,
                        });
                    }
                };
            }
            _ => Ok(None),
        },
        // TODO: "java_code" | "script_tag" | "style_tag" | "interpolated_string"
        _ => Ok(None),
    };
}

fn create_definition_query<'a>(variable: &'a str) -> String {
    return format!(
        r#"
(
    [
        (attribute_tag
            (name_attribute
                (string) @attribute))
        (barcode_tag
            (name_attribute
                (string) @attribute))
        (calendarsheet_tag
            (name_attribute
                (string) @attribute))
        (checkbox_tag
            (name_attribute
                (string) @attribute))
        (
            (collection_tag
                (name_attribute
                    (string) @attribute)
                (action_attribute
                    (string) @action))
            (.eq? @action "\"new\"")
        )
        (diff_tag
            (name_attribute
                (string) @attribute))
        (filter_tag
            (name_attribute
                (string) @attribute))
        (for_tag
            (index_attribute
                (string) @attribute))
        (hidden_tag
            (name_attribute
                (string) @attribute))
        (include_tag
            (return_attribute
                (string) @attribute))
        (iterator_tag
            (item_attribute
                (string) @attribute))
        (json_tag
            (name_attribute
                (string) @attribute))
        (linkedInformation_tag
            (name_attribute
                (string) @attribute))
        (linktree_tag
            (name_attribute
                (string) @attribute))
        (livetree_tag
            (name_attribute
                (string) @attribute))
        (loop_tag
            (item_attribute
                (string) @attribute))
        (
            (map_tag
                (name_attribute
                    (string) @attribute)
                (action_attribute
                    (string) @action))
            (.eq? @action "\"new\"")
        )
        (querytree_tag
            (name_attribute
                (string) @attribute))
        (radio_tag
            (name_attribute
                (string) @attribute))
        (range_tag
            (name_attribute
                (string) @attribute))
        (sass_tag
            (name_attribute
                (string) @attribute))
        (scaleimage_tag
            (name_attribute
                (string) @attribute))
        (search_tag
            (name_attribute
                (string) @attribute))
        (select_tag
            (name_attribute
                (string) @attribute))
        (set_tag
            (name_attribute
                (string) @attribute))
        (sort_tag
            (name_attribute
                (string) @attribute))
        (subinformation_tag
            (name_attribute
                (string) @attribute))
        (text_tag
            (name_attribute
                (string) @attribute))
        (textarea_tag
            (name_attribute
                (string) @attribute))
        (textimage_tag
            (name_attribute
                (string) @attribute))
        (upload_tag
            (name_attribute
                (string) @attribute))
        (worklist_tag
            (name_attribute
                (string) @attribute))
        (zip_tag
            (name_attribute
                (string) @attribute))
        (spt_counter_tag
            (name_attribute
                (string) @attribute))
        (spt_date_tag
            (name_attribute
                (string) @attribute))
        (spt_email2img_tag
            (name_attribute
                (string) @attribute))
        (spt_encryptemail_tag
            (name_attribute
                (string) @attribute))
        (spt_escapeemail_tag
            (name_attribute
                (string) @attribute))
        (spt_formsolutions_tag
            (name_attribute
                (string) @attribute))
        (spt_id2url_tag
            (name_attribute
                (string) @attribute))
        (spt_imageeditor_tag
            (name_attribute
                (string) @attribute))
        (spt_iterator_tag
            (name_attribute
                (string) @attribute))
        (spt_link_tag
            (name_attribute
                (string) @attribute))
        (spt_number_tag
            (name_attribute
                (string) @attribute))
        (spt_personalization_tag
            (name_attribute
                (string) @attribute))
        (spt_prehtml_tag
            (name_attribute
                (string) @attribute))
        (spt_smarteditor_tag
            (name_attribute
                (string) @attribute))
        (spt_text_tag
            (name_attribute
                (string) @attribute))
        (spt_textarea_tag
            (name_attribute
                (string) @attribute))
        (spt_timestamp_tag
            (connect_attribute
                (string) @attribute))
        (spt_tinymce_tag
            (name_attribute
                (string) @attribute))
        (spt_updown_tag
            (name_attribute
                (string) @attribute))
        (spt_upload_tag
            (name_attribute
                (string) @attribute))
    ]
    (.eq? @attribute "\"{}\"")
)"#,
        variable,
    );
}
