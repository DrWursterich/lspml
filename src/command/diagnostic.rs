use super::{LsError, ResponseErrorCode};
use crate::document_store;
use lsp_types::{Diagnostic, DiagnosticSeverity, DocumentDiagnosticParams, Position, Range};
use tree_sitter::{Query, QueryCursor};

pub(crate) fn diagnostic(params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>, LsError> {
    let document = match document_store::get(&params.text_document.uri) {
        Some(document) => Ok(document),
        None => document_store::Document::new(&params.text_document.uri)
            .map(|document| document_store::put(&params.text_document.uri, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", params.text_document.uri, err);
                return LsError {
                    message: format!("cannot read file {}", params.text_document.uri),
                    code: ResponseErrorCode::RequestFailed,
                };
            }),
    }?;
    return Query::new(tree_sitter_spml::language(), "(ERROR)+ @error")
        .map(|query| {
            QueryCursor::new()
                .matches(&query, document.tree.root_node(), document.text.as_bytes())
                .into_iter()
                .flat_map(|m| m.captures.iter())
                .map(|c| c.node)
                .map(|result| Diagnostic {
                    source: Some("lspml".to_string()),
                    message: "syntax error".to_string(),
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
                    severity: Some(DiagnosticSeverity::ERROR),
                    ..Default::default()
                })
                .collect()
        })
        .map_err(|err| {
            log::error!("error in query for ERROR location: {}", err);
            return LsError {
                message: format!("error in query for ERROR location: {}", err),
                code: ResponseErrorCode::RequestFailed,
            };
        });
}
