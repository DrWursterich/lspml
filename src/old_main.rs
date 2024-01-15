use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tower_lsp::LanguageServer;
use tower_lsp::{LspService, Server};

#[derive(Debug)]
pub struct Backend {
    pub(crate) client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        return Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["custom.notification".to_string()],
                    work_done_progress_options: Default::default(),
                }),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        });
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        return Ok(());
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        if params.command == "custom.notification" {
            self.client
                .send_notification::<CustomNotification>(CustomNotificationParams::new(
                    "Hello", "Message",
                ))
                .await;
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Command executed with params: {params:?}"),
                )
                .await;
            return Ok(None);
        }
        return Err(Error::invalid_request());
    }
}

// NOTIFICATION SHIT

#[derive(Debug, Deserialize, Serialize)]
struct CustomNotificationParams {
    title: String,
    message: String,
}

impl CustomNotificationParams {
    fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        return CustomNotificationParams {
            title: title.into(),
            message: message.into(),
        };
    }
}

enum CustomNotification {}

impl Notification for CustomNotification {
    type Params = CustomNotificationParams;

    const METHOD: &'static str = "custom/notification";
}

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
    return Ok(());
}

#[cfg(test)]
mod tests {
    use crate::Backend;
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;
    use tower_lsp::LspService;
    use tower_lsp::Server;

    #[test]
    fn test_custom_notification() {
        // prints funktionieren nicht!
        println!("testing custom notification");
        tokio_test::block_on(trigger_custom_notification());
    }

    async fn trigger_custom_notification() {
        let initialize = r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{"textDocumentSync":1}},"id":1}"#;

        let did_open = r#"{
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                  "textDocument": {
                    "uri": "file:///foo.rs",
                    "languageId": "rust",
                    "version": 1,
                    "text": "this is a\ntest fo typos\n"
                  }
                }
              }
              "#;

        let (mut req_client, mut resp_client) = start_server();
        let mut buf = vec![0; 1024];

        println!("requesting initilize...");
        req_client
            .write_all(req(initialize).as_bytes())
            .await
            .unwrap();
        println!("reading initilize...");
        let _ = resp_client.read(&mut buf).await.unwrap();

        println!("request {}", did_open);
        req_client
            .write_all(req(did_open).as_bytes())
            .await
            .unwrap();
        println!("reading request...");
        let n = resp_client.read(&mut buf).await.unwrap();

        let result = body(&buf[..n]).unwrap();
        println!("result {}", result);
        assert_eq!(
            result,
            r#"{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"diagnostics":[{"message":"`fo` should be `of`, `for`","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}},"severity":2,"source":"typos-lsp"}],"uri":"file:///foo.rs","version":1}}"#,
        )
    }

    fn start_server() -> (tokio::io::DuplexStream, tokio::io::DuplexStream) {
        let (req_client, req_server) = tokio::io::duplex(1024);
        let (resp_server, resp_client) = tokio::io::duplex(1024);

        let (service, socket) = LspService::new(|client| Backend { client });

        // start server as concurrent task
        tokio::spawn(Server::new(req_server, resp_server, socket).serve(service));

        return (req_client, resp_client);
    }

    fn req(msg: &str) -> String {
        return format!("Content-Length: {}\r\n\r\n{}", msg.len(), msg);
    }

    fn body(src: &[u8]) -> anyhow::Result<&str, anyhow::Error> {
        // parse headers to get headers length
        let mut dst = [httparse::EMPTY_HEADER; 2];

        let (headers_len, _) = match httparse::parse_headers(src, &mut dst)? {
            httparse::Status::Complete(output) => output,
            httparse::Status::Partial => return Err(anyhow::anyhow!("partial headers")),
        };

        // skip headers
        let skipped = &src[headers_len..];

        // return the rest (ie: the body) as &str
        std::str::from_utf8(skipped).map_err(anyhow::Error::from)
    }
}
