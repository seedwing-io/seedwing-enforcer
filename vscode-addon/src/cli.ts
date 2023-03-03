import { ExtensionContext, workspace } from "vscode";
import * as os from "os";
import * as fs from "fs";
import * as vscode from "vscode";

export interface Cli {
    path: string,
}

/// Fetch the CLI (if required) and return its path
export async function acquire(context: ExtensionContext): Promise<Cli> {

    const fromEnv = process.env.SENF_BIN || process.env.SERVER_PATH;
    if (fromEnv !== undefined) {
        return {
            path: fromEnv
        }
    }

    const config = workspace.getConfiguration("seedwing-enforcer", vscode.workspace.workspaceFolders[0].uri);
    const fromConfig = config.get("cli.path");
    console.info(`Enforcer CLI config: ${fromConfig}`);
    if (fromConfig !== undefined && typeof fromConfig == "string" && fromConfig !== "") {
        return {
            path: fromConfig
        }
    }

    let name;

    // FIXME: need to handle non-amd64 targets
    const target = `${os.platform()}-${os.arch()}`;
    switch (target) {
        case "linux-x64":
            name = "senf-linux-amd64";
            break;
        case "darwin-x64":
            name = "senf-macos-amd64";
            break;
        case "darwin-arm64":
            name = "senf-macos-aarch64";
            break;
        case "win32-x64":
            name = "senf-windows-amd64.exe";
            break;
        default:
            throw `Unsupported target: ${target}`;
    }

    const path = context.asAbsolutePath("cli/" + name);

    if (os.platform() !== "win32") {
        fs.chmodSync(path, "0755");
    }

    return { path };
}
