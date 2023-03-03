/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import * as fs from 'fs';
import * as path from 'path';
import { parentPort } from 'worker_threads';

import { ClientConnection } from '@vscode/sync-api-common/node';
import { ApiClient, ApiClientConnection, Requests } from '@vscode/sync-api-client';
import { WASI, DeviceDescription } from '@vscode/wasm-wasi/node';

if (parentPort === null) {
    process.exit();
}

// A special connection that allows the worker to talk to the
// extension host API using sync API.
const apiClient = new ApiClient(new ClientConnection<Requests, ApiClientConnection.ReadyParams>(parentPort));
apiClient.serviceReady().then(async (params) => {
    console.log("Starting WASI service");

    // A client that provides sync VS Code API
    const exitHandler = (rval: number): void => {
        apiClient.process.procExit(rval);
    };
    const workspaceFolders = apiClient.vscode.workspace.workspaceFolders;
    const devices: DeviceDescription[] = [];
    if (workspaceFolders.length === 1) {
        devices.push({ kind: 'fileSystem', uri: workspaceFolders[0].uri, mountPoint: path.posix.join(path.posix.sep, 'workspace') });
    } else {
        for (const folder of workspaceFolders) {
            devices.push({ kind: 'fileSystem', uri: folder.uri, mountPoint: path.posix.join(path.posix.sep, 'workspaces', folder.name) });
        }
    }
    // The WASI implementation
    const wasi = WASI.create("Seedwing LSP server", apiClient, exitHandler, devices, params.stdio);
    // The file contain the web assembly code
    // const wasmFile = path.join(__dirname, 'hello.wasm');
    const wasmFile = path.join(__dirname, '..', '..', 'seedwing-enforcer-lsp-wasi', 'target', 'wasm32-wasi', 'debug', 'seedwing_enforcer_lsp_wasi.wasm');
    const binary = fs.readFileSync(wasmFile);
    // Create a web assembly instance from the wasm file using the
    // provided WASI implementation.
    console.log("Pre inst");
    const { instance } = await WebAssembly.instantiate(binary, {
        wasi_snapshot_preview1: wasi
    });
    console.log("Pre init");
    wasi.initialize(instance);
    console.log("Post init");
    // Run the web assembly
    // (instance.exports._start as Function)();
    //const config = new ServerConfig();
    console.log("Config", instance.exports.ServerConfig);
    const config = (instance.exports.ServerConfig as Object).constructor();
    (instance.exports.serve as Function)(config);
    apiClient.process.procExit(0);
}).catch(console.error);