import {ExtensionContext} from "vscode";

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

    // TODO: allow configuring this

    return {
        path: context.asAbsolutePath("cli/senf")
    }
}
