use super::{LsError, ResponseErrorCode};
use crate::document_store;
use crate::grammar;
use crate::parser;
use lsp_types::{Documentation, Hover, HoverContents, HoverParams, MarkedString};

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
            grammar::SpTag::Argument.properties().documentation
        }
        "attribute_tag_open" => grammar::SpTag::Attribute.properties().documentation,
        "barcode_tag_open" => grammar::SpTag::Barcode.properties().documentation,
        "break_tag_open" => grammar::SpTag::Break.properties().documentation,
        "calendarsheet_tag_open" | "calendarsheet_tag_close" => {
            grammar::SpTag::Calendarsheet.properties().documentation
        }
        "checkbox_tag_open" | "checkbox_tag_close" => {
            grammar::SpTag::Checkbox.properties().documentation
        }
        "code_tag_open" | "code_tag_close" => grammar::SpTag::Code.properties().documentation,
        "collection_tag_open" | "collection_tag_close" => {
            grammar::SpTag::Collection.properties().documentation
        }
        "condition_tag_open" | "condition_tag_close" => {
            grammar::SpTag::Condition.properties().documentation
        }
        "diff_tag_open" | "diff_tag_close" => grammar::SpTag::Diff.properties().documentation,
        "else_tag_open" | "else_tag_close" => grammar::SpTag::Else.properties().documentation,
        "elseif_tag_open" | "elseif_tag_close" => grammar::SpTag::Elseif.properties().documentation,
        "error_tag_open" | "error_tag_close" => grammar::SpTag::Error.properties().documentation,
        "expire_tag_open" | "expire_tag_close" => grammar::SpTag::Expire.properties().documentation,
        "filter_tag_open" | "filter_tag_close" => grammar::SpTag::Filter.properties().documentation,
        "for_tag_open" | "for_tag_close" => grammar::SpTag::For.properties().documentation,
        "form_tag_open" | "form_tag_close" => grammar::SpTag::Form.properties().documentation,
        "hidden_tag_open" | "hidden_tag_close" => grammar::SpTag::Hidden.properties().documentation,
        "if_tag_open" | "if_tag_close" => grammar::SpTag::If.properties().documentation,
        "include_tag_open" | "include_tag_close" => {
            grammar::SpTag::Include.properties().documentation
        }
        "io_tag_open" | "io_tag_close" => grammar::SpTag::Io.properties().documentation,
        "iterator_tag_open" | "iterator_tag_close" => {
            grammar::SpTag::Iterator.properties().documentation
        }
        "json_tag_open" | "json_tag_close" => grammar::SpTag::Json.properties().documentation,
        "linktree_tag_open" | "linktree_tag_close" => {
            grammar::SpTag::Linktree.properties().documentation
        }
        "linkedinformation_tag_open" | "linkedinformation_tag_close" => {
            grammar::SpTag::LinkedInformation.properties().documentation
        }
        "livetree_tag_open" => grammar::SpTag::Livetree.properties().documentation,
        "log_tag_open" | "log_tag_close" => grammar::SpTag::Log.properties().documentation,
        "login_tag_open" => grammar::SpTag::Login.properties().documentation,
        "loop_tag_open" | "loop_tag_close" => grammar::SpTag::Loop.properties().documentation,
        "map_tag_open" | "map_tag_close" => grammar::SpTag::Map.properties().documentation,
        "option_tag_open" | "option_tag_close" => grammar::SpTag::Option.properties().documentation,
        "password_tag_open" | "password_tag_close" => {
            grammar::SpTag::Password.properties().documentation
        }
        "print_tag_open" | "print_tag_close" => grammar::SpTag::Print.properties().documentation,
        "querytree_tag_open" | "querytree_tag_close" => {
            grammar::SpTag::Querytree.properties().documentation
        }
        "radio_tag_open" | "radio_tag_close" => grammar::SpTag::Radio.properties().documentation,
        "range_tag_open" | "range_tag_close" => grammar::SpTag::Range.properties().documentation,
        "return_tag_open" | "return_tag_close" => grammar::SpTag::Return.properties().documentation,
        "sass_tag_open" | "sass_tag_close" => grammar::SpTag::Sass.properties().documentation,
        "scaleimage_tag_open" => grammar::SpTag::Scaleimage.properties().documentation,
        "scope_tag_open" | "scope_tag_close" => grammar::SpTag::Scope.properties().documentation,
        "search_tag_open" | "search_tag_close" => grammar::SpTag::Search.properties().documentation,
        "select_tag_open" | "select_tag_close" => grammar::SpTag::Select.properties().documentation,
        "set_tag_open" | "set_tag_close" => grammar::SpTag::Set.properties().documentation,
        "sort_tag_open" | "sort_tag_close" => grammar::SpTag::Sort.properties().documentation,
        "subinformation_tag_open" | "subinformation_tag_close" => {
            grammar::SpTag::Subinformation.properties().documentation
        }
        "tagbody_tag_open" => grammar::SpTag::Tagbody.properties().documentation,
        "text_tag_open" | "text_tag_close" => grammar::SpTag::Text.properties().documentation,
        "textarea_tag_open" | "textarea_tag_close" => {
            grammar::SpTag::Textarea.properties().documentation
        }
        "textimage_tag_open" => grammar::SpTag::Textimage.properties().documentation,
        "throw_tag_open" => grammar::SpTag::Throw.properties().documentation,
        "toggle_tag_open" | "toggle_tag_close" => grammar::SpTag::Toggle.properties().documentation,
        "upload_tag_open" | "upload_tag_close" => grammar::SpTag::Upload.properties().documentation,
        "url_tag_open" | "url_tag_close" => grammar::SpTag::Url.properties().documentation,
        "warning_tag_open" | "warning_tag_close" => {
            grammar::SpTag::Warning.properties().documentation
        }
        "worklist_tag_open" | "worklist_tag_close" => {
            grammar::SpTag::Worklist.properties().documentation
        }
        "zip_tag_open" | "zip_tag_close" => grammar::SpTag::Zip.properties().documentation,
        kind => {
            log::info!("no hover information about {}", kind);
            return Ok(None);
        }
    })
    .map(|doc| Hover {
        contents: match doc {
            Documentation::MarkupContent(markup) => HoverContents::Markup(markup),
            Documentation::String(string) => HoverContents::Scalar(MarkedString::String(string)),
        },
        range: None,
    }));
}
