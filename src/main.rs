use anyhow::Result;
use clap::Parser;
use lsp_server::{Connection, Message, Response};
use lsp_types::{
    CancelParams, CompletionItem, CompletionItemKind, CompletionOptions,
    CompletionOptionsCompletionItem, CompletionParams, CompletionResponse, Diagnostic,
    DiagnosticOptions, DiagnosticServerCapabilities, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    DocumentDiagnosticParams, Documentation, FullDocumentDiagnosticReport, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverContents, HoverOptions, HoverParams,
    HoverProviderCapability, InitializeParams, InsertTextMode, Location, MarkedString, OneOf,
    Position, Range, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    WorkDoneProgressOptions,
};
use std::{error::Error, fmt, fs::File, path::Path};
use structured_logger::Builder;
use tree_sitter::{Query, QueryCursor};
mod document_store;
mod grammar;
mod parser;
mod project;

#[derive(Parser, Debug)]
#[clap(name = "lspml")]
struct CommandLineOpts {
    #[clap(long)]
    log_file: Option<String>,
    #[clap(long, default_value = "INFO")]
    log_level: String,
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let opts = CommandLineOpts::parse();

    Builder::with_level(&opts.log_level)
        .with_target_writer(
            "*",
            opts.log_file
                .clone()
                .and_then(|file| File::options().create(true).append(true).open(file).ok())
                .map(|file| structured_logger::json::new_writer(file))
                .unwrap_or_else(|| structured_logger::json::new_writer(std::io::stderr())),
        )
        .init();
    log::info!("lspml starting...");
    log::trace!("commandline opts: {:?}", &opts);

    let (connection, io_threads) = Connection::stdio();
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        definition_provider: Some(OneOf::Left(true)),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::NONE)),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            inter_file_dependencies: true,
            ..DiagnosticOptions::default()
        })),
        completion_provider: Some(CompletionOptions {
            completion_item: Some(CompletionOptionsCompletionItem {
                label_details_support: Some(true),
            }),
            ..CompletionOptions::default()
        }),
        hover_provider: Some(HoverProviderCapability::Options(HoverOptions {
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: Some(true),
            },
        })),
        ..ServerCapabilities::default()
    })?;
    let initialization_params = match connection.initialize(server_capabilities) {
        Ok(params) => serde_json::from_value(params)?,
        Err(err) => {
            if err.channel_is_disconnected() {
                io_threads.join()?;
            }
            return Err(err.into());
        }
    };

    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    log::info!("shutting down lspml...");
    return Ok(());
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
    // MethodNotFound = -32601,
    // InvalidRequest = -32600,
    // ServerNotInitialized = -32002,
    // UnknownErrorCode = -32001,
}

impl fmt::Display for ResponseErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return write!(formatter, "{}", self.to_string());
    }
}

#[derive(Debug)]
struct LsError {
    message: String,
    code: ResponseErrorCode,
}

impl Error for LsError {}

impl fmt::Display for LsError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return write!(formatter, "{}: {}", self.code, self.message);
    }
}

