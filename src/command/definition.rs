use super::{LsError, ResponseErrorCode};
use crate::document_store;
use crate::parser;
use crate::modules;
use lsp_types::{GotoDefinitionParams, Location, Position, Range, Url};
use std::path::Path;
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
        // if string is not evaluated ....
        "string" => match node.parent().map(|p| p.kind()) {
            Some("name_attribute") => {
                match node.parent().and_then(|p| p.parent()).map(|p| p.kind()) {
                    Some("argument_tag") => Ok(None), // would be nice
                    _ => {
                        let variable = &node.utf8_text(document.text.as_bytes()).unwrap();
                        let variable = &variable[1..variable.len() - 1];
                        let qry = format!(
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
                                (.eq? @attribute "\"{variable}\"")
                            )"#
                        );
                        return match Query::new(tree_sitter_spml::language(), qry.as_str()) {
                            Ok(query) => Ok(QueryCursor::new()
                                .matches(
                                    &query,
                                    document.tree.root_node(),
                                    document.text.as_bytes(),
                                )
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
                                    message: format!(
                                        "error in definition query of {}: {}",
                                        variable, err
                                    ),
                                    code: ResponseErrorCode::RequestFailed,
                                });
                            }
                        };
                    }
                }
            }
            Some("uri_attribute") => match node.parent().and_then(|p| p.parent()).map(|p| p.kind())
            {
                Some("include_tag") => match &node.utf8_text(document.text.as_bytes()) {
                    Ok(path) => Ok(match node
                        .parent()
                        .and_then(|p| p.parent())
                        .and_then(|p| {
                            p.children(&mut document.tree.walk())
                                .find(|node| node.kind() == "module_attribute")
                        })
                        .and_then(|attribute| attribute.child(2))
                        .map(|node| node.utf8_text(document.text.as_bytes()))
                    {
                        Some(Ok("\"${module.id}\"")) | None => {
                            text_params.text_document.uri
                            .to_file_path()
                            .ok()
                            .and_then(|file| modules::find_module_for_file(file.as_path()))
                        }
                        Some(Ok(module)) => modules::find_module_by_name(&module[1..module.len() - 1]),
                        Some(Err(err)) => {
                            log::error!(
                                "error while reading include_tag module_attribute text {}",
                                err
                            );
                            return Err(LsError {
                                message: format!(
                                    "error while reading include_tag module_attribute text {}",
                                    err
                                ),
                                code: ResponseErrorCode::RequestFailed,
                            });
                        }
                    }
                    .and_then(|include_module| {
                        let file = include_module.path + &path[1..path.len() - 1];
                        if !Path::new(&file).exists() {
                            log::info!("included file {} does not exist", file);
                            return None;
                        }
                        let mut target = "file://".to_owned();
                        target.push_str(&file);
                        return Some(Location {
                            range: Range {
                                ..Default::default()
                            },
                            uri: Url::parse(&target).unwrap(),
                        });
                    })),
                    Err(err) => {
                        log::error!("error while reading include_tag uri_attribute text {}", err);
                        return Err(LsError {
                            message: format!(
                                "error while reading include_tag uri_attribute text {}",
                                err
                            ),
                            code: ResponseErrorCode::RequestFailed,
                        });
                    }
                },
                _ => Ok(None),
            },
            _ => Ok(None),
        },
        "interpolated_string" => {
            return Ok(None);
        }
        // TODO:
        "java_code" => Ok(None),
        "tag_code" => Ok(None),
        _ => Ok(None),
    };
}
