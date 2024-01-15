use anyhow::Result;
use lsp_server::{Connection, Message, Response};
use lsp_types::{
    CancelParams,
    CompletionItem,
    CompletionItemKind,
    //CompletionItemLabelDetails,
    CompletionOptions,
    CompletionOptionsCompletionItem,
    CompletionParams,
    CompletionResponse,
    DidSaveTextDocumentParams,
    Documentation,
    GotoDefinitionParams,
    GotoDefinitionResponse,
    InitializeParams,
    Location,
    OneOf,
    Position,
    Range,
    ServerCapabilities,
    Url,
};
use std::{error::Error, path::Path};
use tree_sitter::{Parser, Point, Query, QueryCursor};
mod parser;
mod project;
mod symbols;

// format!(
//     r#"
//     (
//         [
//             (attribute_tag
//                 (name_attribute
//                     (string) @attribute))
//             (barcode_tag
//                 (name_attribute
//                     (string) @attribute))
//             (calendarsheet_tag
//                 (name_attribute
//                     (string) @attribute))
//             (checkbox_tag
//                 (name_attribute
//                     (string) @attribute))
//             (
//                 (collection_tag
//                     (name_attribute
//                         (string) @attribute)
//                     (action_attribute
//                         (string) @action))
//                 (#eq? @action "new")
//             )
//             (diff_tag
//                 (name_attribute
//                     (string) @attribute))
//             (filter_tag
//                 (name_attribute
//                     (string) @attribute))
//             (for_tag
//                 (index_attribute
//                     (string) @attribute))
//             (hidden_tag
//                 (name_attribute
//                     (string) @attribute))
//             (include_tag
//                 (return_attribute
//                     (string) @attribute))
//             (iterator_tag
//                 (item_attribute
//                     (string) @attribute))
//             (json_tag
//                 (name_attribute
//                     (string) @attribute))
//             (linkedInformation_tag
//                 (name_attribute
//                     (string) @attribute))
//             (linktree_tag
//                 (name_attribute
//                     (string) @attribute))
//             (livetree_tag
//                 (name_attribute
//                     (string) @attribute))
//             (loop_tag
//                 (item_attribute
//                     (string) @attribute))
//             (
//                 (map_tag
//                     (name_attribute
//                         (string) @attribute)
//                     (action_attribute
//                         (string) @action))
//                 (#eq? @action "new")
//             )
//             (querytree_tag
//                 (name_attribute
//                     (string) @attribute))
//             (radio_tag
//                 (name_attribute
//                     (string) @attribute))
//             (range_tag
//                 (name_attribute
//                     (string) @attribute))
//             (sass_tag
//                 (name_attribute
//                     (string) @attribute))
//             (scaleimage_tag
//                 (name_attribute
//                     (string) @attribute))
//             (search_tag
//                 (name_attribute
//                     (string) @attribute))
//             (select_tag
//                 (name_attribute
//                     (string) @attribute))
//             (set_tag
//                 (name_attribute
//                     (string) @attribute))
//             (sort_tag
//                 (name_attribute
//                     (string) @attribute))
//             (subinformation_tag
//                 (name_attribute
//                     (string) @attribute))
//             (text_tag
//                 (name_attribute
//                     (string) @attribute))
//             (textarea_tag
//                 (name_attribute
//                     (string) @attribute))
//             (textimage_tag
//                 (name_attribute
//                     (string) @attribute))
//             (upload_tag
//                 (name_attribute
//                     (string) @attribute))
//             (worklist_tag
//                 (name_attribute
//                     (string) @attribute))
//             (zip_tag
//                 (name_attribute
//                     (string) @attribute))
//         ]
//         (#eq? @attribute "{variable}")
//     )"#,
//     variable
// )

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
                        let result = complete(serde_json::from_value(request.params)?);
                        connection.sender.send(Message::Response(Response {
                            id: request.id,
                            result: result.map(|value| {
                                serde_json::to_value(CompletionResponse::Array(value)).unwrap()
                            }),
                            error: None,
                        }))?;
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
    let text = parser::get_text_document(&text_params).ok()?;
    let mut parser = Parser::new();
    match parser.set_language(tree_sitter_spml::language()) {
        Err(err) => {
            eprintln!("failed to set tree sitter language to spml: {}", err);
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
    // let node = root_node.descendant_for_point_range(trigger_point, trigger_point)?;
    let mut cursor = root_node.walk();
    let mut node;
    let mut previous;
    loop {
        node = cursor.node();
        if node.end_position() < trigger_point {
            previous = Some(node);
            if !cursor.goto_next_sibling() || cursor.node().start_position() > trigger_point {
                node = node.parent().unwrap();
                break;
            }
        } else if !cursor.goto_first_child() {
            previous = node.prev_sibling();
            break;
        }
    }
    if let Some(prev) = previous {
        if prev.kind() == "ERROR" {
            previous = prev.child(prev.child_count() - 1);
        }
    }
    eprintln!(
        "previous: {previous:?}, closest: {node:?} ({})",
        node.utf8_text(text.as_bytes()).unwrap()
    );
    let working_directory = project::get_working_directory(&text_params.text_document.uri)?;
    return match node.kind() {
        // if string is not evaluated ....
        "string" => match node.parent()?.kind() {
            "name_attribute" => match node.parent()?.parent()?.kind() {
                "argument_tag" => None, // would be nice
                _ => {
                    let variable = &node.utf8_text(text.as_bytes()).unwrap();
                    // let variable = &variable[1..variable.len() - 1];
                    let qry = format!(
                        r#"
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
                            (collection_tag
                                (name_attribute
                                    (string) @attribute))
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
                            (map_tag
                                (name_attribute
                                    (string) @attribute))
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
                        ]"#
                    );
                    return match Query::new(tree_sitter_spml::language(), qry.as_str()) {
                        Ok(query) => QueryCursor::new()
                            .matches(&query, root_node, text.as_bytes())
                            .into_iter()
                            .flat_map(|m| m.captures.iter())
                            .map(|c| {
                                eprintln!(
                                    "query found {c:?} '{}'",
                                    c.node.utf8_text(text.as_bytes()).unwrap()
                                );
                                c.node
                            })
                            // '#eq?' predicates do not work, we have to do it manually:
                            .filter(|n| n.utf8_text(text.as_bytes()).unwrap() == *variable)
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
                            }),
                        Err(err) => {
                            eprintln!("error in query for declaration of {}: {}", variable, err);
                            return None;
                        }
                    };
                }
            },
            "uri_attribute" => match node.parent()?.parent()?.kind() {
                "include_tag" => match &node.utf8_text(text.as_bytes()) {
                    Ok(path) => match node
                        .parent()?
                        .parent()?
                        .children(&mut tree.walk())
                        .find(|node| node.kind() == "module_attribute")
                        .and_then(|attribute| attribute.child(1))
                        .map(|node| node.utf8_text(text.as_bytes()))
                    {
                        Some(Ok("\"${module.id}\"")) | None => {
                            Some(working_directory.module.as_str())
                        }
                        Some(Ok(module)) => Some(&module[1..module.len() - 1]),
                        Some(Err(err)) => {
                            eprintln!(
                                "error while reading include_tag module_attribute text {}",
                                err
                            );
                            return None;
                        }
                    }
                    .map(|include_module| {
                        // .unwrap_or(working_directory.module.as_str());
                        let mut file = working_directory.path;
                        file.push_str(&include_module);
                        file.push_str("/src/main/webapp");
                        file.push_str(&path[1..path.len() - 1]);
                        if !Path::new(&file).exists() {
                            eprintln!("file {} does not exist", file);
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
                    })?,
                    Err(err) => {
                        eprintln!("error while reading include_tag uri_attribute text {}", err);
                        return None;
                    }
                },
                _ => None,
            },
            kind => {
                eprintln!("string parent is not uri_attribute, its {}", kind);
                return None;
            }
        },
        "interpolated_string" => {
            return None;
        }
        // TODO:
        "java_code" => None,
        "tag_code" => None,
        _ => None,
    };
}

