use anyhow::{Error, Result};
use lsp_server::{Message, Request, RequestId, Response, ResponseError};
use lsp_types::{CompletionResponse, FullDocumentDiagnosticReport, GotoDefinitionResponse};
use std::fmt;
mod complete;
mod definition;
mod diagnostic;
mod hover;

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
        }
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
    log::debug!("got completion request: {request:?}");
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
    log::debug!("got go to definition request: {request:?}");
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
    log::debug!("got diagnose request: {request:?}");
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

pub(crate) fn hover(request: Request) -> Result<Message> {
    log::debug!("got hover request: {request:?}");
    return serde_json::from_value(request.params)
        .map(|params| {
            Message::Response(match hover::hover(params) {
                Ok(hover) => Response {
                    id: request.id,
                    result: hover.and_then(|hover| serde_json::to_value(hover).ok()),
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
