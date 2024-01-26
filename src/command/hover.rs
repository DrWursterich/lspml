use super::{LsError, ResponseErrorCode};
use crate::document_store;
use crate::grammar;
use crate::parser;
use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};
use std::str::FromStr;

pub(crate) fn hover(params: HoverParams) -> Result<Option<Hover>, LsError> {
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
    return Ok((match node.kind() {
        "argument_tag_open" | "argument_tag_close" => {
            grammar::Tag::SpArgument.properties().documentation
        }
        "attribute_tag_open" => grammar::Tag::SpAttribute.properties().documentation,
        "barcode_tag_open" => grammar::Tag::SpBarcode.properties().documentation,
        "break_tag_open" => grammar::Tag::SpBreak.properties().documentation,
        "calendarsheet_tag_open" | "calendarsheet_tag_close" => {
            grammar::Tag::SpCalendarsheet.properties().documentation
        }
        "checkbox_tag_open" | "checkbox_tag_close" => {
            grammar::Tag::SpCheckbox.properties().documentation
        }
        "code_tag_open" | "code_tag_close" => grammar::Tag::SpCode.properties().documentation,
        "collection_tag_open" | "collection_tag_close" => {
            grammar::Tag::SpCollection.properties().documentation
        }
        "condition_tag_open" | "condition_tag_close" => {
            grammar::Tag::SpCondition.properties().documentation
        }
        "diff_tag_open" | "diff_tag_close" => grammar::Tag::SpDiff.properties().documentation,
        "else_tag_open" | "else_tag_close" => grammar::Tag::SpElse.properties().documentation,
        "elseif_tag_open" | "elseif_tag_close" => grammar::Tag::SpElseif.properties().documentation,
        "error_tag_open" | "error_tag_close" => grammar::Tag::SpError.properties().documentation,
        "expire_tag_open" | "expire_tag_close" => grammar::Tag::SpExpire.properties().documentation,
        "filter_tag_open" | "filter_tag_close" => grammar::Tag::SpFilter.properties().documentation,
        "for_tag_open" | "for_tag_close" => grammar::Tag::SpFor.properties().documentation,
        "form_tag_open" | "form_tag_close" => grammar::Tag::SpForm.properties().documentation,
        "hidden_tag_open" | "hidden_tag_close" => grammar::Tag::SpHidden.properties().documentation,
        "if_tag_open" | "if_tag_close" => grammar::Tag::SpIf.properties().documentation,
        "include_tag_open" | "include_tag_close" => {
            grammar::Tag::SpInclude.properties().documentation
        }
        "io_tag_open" | "io_tag_close" => grammar::Tag::SpIo.properties().documentation,
        "iterator_tag_open" | "iterator_tag_close" => {
            grammar::Tag::SpIterator.properties().documentation
        }
        "json_tag_open" | "json_tag_close" => grammar::Tag::SpJson.properties().documentation,
        "linktree_tag_open" | "linktree_tag_close" => {
            grammar::Tag::SpLinktree.properties().documentation
        }
        "linkedinformation_tag_open" | "linkedinformation_tag_close" => {
            grammar::Tag::SpLinkedInformation.properties().documentation
        }
        "livetree_tag_open" => grammar::Tag::SpLivetree.properties().documentation,
        "log_tag_open" | "log_tag_close" => grammar::Tag::SpLog.properties().documentation,
        "login_tag_open" => grammar::Tag::SpLogin.properties().documentation,
        "loop_tag_open" | "loop_tag_close" => grammar::Tag::SpLoop.properties().documentation,
        "map_tag_open" | "map_tag_close" => grammar::Tag::SpMap.properties().documentation,
        "option_tag_open" | "option_tag_close" => grammar::Tag::SpOption.properties().documentation,
        "password_tag_open" | "password_tag_close" => {
            grammar::Tag::SpPassword.properties().documentation
        }
        "print_tag_open" | "print_tag_close" => grammar::Tag::SpPrint.properties().documentation,
        "querytree_tag_open" | "querytree_tag_close" => {
            grammar::Tag::SpQuerytree.properties().documentation
        }
        "radio_tag_open" | "radio_tag_close" => grammar::Tag::SpRadio.properties().documentation,
        "range_tag_open" | "range_tag_close" => grammar::Tag::SpRange.properties().documentation,
        "return_tag_open" | "return_tag_close" => grammar::Tag::SpReturn.properties().documentation,
        "sass_tag_open" | "sass_tag_close" => grammar::Tag::SpSass.properties().documentation,
        "scaleimage_tag_open" => grammar::Tag::SpScaleimage.properties().documentation,
        "scope_tag_open" | "scope_tag_close" => grammar::Tag::SpScope.properties().documentation,
        "search_tag_open" | "search_tag_close" => grammar::Tag::SpSearch.properties().documentation,
        "select_tag_open" | "select_tag_close" => grammar::Tag::SpSelect.properties().documentation,
        "set_tag_open" | "set_tag_close" => grammar::Tag::SpSet.properties().documentation,
        "sort_tag_open" | "sort_tag_close" => grammar::Tag::SpSort.properties().documentation,
        "subinformation_tag_open" | "subinformation_tag_close" => {
            grammar::Tag::SpSubinformation.properties().documentation
        }
        "tagbody_tag_open" => grammar::Tag::SpTagbody.properties().documentation,
        "text_tag_open" | "text_tag_close" => grammar::Tag::SpText.properties().documentation,
        "textarea_tag_open" | "textarea_tag_close" => {
            grammar::Tag::SpTextarea.properties().documentation
        }
        "textimage_tag_open" => grammar::Tag::SpTextimage.properties().documentation,
        "throw_tag_open" => grammar::Tag::SpThrow.properties().documentation,
        "toggle_tag_open" | "toggle_tag_close" => grammar::Tag::SpToggle.properties().documentation,
        "upload_tag_open" | "upload_tag_close" => grammar::Tag::SpUpload.properties().documentation,
        "url_tag_open" | "url_tag_close" => grammar::Tag::SpUrl.properties().documentation,
        "warning_tag_open" | "warning_tag_close" => {
            grammar::Tag::SpWarning.properties().documentation
        }
        "worklist_tag_open" | "worklist_tag_close" => {
            grammar::Tag::SpWorklist.properties().documentation
        }
        "zip_tag_open" | "zip_tag_close" => grammar::Tag::SpZip.properties().documentation,
        "spt_counter_tag_open" => grammar::Tag::SptCounter.properties().documentation,
        "spt_date_tag_open" => grammar::Tag::SptDate.properties().documentation,
        "spt_diff_tag_open" => grammar::Tag::SptDiff.properties().documentation,
        "spt_email2img_tag_open" => grammar::Tag::SptEmail2img.properties().documentation,
        "spt_encryptemail_tag_open" => grammar::Tag::SptEncryptemail.properties().documentation,
        "spt_escapeemail_tag_open" => grammar::Tag::SptEscapeemail.properties().documentation,
        "spt_formsolutions_tag_open" => grammar::Tag::SptFormsolutions.properties().documentation,
        "spt_id2url_tag_open" => grammar::Tag::SptId2url.properties().documentation,
        "spt_ilink_tag_open" => grammar::Tag::SptIlink.properties().documentation,
        "spt_imageeditor_tag_open" => grammar::Tag::SptImageeditor.properties().documentation,
        "spt_imp_tag_open" => grammar::Tag::SptImp.properties().documentation,
        "spt_iterator_tag_open" | "spt_iterator_tag_close" => {
            grammar::Tag::SptIterator.properties().documentation
        }
        "spt_link_tag_open" => grammar::Tag::SptLink.properties().documentation,
        "spt_number_tag_open" => grammar::Tag::SptNumber.properties().documentation,
        "spt_personalization_tag_open" => {
            grammar::Tag::SptPersonalization.properties().documentation
        }
        "spt_prehtml_tag_open" => grammar::Tag::SptPrehtml.properties().documentation,
        "spt_smarteditor_tag_open" => grammar::Tag::SptSmarteditor.properties().documentation,
        "spt_spml_tag_open" => grammar::Tag::SptSpml.properties().documentation,
        "spt_text_tag_open" => grammar::Tag::SptText.properties().documentation,
        "spt_textarea_tag_open" => grammar::Tag::SptTextarea.properties().documentation,
        "spt_timestamp_tag_open" => grammar::Tag::SptTimestamp.properties().documentation,
        "spt_tinymce_tag_open" => grammar::Tag::SptTinymce.properties().documentation,
        "spt_updown_tag_open" => grammar::Tag::SptUpdown.properties().documentation,
        "spt_upload_tag_open" => grammar::Tag::SptUpload.properties().documentation,
        "spt_worklist_tag_open" => grammar::Tag::SptWorklist.properties().documentation,
        kind => match node.parent() {
            Some(parent) if parent.kind().ends_with("_attribute") => match parent
                .parent()
                .and_then(|parent| grammar::Tag::from_str(parent.kind()).ok())
                .map(|tag| tag.properties().attributes)
            {
                Some(grammar::TagAttributes::These(attributes)) => {
                    let kind = &parent.kind();
                    let attribute_name = &kind[..kind.len() - "_attribute".len()];
                    attributes
                        .iter()
                        .find(|attribute| attribute.name == attribute_name)
                        .and_then(|attribute| attribute.documentation)
                }
                _ => {
                    log::info!("no hover information about node \"{}\"", kind);
                    return Ok(None);
                }
            },
            _ => {
                log::info!("no hover information about node \"{}\"", kind);
                return Ok(None);
            }
        },
    })
    .map(|doc| Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: doc.to_string(),
        }),
        range: None,
    }));
}
