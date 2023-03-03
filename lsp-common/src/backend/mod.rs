//! Workspace: Something that VScode/LSP considers a set of folders
//! Folder: Some "root" folder in a workspace
//! Project: An actual project that we want to work with. Found in the root- or sub-folder.

use crate::backend::workspace::Workspace;
use std::{cell::Cell, sync::Mutex};
use tower_lsp::{
    jsonrpc::{Error, ErrorCode, Result},
    lsp_types::*,
    Client, LanguageServer,
};

mod notification;
mod project;
mod workspace;

pub struct Backend {
    pub client: Client,
    workspace: Workspace,
    initial_folders: Mutex<Cell<Option<Vec<WorkspaceFolder>>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        log::info!("Workspaces: {:?}", params.workspace_folders);

        // remember how we got initialized, unfortunately we cannot set it up yet, as we can't
        // send notification before the `initialized` function got called.

        match (params.root_uri, params.workspace_folders) {
            (_, Some(workspace_folders)) => {
                self.initial_folders
                    .lock()
                    .unwrap()
                    .set(Some(workspace_folders));
            }
            (Some(uri), _) => {
                let name = uri.to_string();
                self.initial_folders
                    .lock()
                    .unwrap()
                    .set(Some(vec![WorkspaceFolder { uri, name }]));
            }
            _ => {}
        }

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Seedwing Enforcer".to_string(),
                ..Default::default()
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                        change: Some(TextDocumentSyncKind::NONE),
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
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),

                // definition: Some(GotoCapability::default()),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        let folders = self.initial_folders.lock().unwrap().replace(None);
        if let Some(folders) = folders {
            self.workspace.folders_changed(folders, vec![]).await;
        }
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

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        log::info!("Code action: {params:?}");

        let result = self
            .workspace
            .code_action(&params.text_document.uri, &params.range, &params.context)
            .await
            .map_err(|err| Error {
                code: ErrorCode::InternalError,
                message: err.to_string(),
                data: None,
            })?;

        Ok(if result.is_empty() {
            None
        } else {
            Some(result)
        })
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        log::info!("Code lens: {params:?}");

        let result = self
            .workspace
            .code_lens(&params.text_document.uri)
            .await
            .map_err(|err| Error {
                code: ErrorCode::InternalError,
                message: err.to_string(),
                data: None,
            })?;

        Ok(if result.is_empty() {
            None
        } else {
            Some(result)
        })
    }
}

impl Backend {
    pub fn new(client: Client) -> Self {
        let workspace = Workspace::new(client.clone());
        Self {
            client,
            workspace,
            initial_folders: Default::default(),
        }
    }
}
