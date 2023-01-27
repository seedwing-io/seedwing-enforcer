/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import {
  workspace,
  ExtensionContext,
  window,
} from "vscode";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

export async function activate(context: ExtensionContext) {
  const traceOutputChannel = window.createOutputChannel("Seedwing Enforcer Language Server trace");
  const command = process.env.SERVER_PATH || "seedwing-enforcer-lsp";
  const run: Executable = {
    command,
    options: {
      env: {
        ...process.env,
        // eslint-disable-next-line @typescript-eslint/naming-convention
        RUST_LOG: "debug",
      },
    },
  };
  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };
  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", language: "seedwing-pom" },
      { scheme: "file", language: "enforcer-config" },
      { pattern: "pom.xml" }
    ],
    synchronize: {
      fileEvents: [
        workspace.createFileSystemWatcher("**/pom.xml"),
        workspace.createFileSystemWatcher("**/.enforcer.yaml"),
        workspace.createFileSystemWatcher("**/*.dog")
      ],
    },
    traceOutputChannel,
  };

  // Create the language client and start the client.
  client = new LanguageClient(
    "seedwing-enforcer-lsp",
    "Seedwing Enforcer",
    serverOptions,
    clientOptions
  );
  await client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
