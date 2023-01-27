//! Workspace: Something that VScode/LSP considers a set of folders
//! Folder: Some "root" folder in a workspace
//! Project: An actual project that we want to work with. Found in the root- or sub-folder.

use crate::backend::workspace::Workspace;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

mod project;
mod workspace;

pub struct Backend {
    pub client: Client,
    workspace: Workspace,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        log::info!("Workspaces: {:?}", params.workspace_folders);

        if let Some(folders) = params.workspace_folders {
            self.workspace.folders_changed(folders, vec![]).await;
        }

        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                        change: Some(TextDocumentSyncKind::NONE),
                        open_close: Some(true),
                        ..Default::default()
                    },
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                // definition: Some(GotoCapability::default()),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, param: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;

        self.workspace
            .folders_changed(param.event.added, param.event.removed)
            .await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        for event in params.changes {
            self.workspace.changed(&event.uri).await;
        }
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        // TODO: we might want to notify the project now
    }
}

impl Backend {
    pub fn new(client: Client) -> Self {
        let workspace = Workspace::new(client.clone());
        Self { client, workspace }
    }
}
