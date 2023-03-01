import {ExtensionContext} from "vscode";
import * as os from "os";
import * as fs from "fs";

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

    let name;

    // FIXME: need to handle non-amd64 targets
    const target = `${os.platform()}-${os.arch()}`;
    switch (target) {
        case "win32-x64":
            name = "senf-windows-amd64.exe";
            break;
        case "darwin-x64":
        case "darwin-arm64":
            name = "senf-macos-amd64";
            break;
        case "linux-x64":
            name = "senf-linux-amd64";
            break;
        default:
            throw `Unsupported target: ${target}`;
    }

    const path = context.asAbsolutePath("cli/" + name);

    fs.chmodSync(path, "0755");

    return { path };
}
