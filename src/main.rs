use anyhow::Result;
use clap::Parser;
use lsp_server::{Connection, Message};
use lsp_types::{
    CancelParams, CodeActionKind, CodeActionOptions, CodeActionProviderCapability,
    CompletionOptions, CompletionOptionsCompletionItem, DiagnosticOptions,
    DiagnosticServerCapabilities, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, HoverOptions, HoverProviderCapability,
    InitializeParams, NumberOrString, OneOf, SemanticTokenModifier, SemanticTokenType,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, WorkDoneProgressOptions,
};
use std::{
    error::Error,
    fmt::{Display, Formatter},
    fs::File,
    str::FromStr,
};
use structured_logger::Builder;
mod command;
mod document_store;
mod grammar;
mod modules;
mod parser;
mod spel;

#[derive(Parser, Debug)]
#[clap(name = "lspml")]
struct CommandLineOpts {
    #[clap(long)]
    log_file: Option<String>,
    #[clap(long, default_value = "INFO")]
    log_level: String,
    #[clap(long)]
    modules_file: Option<String>,
}

pub(crate) const TOKEN_TYPES: &'static [SemanticTokenType] = &[
    SemanticTokenType::ENUM,
    SemanticTokenType::ENUM_MEMBER,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::MACRO,
    SemanticTokenType::METHOD,
    SemanticTokenType::NUMBER,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::REGEXP,
    SemanticTokenType::STRING,
    SemanticTokenType::VARIABLE,
];

pub(crate) const TOKEN_MODIFIERS: &'static [SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,
    SemanticTokenModifier::DEFINITION,
    SemanticTokenModifier::DEPRECATED,
    SemanticTokenModifier::DOCUMENTATION,
    SemanticTokenModifier::MODIFICATION,
];

pub(crate) enum CodeActionImplementation {
    GenerateDefaultHeaders,
    NameToCondition,
    ConditionToName,
}

impl CodeActionImplementation {
    pub(crate) const GENERATE_DEFAULT_HEADER_CODE: NumberOrString = NumberOrString::Number(7126);

    fn kinds() -> Vec<CodeActionKind> {
        return vec![
            CodeActionImplementation::GenerateDefaultHeaders.to_kind(),
            CodeActionImplementation::NameToCondition.to_kind(),
            CodeActionImplementation::ConditionToName.to_kind(),
        ];
    }

    fn to_kind(self) -> CodeActionKind {
        return CodeActionKind::new(match self {
            CodeActionImplementation::GenerateDefaultHeaders => "refactor.generate_default_headers",
            CodeActionImplementation::NameToCondition => "refactor.name_to_condition",
            CodeActionImplementation::ConditionToName => "refactor.condition_to_name",
        });
    }
}

impl Display for CodeActionImplementation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            CodeActionImplementation::GenerateDefaultHeaders => "refactor.generate_default_headers",
            CodeActionImplementation::NameToCondition => "refactor.name_to_condition",
            CodeActionImplementation::ConditionToName => "refactor.condition_to_name",
        })
    }
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
    log::trace!("commandline opts: {:?}", opts);
    match opts.modules_file {
        Some(file) => modules::init_module_mappings_from_file(&file),
        None => modules::init_empty_module_mappings(),
    }?;

    let (connection, io_threads) = Connection::stdio();
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        definition_provider: Some(OneOf::Left(true)),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        document_highlight_provider: Some(OneOf::Left(true)),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                full: Some(SemanticTokensFullOptions::Bool(true)),
                legend: SemanticTokensLegend {
                    token_types: TOKEN_TYPES.to_vec(),
                    token_modifiers: TOKEN_MODIFIERS.to_vec(),
                },
                ..Default::default()
            },
        )),
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
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(CodeActionImplementation::kinds()),
            ..CodeActionOptions::default()
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
                    "textDocument/completion" => command::complete(request).map(Some),
                    "textDocument/definition" => command::definition(request).map(Some),
                    "textDocument/diagnostic" => command::diagnostic(request).map(Some),
                    "textDocument/documentHighlight" => command::highlight(request).map(Some), // stub
                    "textDocument/semanticTokens/full" => command::semantics(request).map(Some),
                    "textDocument/codeAction" => command::action(request).map(Some),
                    "textDocument/hover" => command::hover(request),
                    _ => command::unknown(request).map(Some),
                }
                .and_then(|response| match response {
                    Some(response) => connection
                        .sender
                        .send(response)
                        .map_err(|err| anyhow::anyhow!(err)),
                    None => Ok(()),
                })?;
            }
            Message::Response(response) => {
                log::info!("got unknown response: {:?}", response);
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
                "textDocument/didClose" => {
                    closed(serde_json::from_value(notification.params)?)?;
                }
                "$/cancelRequest" => {
                    let params: CancelParams = serde_json::from_value(notification.params).unwrap();
                    log::debug!("attempted to cancel request {:?}", params.id);
                }
                _ => log::info!("got unknown notification: {:?}", notification),
            },
        }
    }

    return Ok(());
}

fn changed(params: DidChangeTextDocumentParams) -> Result<()> {
    if let Some(change) = &params.content_changes.last() {
        return document_store::Document::from_str(&change.text).map(|document| {
            log::debug!("updated {}", params.text_document.uri);
            document_store::put(&params.text_document.uri, document);
            return ();
        });
    }
    return Ok(());
}

fn opened(params: DidOpenTextDocumentParams) -> Result<()> {
    return match document_store::get(&params.text_document.uri) {
        Some(_) => Result::Ok(()),
        None => document_store::Document::from_str(&params.text_document.text).map(|document| {
            document_store::put(&params.text_document.uri, document);
            log::debug!("opened {}", params.text_document.uri);
            return ();
        }),
    };
}

fn saved(params: DidSaveTextDocumentParams) -> Result<()> {
    return document_store::Document::new(&params.text_document.uri).map(|document| {
        document_store::put(&params.text_document.uri, document);
        log::debug!("saved {}", params.text_document.uri);
        return ();
    });
}

fn closed(_: DidCloseTextDocumentParams) -> Result<()> {
    // could free the document... ?
    return Ok(());
}