fn main_loop(
    connection: Connection,
    _initialization_params: InitializeParams,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    log::info!("server started");

    for message in &connection.receiver {
        match message {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    return Ok(());
                }
                match request.method.as_str() {
                    "textDocument/completion" => {
                        log::debug!("got completion request: {request:?}");
                        connection.sender.send(Message::Response(
                            match complete(serde_json::from_value(request.params)?) {
                                Ok(completions) => Response {
                                    id: request.id,
                                    result: serde_json::to_value(CompletionResponse::Array(
                                        completions,
                                    ))
                                    .ok(),
                                    error: None,
                                },
                                Err(err) => Response {
                                    id: request.id,
                                    result: None,
                                    error: Some(lsp_server::ResponseError {
                                        message: err.message,
                                        code: err.code as i32,
                                        data: None,
                                    }),
                                },
                            },
                        ))?;
                    }
                    "textDocument/definition" => {
                        log::debug!("got go to definition request: {request:?}");
                        connection.sender.send(Message::Response(
                            match definition(serde_json::from_value(request.params)?) {
                                Ok(definition) => Response {
                                    id: request.id,
                                    result: definition.and_then(|d| {
                                        serde_json::to_value(GotoDefinitionResponse::Scalar(d)).ok()
                                    }),
                                    error: None,
                                },
                                Err(err) => Response {
                                    id: request.id,
                                    result: None,
                                    error: Some(lsp_server::ResponseError {
                                        message: err.message,
                                        code: err.code as i32,
                                        data: None,
                                    }),
                                },
                            },
                        ))?;
                    }
                    "textDocument/diagnostic" => {
                        log::debug!("got diagnose request: {request:?}");
                        connection.sender.send(Message::Response(
                            match diagnose(serde_json::from_value(request.params)?) {
                                Ok(result) => Response {
                                    id: request.id,
                                    result: serde_json::to_value(FullDocumentDiagnosticReport {
                                        result_id: None,
                                        items: result,
                                    })
                                    .ok(),
                                    error: None,
                                },
                                Err(err) => Response {
                                    id: request.id,
                                    result: None,
                                    error: Some(lsp_server::ResponseError {
                                        message: err.message,
                                        code: err.code as i32,
                                        data: None,
                                    }),
                                },
                            },
                        ))?;
                    }
                    "textDocument/hover" => {
                        log::debug!("got hover request: {request:?}");
                        connection.sender.send(Message::Response(
                            match hover(serde_json::from_value(request.params)?) {
                                Ok(result) => Response {
                                    id: request.id,
                                    result: result.and_then(|value| {
                                        serde_json::to_value(Hover {
                                            contents: value,
                                            range: None,
                                        })
                                        .ok()
                                    }),
                                    error: None,
                                },
                                Err(err) => Response {
                                    id: request.id,
                                    result: None,
                                    error: Some(lsp_server::ResponseError {
                                        message: err.message,
                                        code: err.code as i32,
                                        data: None,
                                    }),
                                },
                            },
                        ))?;
                    }
                    _ => log::info!("got unknonwn request: {request:?}"),
                }
            }
            Message::Response(response) => {
                log::info!("got unknown response: {response:?}");
            }
            Message::Notification(notification) => match notification.method.as_str() {
                "textDocument/didChange" => {
                    changed(serde_json::from_value(notification.params)?)?;
                }
                "textDocument/didOpen" => {
                    opened(serde_json::from_value(notification.params)?)?;
                }
                "textDocument/didSave" => {
                    saved(serde_json::from_value(notification.params)?)?;
                }
                "$/cancelRequest" => {
                    let params: CancelParams = serde_json::from_value(notification.params).unwrap();
                    log::debug!("attempted to cancel request {:?}", params.id);
                }
                _ => log::info!("got unknown notification: {notification:?}"),
            },
        }
    }

    return Ok(());
}

// is probably called on every key hit when TextDocumentSyncKind is INCREMENTAL.
fn changed(params: DidChangeTextDocumentParams) -> Result<()> {
    return document_store::Document::new(&params.text_document.uri).map(|document| {
        document_store::put(&params.text_document.uri, document);
        log::debug!("updated {}", params.text_document.uri);
        return ();
    });
}

fn opened(params: DidOpenTextDocumentParams) -> Result<()> {
    return match document_store::get(&params.text_document.uri) {
        Some(_) => Result::Ok(()),
        None => document_store::Document::new(&params.text_document.uri).map(|document| {
            document_store::put(&params.text_document.uri, document);
            log::debug!("opened {}", params.text_document.uri);
            return ();
        }),
    };
}

fn saved(params: DidSaveTextDocumentParams) -> Result<()> {
    return document_store::Document::new(&params.text_document.uri).map(|document| {
        document_store::put(&params.text_document.uri, document);
        log::debug!("updated {}", params.text_document.uri);
        return ();
    });
}

