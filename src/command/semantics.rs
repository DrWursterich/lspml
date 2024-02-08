use super::{
    super::{TOKEN_MODIFIERS, TOKEN_TYPES},
    LsError, ResponseErrorCode,
};
use crate::document_store;
use lsp_types::{SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensParams};
use tree_sitter::Node;

/**
 * this adds highlighting details for small tokens - not the entire file!
 */
pub(crate) fn semantics(params: SemanticTokensParams) -> Result<Vec<SemanticToken>, LsError> {
    let uri = params.text_document.uri;
    let document = match document_store::get(&uri) {
        Some(document) => Ok(document),
        None => document_store::Document::new(&uri)
            .map(|document| document_store::put(&uri, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", uri, err);
                return LsError {
                    message: format!("cannot read file {}", uri),
                    code: ResponseErrorCode::RequestFailed,
                };
            }),
    }?;
    let root = document.tree.root_node();
    // TODO: actual implementation!
    let token = create_token(
        root.child(0)
            .unwrap()
            .child(2)
            .unwrap()
            .child(2)
            .unwrap()
            .child(1)
            .unwrap(),
        SemanticTokenType::FUNCTION,
        vec![SemanticTokenModifier::DEPRECATED],
    );
    return Ok(vec![token]);
}

fn create_token(
    node: Node,
    r#type: SemanticTokenType,
    modifiers: Vec<SemanticTokenModifier>,
) -> SemanticToken {
    let token_modifiers_bitset = TOKEN_MODIFIERS
        .iter()
        .enumerate()
        .filter_map(|(index, modifier)| match modifiers.contains(modifier) {
            true => Some(1 << index as u32),
            false => None,
        })
        .sum::<u32>();
    return SemanticToken {
        delta_line: node.start_position().row as u32,
        delta_start: node.start_position().column as u32,
        length: (node.end_byte() - node.start_byte()) as u32,
        token_type: TOKEN_TYPES
            .iter()
            .enumerate()
            .find_map(|(index, token_type)| match *token_type == r#type {
                true => Some(index as u32),
                false => None,
            })
            .expect("no function token exists"),
        token_modifiers_bitset,
        // token_modifiers_bitset: TOKEN_MODIFIERS
        //     .iter()
        //     .enumerate()
        //     .filter_map(|(index, modifier)| match modifiers.contains(modifier) {
        //         true => Some(1 << index as u32),
        //         false => None,
        //     })
        //     .sum::<u32>(),
    };
}
