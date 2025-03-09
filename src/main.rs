#![feature(fn_traits)]

use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::Parser;
use command::diagnostic::{self, Diagnostic};
use lsp_server::{Connection, Message};
use lsp_types::{
    CancelParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializeParams,
};
use structured_logger::Builder;

mod analyze;
mod capabilities;
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
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    Analyze {
        #[clap(long)]
        directory: String,
        #[clap(long, default_value_t = analyze::Format::TEXT)]
        format: analyze::Format,
        #[clap(long)]
        ignore: Option<Vec<diagnostic::Type>>,
    },
}

struct NullLogWriter;

impl structured_logger::Writer for NullLogWriter {
    fn write_log(
        &self,
        _value: &BTreeMap<log::kv::Key, log::kv::Value>,
    ) -> std::result::Result<(), std::io::Error> {
        return Ok(());
    }
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let opts = CommandLineOpts::parse();

    Builder::with_level(&opts.log_level)
        .with_target_writer(
            "*",
            opts.log_file
                .clone()
                .and_then(|file| {
                    fs::File::options()
                        .create(true)
                        .append(true)
                        .open(file)
                        .ok()
                })
                .map(|file| structured_logger::json::new_writer(file))
                .unwrap_or_else(|| match opts.command {
                    Some(_) => Box::new(NullLogWriter),
                    None => structured_logger::json::new_writer(std::io::stderr()),
                }),
        )
        .init();
    log::info!("lspml starting...");
    log::debug!("commandline opts: {:?}", opts);
    match opts.modules_file {
        Some(file) => modules::init_module_mappings_from_file(&file),
        None => modules::init_empty_module_mappings(),
    }?;

    match opts.command {
        Some(Commands::Analyze {
            directory,
            format,
            ignore,
        }) => {
            let mut diagnostics: HashMap<PathBuf, Vec<command::diagnostic::Diagnostic>> =
                HashMap::new();
            let path = Path::new(&directory);
            if !path.is_dir() {
                return Err(anyhow::anyhow!("{} is not a directory", directory).into());
            }
            command::diagnostic::diagnose_all(&path, &mut diagnostics)?;
            if let Some(ignore) = ignore {
                diagnostics = diagnostics
                    .into_iter()
                    .filter_map(|(key, values)| {
                        let values = values
                            .into_iter()
                            .filter(|d| !ignore.contains(&d.r#type))
                            .collect::<Vec<Diagnostic>>();
                        return match values.is_empty() {
                            true => None,
                            false => Some((key, values)),
                        };
                    })
                    .collect();
            }
            return Ok(analyze::print(diagnostics, format)?);
        }
        None => (),
    }

    let (connection, io_threads) = Connection::stdio();
    let server_capabilities = serde_json::to_value(capabilities::create())?;
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
                    break;
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
    let uri = params.text_document.uri;
    return match &params.content_changes.last() {
        Some(change) => document_store::Document::new(change.text.to_owned()).map(|document| {
            document_store::put(&uri, document);
            log::debug!("updated {:?}", uri);
        }),
        None => Ok(()),
    };
}

fn opened(params: DidOpenTextDocumentParams) -> Result<()> {
    let uri = params.text_document.uri;
    return match document_store::get(&uri) {
        Some(_) => Ok(()),
        None => document_store::Document::new(params.text_document.text).map(|document| {
            document_store::put(&uri, document);
            log::debug!("opened {:?}", uri);
            return ();
        }),
    };
}

fn saved(params: DidSaveTextDocumentParams) -> Result<()> {
    let uri = params.text_document.uri;
    return document_store::Document::from_uri(&uri).map(|document| {
        document_store::put(&uri, document);
        log::debug!("saved {:?}", uri);
    });
}

fn closed(_: DidCloseTextDocumentParams) -> Result<()> {
    // document_store::free(&uri); ?
    return Ok(());
}
