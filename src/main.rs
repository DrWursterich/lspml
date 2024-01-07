use anyhow::Result;
use lsp_server::{Connection, Message, Response};
use lsp_types::{
    CancelParams, CompletionOptions, CompletionOptionsCompletionItem, CompletionParams,
    DidSaveTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, InitializeParams,
    Location, OneOf, Position, Range, ServerCapabilities, Url,
};
use std::error::Error;
use tree_sitter::{Parser, Point};
mod parser;
mod project;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // logging to stderr as stdout is used for result messages
    eprintln!("lspml starting...");

    let (connection, io_threads) = Connection::stdio();
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        definition_provider: Some(OneOf::Left(true)),
        completion_provider: Some(CompletionOptions {
            completion_item: Some(CompletionOptionsCompletionItem {
                label_details_support: Some(true),
                ..CompletionOptionsCompletionItem::default()
            }),
            ..CompletionOptions::default()
        }),
        ..Default::default()
    })
    .unwrap();
    let initialization_params = match connection.initialize(server_capabilities) {
        Ok(params) => serde_json::from_value(params).unwrap(),
        Err(err) => {
            if err.channel_is_disconnected() {
                io_threads.join()?;
            }
            return Err(err.into());
        }
    };

    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    eprintln!("shutting down lspml...");
    return Ok(());
}

fn main_loop(
    connection: Connection,
    _initialization_params: InitializeParams,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("server started, entering main loop");

    for message in &connection.receiver {
        match message {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    return Ok(());
                }
                match request.method.as_str() {
                    "textDocument/definition" => {
                        eprintln!("got go to definition request: {request:?}");
                        let result = definition(serde_json::from_value(request.params)?);
                        connection.sender.send(Message::Response(Response {
                            id: request.id,
                            result: result.map(|value| {
                                serde_json::to_value(GotoDefinitionResponse::Scalar(value)).unwrap()
                            }),
                            error: None,
                        }))?;
                    }
                    "textDocument/completion" => {
                        eprintln!("got completion request: {request:?}");
                        complete(serde_json::from_value(request.params)?);
                    }
                    _ => eprintln!("got unknonwn request: {request:?}"),
                }
            }
            Message::Response(response) => {
                eprintln!("got unknown response: {response:?}");
            }
            Message::Notification(notification) => match notification.method.as_str() {
                "textDocument/didSave" => {
                    let params: DidSaveTextDocumentParams =
                        serde_json::from_value(notification.params).unwrap();
                    eprintln!("{} was saved", params.text_document.uri);
                }
                "$/cancelRequest" => {
                    let params: CancelParams = serde_json::from_value(notification.params).unwrap();
                    eprintln!("attempted to cancel request {:?}", params.id);
                }
                _ => eprintln!("got unknown notification: {notification:?}"),
            },
        }
    }

    return Ok(());
}

/**
 * variables (check)
 * includes (check)
 * imports
 * object params and functions? (would probably have to jump into java sources..)
 */
fn definition(params: GotoDefinitionParams) -> Option<Location> {
    let text_params = params.text_document_position_params;
    let text = parser::get_text_document(&text_params).unwrap();
    let target_line = text_params.position.line as usize;
    for (line_count, line) in text.lines().enumerate() {
        if line_count < target_line {
            continue;
        }
        // if there happens to be an sp:include tag on this line we assume thats where we want to
        // go to. we (currently) don't check wether the cursor is inside it.
        if let Some(include) = parser::find_include_uri(&line) {
            let working_directory = project::get_working_directory(text_params.text_document.uri).unwrap();
            let target_module = include
                .module
                .filter(|module| module != "${module.id}")
                .or(Some(working_directory.module))
                .unwrap();
            let mut target = "file://".to_owned();
            target.push_str(&working_directory.path);
            target.push_str(&target_module);
            target.push_str("/src/main/webapp");
            target.push_str(&include.uri);
            return Some(Location {
                range: Range {
                    ..Default::default()
                },
                uri: Url::parse(&target).unwrap(),
            });
        }
        let keyword = parser::find_keyword(&line, text_params.position.character as usize).unwrap();
        // otherwise we search for the first appearance of the keyword in question
        for (line_count, line) in text.lines().enumerate() {
            if let Some(index) = line.find(keyword) {
                eprintln!(
                    "first appearance of keyword {} found at line {} and character {}",
                    keyword, line_count, index
                );
                return Some(Location {
                    range: Range {
                        start: Position {
                            line: line_count as u32,
                            character: index as u32,
                        },
                        end: Position {
                            line: line_count as u32,
                            character: (index + keyword.len()) as u32,
                        },
                    },
                    uri: text_params.text_document.uri,
                });
            }
            if line_count > target_line {
                break;
            }
        }
        break;
    }
    return None;
}

fn complete(params: CompletionParams) -> Option<()> {
    let text_params = params.text_document_position;
    let text = parser::get_text_document(&text_params).unwrap();
    let mut parser = Parser::new();
    match parser.set_language(tree_sitter_html::language()) {
        Err(err) => {
            eprintln!("failed to set tree sitter language to html: {}", err);
            return None;
        }
        _ => {}
    }
    eprintln!("created parser");
    let tree = parser.parse(&text, None)?;
    eprintln!("successfully parsed file");
    let root_node = tree.root_node();
    eprintln!("found root_node: {}", root_node.id());
    let trigger_point = Point::new(
        text_params.position.line as usize,
        text_params.position.character as usize,
    );
    let closest_node = root_node.descendant_for_point_range(trigger_point, trigger_point)?;
    eprintln!(
        "closest: {closest_node:?} (kind: {}, content: {})",
        closest_node.kind(),
        closest_node.utf8_text(text.as_bytes()).unwrap()
    );
    match closest_node.kind() {
        "attribute_name" => {}
        "element" => {}
        "fragment" => {}
        _ => {}
    }
    return None;
}
