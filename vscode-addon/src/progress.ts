import * as vscode from "vscode";
import {Channel} from "async-channel";

const operations = new Map<string, Progress>();

interface Update {
    message?: string;
    increment?: number;
}

class Progress {
    private readonly _total: number;
    private readonly _channel: Channel<Update>;

    private _current: number;
    private _last: number;

    constructor(title: string, total: number) {
        this._total = total;
        this._current = 0;
        this._last = 0;

        this._channel = new Channel(0);

        vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            cancellable: false,
            title
        }, async (progress) => {

            for await (const msg of this._channel) {

                console.debug("Next message:", msg);

                if (msg.increment !== undefined) {
                    this._current += msg.increment
                }

                const n = (this._current / this._total) * 100.0;
                let increment = 0;
                if (n !== this._last) {
                    increment = n - this._last ;
                    this._last = n;
                }

                progress.report({
                    message: msg.message,
                    increment,
                })
            }

            progress.report({increment: 100});

        })
    }

    update(message?: string, increment?: number) {
        this._channel.push({
            message, increment
        });
    }

    finish() {
        this._channel.close(true);
    }
}

export function startOperation(token: string, title: string, total: number): void {
    console.debug("Start operation:", token);
    operations.set(token, new Progress(title, total));
}

export function finishOperation(token: string): void {
    console.debug("Finish operation:", token);

    const op = operations.get(token);
    operations.delete(token);
    op?.finish();
}

export function updateOperation(token: string, message?: string, increment?: number): void {
    console.debug("Update operation:", token);

    operations.get(token)?.update(message, increment);
}