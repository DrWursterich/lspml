use anyhow::{Error, Result};
use lsp_server::{Message, Request, RequestId, Response, ResponseError};
use lsp_types::{
    CompletionResponse, FullDocumentDiagnosticReport, GotoDefinitionResponse, SemanticTokens,
    SemanticTokensResult,
};
use std::fmt;
mod complete;
mod definition;
mod diagnostic;
mod highlight;
mod hover;
mod semantics;

#[derive(Debug)]
struct LsError {
    message: String,
    code: ResponseErrorCode,
}

impl std::error::Error for LsError {}

impl fmt::Display for LsError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return write!(formatter, "{}: {}", self.code, self.message);
    }
}

impl LsError {
    fn to_response(self, request_id: RequestId) -> Response {
        return Response {
            id: request_id,
            result: None,
            error: Some(ResponseError {
                message: self.message,
                code: self.code as i32,
                data: None,
            }),
        };
    }
}

#[derive(Debug)]
enum ResponseErrorCode {
    RequestFailed = -32803,
    // ServerCancelled = -32802,
    // ContentModified = -32801,
    // RequestCancelled = -32800,
    // ParseError = -32700,
    // InternalError = -32603,
    // InvalidParams = -32602,
    MethodNotFound = -32601,
    // InvalidRequest = -32600,
    // ServerNotInitialized = -32002,
    // UnknownErrorCode = -32001,
}

impl fmt::Display for ResponseErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return write!(formatter, "{}", self.to_string());
    }
}

pub(crate) fn complete(request: Request) -> Result<Message> {
    log::trace!("got completion request: {request:?}");
    return serde_json::from_value(request.params)
        .map(|params| {
            Message::Response(match complete::complete(params) {
                Ok(completions) => Response {
                    id: request.id,
                    result: serde_json::to_value(CompletionResponse::Array(completions)).ok(),
                    error: None,
                },
                Err(err) => err.to_response(request.id),
            })
        })
        .map_err(|err| Error::from(err));
}

pub(crate) fn definition(request: Request) -> Result<Message> {
    log::trace!("got go to definition request: {request:?}");
    return serde_json::from_value(request.params)
        .map(|params| {
            Message::Response(match definition::definition(params) {
                Ok(definition) => Response {
                    id: request.id,
                    result: definition
                        .and_then(|d| serde_json::to_value(GotoDefinitionResponse::Scalar(d)).ok()),
                    error: None,
                },
                Err(err) => err.to_response(request.id),
            })
        })
        .map_err(|err| Error::from(err));
}

pub(crate) fn diagnostic(request: Request) -> Result<Message> {
    log::trace!("got diagnose request: {request:?}");
    return serde_json::from_value(request.params)
        .map(|params| {
            Message::Response(match diagnostic::diagnostic(params) {
                Ok(diagnostic) => Response {
                    id: request.id,
                    result: serde_json::to_value(FullDocumentDiagnosticReport {
                        result_id: None,
                        items: diagnostic,
                    })
                    .ok(),
                    error: None,
                },
                Err(err) => err.to_response(request.id),
            })
        })
        .map_err(|err| Error::from(err));
}

pub(crate) fn highlight(request: Request) -> Result<Message> {
    log::trace!("got highlight request: {request:?}");
    return serde_json::from_value(request.params)
        .map(|params| {
            Message::Response(match highlight::highlight(params) {
                Ok(highlights) => Response {
                    id: request.id,
                    result: serde_json::to_value(highlights).ok(),
                    error: None,
                },
                Err(err) => err.to_response(request.id),
            })
        })
        .map_err(|err| Error::from(err));
}

pub(crate) fn hover(request: Request) -> Result<Option<Message>> {
    log::trace!("got hover request: {request:?}");
    return Ok(
        match hover::hover(serde_json::from_value(request.params)?) {
            Ok(Some(result)) => Some(Message::Response(Response {
                id: request.id,
                result: Some(serde_json::to_value(result)?),
                error: None,
            })),
            Ok(None) => None,
            Err(err) => Some(Message::Response(err.to_response(request.id))),
        },
    );
}

pub(crate) fn semantics(request: Request) -> Result<Message> {
    log::trace!("got semantics request: {request:?}");
    return serde_json::from_value(request.params)
        .map(|params| {
            Message::Response(match semantics::semantics(params) {
                Ok(tokens) => Response {
                    id: request.id,
                    result: serde_json::to_value(SemanticTokensResult::Tokens(SemanticTokens {
                        result_id: None,
                        data: tokens,
                    }))
                    .ok(),
                    error: None,
                },
                Err(err) => err.to_response(request.id),
            })
        })
        .map_err(|err| Error::from(err));
}

pub(crate) fn unknown(request: Request) -> Result<Message> {
    log::info!("got unknonwn request: {request:?}");
    return Ok(Message::Response(Response {
        id: request.id,
        result: None,
        error: Some(ResponseError {
            message: format!("method \"{}\" not found", request.method),
            code: ResponseErrorCode::MethodNotFound as i32,
            data: None,
        }),
    }));
}