fn complete(params: CompletionParams) -> Result<Vec<CompletionItem>, LsError> {
    let text_params = params.text_document_position;
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
    let (node, previous) =
        parser::find_current_and_previous_nodes(&document.tree, text_params.position).ok_or_else(
            || LsError {
                message: format!(
                    "could not determine node in {} at line {}, character {}",
                    text_params.text_document.uri,
                    text_params.position.line,
                    text_params.position.character
                ),
                code: ResponseErrorCode::RequestFailed,
            },
        )?;
    return match node.kind() {
        "text" | "document" => Ok(grammar::SpTag::iter()
            .map(|tag| tag.properties())
            .map(|properties| CompletionItem {
                kind: Some(CompletionItemKind::METHOD),
                detail: properties.detail,
                documentation: properties.documentation,
                insert_text: Some(properties.name),
                insert_text_mode: Some(InsertTextMode::AS_IS),
                ..Default::default()
            })
            .collect()),
        "include_tag" => match previous.map(|p| p.kind()) {
            Some(">") | Some("argument_tag") => Ok(vec![CompletionItem {
                kind: Some(CompletionItemKind::METHOD),
                detail: grammar::SpTag::Argument.properties().detail,
                documentation: grammar::SpTag::Argument.properties().documentation,
                insert_text: Some(grammar::SpTag::Argument.properties().name),
                insert_text_mode: Some(InsertTextMode::AS_IS),
                ..Default::default()
            }]),
            Some("argument_tag_open") => Ok(vec![CompletionItem {
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(String::from("Attribute(String)")),
                documentation: Some(Documentation::String(String::from(
                    "the name of the argument",
                ))),
                insert_text: Some(String::from("name=\"")),
                insert_text_mode: Some(InsertTextMode::AS_IS),
                ..Default::default()
            }]),
            Some("name_attribute") => {
                match previous.and_then(|p| p.prev_sibling()).map(|p| p.kind()) {
                    Some("argument_tag_open") => Ok(vec![CompletionItem {
                        kind: Some(CompletionItemKind::FIELD),
                        detail: Some(String::from("Attribute(Object)")),
                        documentation: Some(Documentation::String(String::from(
                            "the interpreted value of the argument",
                        ))),
                        insert_text: Some(String::from("object=\"")),
                        insert_text_mode: Some(InsertTextMode::AS_IS),
                        ..Default::default()
                    }]),
                    _ => Ok(Vec::new()),
                }
            }
            _ => Ok(Vec::new()),
        },
        "string" => match previous.map(|p| p.kind()) {
            Some("name=") => match previous
                .and_then(|p| p.parent())
                .and_then(|p| p.prev_sibling())
                .map(|p| p.kind())
            {
                Some("argument_tag_open") => Ok(vec![
                    CompletionItem {
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(String::from("Argument(ID)")),
                        documentation: Some(Documentation::String(String::from(
                            "the itemScope to do something for",
                        ))),
                        insert_text: Some(String::from("itemScope\"")),
                        insert_text_mode: Some(InsertTextMode::AS_IS),
                        ..Default::default()
                    },
                    CompletionItem {
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(String::from("Argument(Map)")),
                        documentation: Some(Documentation::String(String::from(
                            "options to configure the process of doing something",
                        ))),
                        insert_text: Some(String::from("options\"")),
                        insert_text_mode: Some(InsertTextMode::AS_IS),
                        ..Default::default()
                    },
                ]),
                _ => Ok(Vec::new()),
            },
            _ => Ok(Vec::new()),
        },
        // "start_tag" =>
        // "attribute" =>
        // "attribute_name" =>
        // "quoted_attribute_value" =>
        // "attribute_value" =>
        // "raw_text" =>
        // "end_tag" =>
        // "self_closing_tag" =>
        // "error" =>
        // "expression_statement" =>
        // "member_expression" =>
        // "object" =>
        // "property" =>
        _ => Ok(Vec::new()),
    };
}

/**
 * variables (check)
 * includes (check)
 * imports
 * object params and functions? (would probably have to jump into java sources..)
 */
fn definition(params: GotoDefinitionParams) -> Result<Option<Location>, LsError> {
    let text_params = params.text_document_position_params;
    let document = match document_store::get(&text_params.text_document.uri) {
        Some(document) => Ok(document),
        None => document_store::Document::new(&text_params.text_document.uri)
            .map(|document|
                document_store::put(
                    &text_params.text_document.uri,
                    document,
                )
            )
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
    let working_directory = project::get_working_directory(&text_params.text_document.uri)
        .expect("cannot determine module - requires path to be <module>/src/main/webapp/");
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
                                ]
                                (.eq? @attribute "\"{variable}\"")
                            )"#
                        );
                        return match Query::new(tree_sitter_spml::language(), qry.as_str()) {
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
                        .and_then(|attribute| attribute.child(1))
                        .map(|node| node.utf8_text(document.text.as_bytes()))
                    {
                        Some(Ok("\"${module.id}\"")) | None => {
                            Some(working_directory.module.as_str())
                        }
                        Some(Ok(module)) => Some(&module[1..module.len() - 1]),
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
                        // .unwrap_or(working_directory.module.as_str());
                        let mut file = working_directory.path;
                        file.push_str(&include_module);
                        file.push_str("/src/main/webapp");
                        file.push_str(&path[1..path.len() - 1]);
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

fn diagnose(params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>, LsError> {
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

fn hover(params: HoverParams) -> Result<Option<HoverContents>, LsError> {
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
    .map(|doc| match doc {
        Documentation::MarkupContent(markup) => HoverContents::Markup(markup),
        Documentation::String(string) => HoverContents::Scalar(MarkedString::String(string)),
    }));
}
