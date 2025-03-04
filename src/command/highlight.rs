use lsp_types::{
    DocumentHighlight, DocumentHighlightKind, DocumentHighlightParams, Position, Range,
};

use super::LsError;

/**
 * this highlights occurences of a hovered identifier - not the entire file!
 */
pub(crate) fn highlight(
    _params: DocumentHighlightParams,
) -> Result<Vec<DocumentHighlight>, LsError> {
    // let uri = params.text_document_position_params.text_document.uri;
    // let document = match document_store::get(&uri) {
    //     Some(document) => Ok(document),
    //     None => document_store::Document::from_uri(&uri)
    //         .map(|document| document_store::put(&uri, document))
    //         .map_err(|err| {
    //             log::error!("failed to read {}: {}", uri, err);
    //             return LsError {
    //                 message: format!("cannot read file {}", uri),
    //                 code: ErrorCode::RequestFailed,
    //             };
    //         }),
    // }?;
    // TODO: implementation!
    let highlight = DocumentHighlight {
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 0,
            },
        },
        kind: Some(DocumentHighlightKind::TEXT),
    };
    return Ok(vec![highlight]);
}
