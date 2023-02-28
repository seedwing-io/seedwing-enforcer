/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as vscode from "vscode";
import { ExtensionContext, window, workspace, } from "vscode";
import {
    Executable,
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
} from "vscode-languageclient/node";
import { EnforcerDependenciesProvider } from "./deps";
import { SeedwingReport, UpdatedDependencies } from "./data";
import { Report } from "./report";

/*
import { ServiceConnection } from '@vscode/sync-api-common/node';
import {ApiService, ApiServiceConnection, Requests, ServicePseudoTerminal} from '@vscode/sync-api-service';
import * as path from 'path';
import { Worker } from 'worker_threads';
 */

let client: LanguageClient;

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function serverOptionsNative(context: ExtensionContext): ServerOptions {
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
    return {
        run,
        debug: run,
    };
}

async function startLsp(context: ExtensionContext): Promise<LanguageClient> {
    const traceOutputChannel = window.createOutputChannel("Seedwing Enforcer Language Server trace");

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

    const serverOptions = serverOptionsNative(context);
    //const serverOptions = serverOptionsWasi(context);

    // Create the language client and start the client.
    client = new LanguageClient(
        "seedwing-enforcer-lsp",
        "Seedwing Enforcer",
        serverOptions,
        clientOptions
    );

    return client;
}

export async function activate(context: ExtensionContext): Promise<void> {

    // register report view

    context.subscriptions.push(
        vscode.commands.registerCommand("seedwingEnforcer.showReport", (reports: SeedwingReport[]) => {
            new Report(context.extensionUri, reports);
        })
    );

    // LSP

    client = await startLsp(context);

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
