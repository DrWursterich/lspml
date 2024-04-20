use lsp_server::ErrorCode;
use lsp_types::{
    DocumentHighlight, DocumentHighlightKind, DocumentHighlightParams, Position, Range,
};

use crate::document_store;

use super::LsError;

/**
 * this highlights occurences of a hovered identifier - not the entire file!
 */
pub(crate) fn highlight(
    params: DocumentHighlightParams,
) -> Result<Vec<DocumentHighlight>, LsError> {
    let uri = params.text_document_position_params.text_document.uri;
    let document = match document_store::get(&uri) {
        Some(document) => Ok(document),
        None => document_store::Document::from_uri(&uri)
            .map(|document| document_store::put(&uri, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", uri, err);
                return LsError {
                    message: format!("cannot read file {}", uri),
                    code: ErrorCode::RequestFailed,
                };
            }),
    }?;
    let _tree = document.tree.root_node();
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