fn complete(params: CompletionParams) -> Option<Vec<CompletionItem>> {
    let text_params = params.text_document_position;
    let text = parser::get_text_document(&text_params).ok()?;
    let mut parser = Parser::new();
    match parser.set_language(tree_sitter_spml::language()) {
        Err(err) => {
            eprintln!("failed to set tree sitter language to spml: {}", err);
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
    // let node = root_node.descendant_for_point_range(trigger_point, trigger_point)?;
    let mut cursor = root_node.walk();
    let mut node;
    let mut previous;
    loop {
        node = cursor.node();
        if node.end_position() < trigger_point {
            previous = Some(node);
            if !cursor.goto_next_sibling() || cursor.node().start_position() > trigger_point {
                node = node.parent().unwrap();
                break;
            }
        } else if !cursor.goto_first_child() {
            previous = node.prev_sibling();
            break;
        }
    }
    if let Some(prev) = previous {
        if prev.kind() == "ERROR" {
            previous = prev.child(prev.child_count() - 1);
        }
    }
    eprintln!(
        "previous: {previous:?}, closest: {node:?} ({})",
        node.utf8_text(text.as_bytes()).unwrap()
    );
    return match node.kind() {
        // "element" | "<" => Some(
        //     symbols::SpTag::iter()
        //         .map(|tag| tag.properties())
        //         .map(|properties| {
        //             let mut insert_text = "<".to_owned();
        //             insert_text.push_str(&properties.name);
        //             return CompletionItem {
        //                 detail: properties.detail,
        //                 documentation: properties.documentation,
        //                 insert_text: Some(insert_text),
        //                 label_details: Some(CompletionItemLabelDetails {
        //                     detail: Some("label_details.details".to_string()),
        //                     description: Some("label_details.description".to_string()),
        //                 }),
        //                 ..Default::default()
        //             };
        //         })
        //         .collect(),
        // ),
        // "tag_name" =>
        // "text" => {
        //     return match parent.kind() {
        "text" | "document" => Some(
            symbols::SpTag::iter()
                .map(|tag| tag.properties())
                .map(|properties| CompletionItem {
                    kind: Some(CompletionItemKind::METHOD),
                    detail: properties.detail,
                    documentation: properties.documentation,
                    insert_text: Some(properties.name),
                    ..Default::default()
                })
                .collect(),
        ),
        // _ => None
        // };
        // }
        "include_tag" => match previous.unwrap().kind() {
            ">" | "argument_tag" => Some(vec![CompletionItem {
                kind: Some(CompletionItemKind::METHOD),
                detail: symbols::SpTag::Argument.properties().detail,
                documentation: symbols::SpTag::Argument.properties().documentation,
                insert_text: Some(symbols::SpTag::Argument.properties().name),
                ..Default::default()
            }]),
            "argument_tag_open" => Some(vec![CompletionItem {
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(String::from("Attribute(String)")),
                documentation: Some(Documentation::String(String::from(
                    "the name of the argument",
                ))),
                insert_text: Some(String::from("name=\"")),
                ..Default::default()
            }]),
            "name_attribute" => {
                let previous_previous = previous?.prev_sibling();
                eprintln!("previous_previous: {previous_previous:?}");
                return match previous_previous?.kind() {
                    "argument_tag_open" => Some(vec![CompletionItem {
                        kind: Some(CompletionItemKind::FIELD),
                        detail: Some(String::from("Attribute(Object)")),
                        documentation: Some(Documentation::String(String::from(
                            "the interpreted value of the argument",
                        ))),
                        insert_text: Some(String::from("object=\"")),
                        ..Default::default()
                    }]),
                    _ => None,
                };
            }
            _ => None,
        },
        "string" => match previous?.kind() {
            "name=" => match previous?.parent()?.prev_sibling()?.kind() {
                "argument_tag_open" => Some(vec![
                    CompletionItem {
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(String::from("Argument(ID)")),
                        documentation: Some(Documentation::String(String::from(
                            "the itemScope to do something for",
                        ))),
                        insert_text: Some(String::from("itemScope\"")),
                        ..Default::default()
                    },
                    CompletionItem {
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(String::from("Argument(Map)")),
                        documentation: Some(Documentation::String(String::from(
                            "options to configure the process of doing something",
                        ))),
                        insert_text: Some(String::from("options\"")),
                        ..Default::default()
                    },
                ]),
                _ => None,
            },
            _ => None,
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
        _ => None,
    };
}
