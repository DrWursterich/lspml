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
    let file = &text_params.text_document.uri;
    let document = match document_store::get(file) {
        Some(document) => Ok(document),
        None => document_store::Document::new(file)
            .map(|document| document_store::put(file, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", file, err);
                return LsError {
                    message: format!("cannot read file {}", file),
                    code: ResponseErrorCode::RequestFailed,
                };
            }),
    }?;
    let node =
        parser::find_current_node(&document.tree, text_params.position).ok_or_else(|| LsError {
            message: format!(
                "could not determine node in {} at line {}, character {}",
                file, text_params.position.line, text_params.position.character
            ),
            code: ResponseErrorCode::RequestFailed,
        })?;
    return match node.kind() {
        // check if string is evaluated ?
        "string_content" => match node.parent().and_then(|p| p.parent()).map(|p| p.kind()) {
            Some("uri_attribute")
                if node
                    .parent()
                    .and_then(|parent| parent.parent())
                    .and_then(|parent| parent.parent())
                    .is_some_and(|tag| tag.kind() == "include_tag") =>
            {
                node.utf8_text(document.text.as_bytes())
                    .map_err(|err| LsError {
                        message: format!("error while reading file: {}", err),
                        code: ResponseErrorCode::RequestFailed,
                    })
                    .map(|path| {
                        node.parent()
                            .and_then(|p| p.parent())
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
                            .map(|module| module.path + &path)
                            .filter(|file| Path::new(&file).exists())
                            .and_then(|file| Url::parse(format!("file://{}", &file).as_str()).ok())
                            .map(|uri| Location {
                                range: Range {
                                    ..Default::default()
                                },
                                uri,
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
                                    character: result.start_position().column as u32,
                                },
                                end: Position {
                                    line: result.end_position().row as u32,
                                    character: result.end_position().column as u32,
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
                (string
                    (string_content) @attribute)))
        (barcode_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (calendarsheet_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (checkbox_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (
            (collection_tag
                (name_attribute
                    (string
                    (string_content) @attribute))
                (action_attribute
                    (string
                        (string_content) @action)))
            (.eq? @action "\"new\"")
        )
        (diff_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (filter_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (for_tag
            (index_attribute
                (string
                    (string_content) @attribute)))
        (hidden_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (include_tag
            (return_attribute
                (string
                    (string_content) @attribute)))
        (iterator_tag
            (item_attribute
                (string
                    (string_content) @attribute)))
        (json_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (linkedInformation_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (linktree_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (livetree_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (loop_tag
            (item_attribute
                (string
                    (string_content) @attribute)))
        (
            (map_tag
                (name_attribute
                    (string
                        (string_content) @attribute))
                (action_attribute
                    (string
                        (string_content) @action)))
            (.eq? @action "\"new\"")
        )
        (querytree_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (radio_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (range_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (sass_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (scaleimage_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (search_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (select_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (set_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (sort_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (subinformation_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (text_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (textarea_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (textimage_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (upload_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (worklist_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (zip_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_counter_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_date_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_email2img_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_encryptemail_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_escapeemail_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_formsolutions_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_id2url_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_imageeditor_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_iterator_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_link_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_number_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_personalization_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_prehtml_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_smarteditor_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_text_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_textarea_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_timestamp_tag
            (connect_attribute
                (string
                    (string_content) @attribute)))
        (spt_tinymce_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_updown_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
        (spt_upload_tag
            (name_attribute
                (string
                    (string_content) @attribute)))
    ]
    (.eq? @attribute "{}")
)"#,
        variable,
    );
}
