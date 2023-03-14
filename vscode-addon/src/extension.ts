import * as vscode from "vscode";
import {ExtensionContext, window, workspace} from "vscode";
import {
    Executable,
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
} from "vscode-languageclient/node";
import {EnforcerDependenciesProvider} from "./deps";
import {FinishOperation, SeedwingReport, StartOperation, UpdatedDependencies, UpdateOperation} from "./data";
import {Report} from "./report";
import {acquire} from "./cli";
import {startOperation, updateOperation, finishOperation} from "./progress";

/*
import { ServiceConnection } from '@vscode/sync-api-common/node';
import {ApiService, ApiServiceConnection, Requests, ServicePseudoTerminal} from '@vscode/sync-api-service';
import * as path from 'path';
import { Worker } from 'worker_threads';
 */

let client: LanguageClient;

async function serverOptionsNative(context: ExtensionContext): Promise<ServerOptions> {
    const command = (await acquire(context)).path;

    console.log("Using CLI:", command);

    const run: Executable = {
        command,
        args: ["lsp", "--"],
        transport: TransportKind.stdio,
        options: {
            env: {
                ...process.env,
                "RUST_BACKTRACE": "1",
            },
        },
    };

    const debug: Executable = {
        ...run
    };
    debug.options.env["RUST_LOG"] = "debug";

    console.debug("Run:", run);
    console.debug("Debug:", debug);

    return {
        run,
        debug,
    };
}

async function startLsp(context: ExtensionContext): Promise<LanguageClient> {
    const outputChannel = window.createOutputChannel("Seedwing Enforcer Language Server");
    const traceOutputChannel = window.createOutputChannel("Seedwing Enforcer Language Server Trace");

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            {scheme: "file", pattern: "**/.enforcer.yaml"},
            {scheme: "file", pattern: "**/pom.xml"}
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
        outputChannel,
        outputChannelName: "Seedwing Enforcer",
        traceOutputChannel,
    };

    const serverOptions = await serverOptionsNative(context);
    //const serverOptions = serverOptionsWasi(context);

    // Create the language client and start the client.
    return new LanguageClient(
        "seedwing-enforcer-lsp",
        "Seedwing Enforcer",
        serverOptions,
        clientOptions
    );
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
        console.debug("Params:", params);
        dependencies.update(params);
    });

    client.onNotification(StartOperation.NAME, (params: StartOperation) => {
        startOperation(params.token, params.title, params.total);
    });
    client.onNotification(UpdateOperation.NAME, (params: UpdateOperation) => {
        updateOperation(params.token, params.message, params.increment);
    });
    client.onNotification(FinishOperation.NAME, (params: FinishOperation) => {
        finishOperation(params.token);
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

