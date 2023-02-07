/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import {
  workspace,
  ExtensionContext,
  window,
} from "vscode";

import * as vscode from "vscode";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

import {
  EnforcerDependenciesProvider
} from "./deps";

import {SeedwingReport, UpdatedDependencies} from "./data";
import { Report } from "./report";

let client: LanguageClient;

export async function activate(context: ExtensionContext): Promise<void> {

  // register report view

  context.subscriptions.push(
      vscode.commands.registerCommand("seedwingEnforcer.showReport", (reports: SeedwingReport[]) => {
          new Report(context.extensionUri, reports);
      })
  );

  // LSP

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

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", pattern: "**/.enforcer.yaml" },
      { scheme: "file", pattern: "**/pom.xml" }
    ],
    synchronize: {
      fileEvents: [
        workspace.createFileSystemWatcher("**/pom.xml"),
        workspace.createFileSystemWatcher("**/.enforcer.yaml"),
        workspace.createFileSystemWatcher("**/*.dog")
      ],
    },
    markdown: {
      isTrusted: true,
      supportHtml: true
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

  // view

  const dependencies = new EnforcerDependenciesProvider();
  vscode.window.registerTreeDataProvider(
    "seedwing-enforcer.dependencies", // aligns with the view id in package.json
    dependencies,
  );

  client.onNotification(UpdatedDependencies.NAME, (params: UpdatedDependencies) => {
    console.log("Params:", params);
    dependencies.update(params);
  });

  client.registerProposedFeatures();

  // start client

  await client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
