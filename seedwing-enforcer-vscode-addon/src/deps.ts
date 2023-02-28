import * as vscode from "vscode";
import * as purl from "packageurl-js";

import {
    Dependency, UpdatedDependencies
} from "./data";
import { MarkdownString, ThemeIcon } from "vscode";

export class EnforcerDependenciesProvider implements vscode.TreeDataProvider<DependencyNode> {
    private dependencies: Array<Dependency>;

    private _onDidChangeTreeData: vscode.EventEmitter<DependencyNode | undefined | null | void> = new vscode.EventEmitter<DependencyNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<DependencyNode | undefined | null | void> = this._onDidChangeTreeData.event;

    constructor() {
        this.dependencies = [];
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    update(update: UpdatedDependencies): void {
        // FIXME: handle root information
        this.dependencies = update.dependencies;
        this.dependencies.sort((a, b) => {
            return a.purl.localeCompare(b.purl);
        })
        this.refresh();
    }

    getChildren(element?: DependencyNode): DependencyNode[] {
        console.debug("Update:", this.dependencies);

        if (!element) {
            // root
            return this.dependencies.map((entry) => {
                return new DependencyNode(entry, vscode.TreeItemCollapsibleState.None);
            });
        } else {
            // child
            return [];
        }
    }

    getTreeItem(element: DependencyNode): vscode.TreeItem {
        return element;
    }
}

const ICON_LIBRARY = new ThemeIcon("library");

class DependencyNode extends vscode.TreeItem {
    constructor(
        public readonly dependency: Dependency,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState
    ) {
        super(dependency.purl, collapsibleState);
        this.iconPath = ICON_LIBRARY;

        this.makeLabels();
    }

    /// Create the labels, descriptions, …
    makeLabels() {
        try {
            const pkg = purl.PackageURL.fromString(this.dependency.purl);
            this.label = `${pkg.namespace}/${pkg.name}@${pkg.version} (${pkg.type})`;
            let docs = `$(library) ${pkg.namespace}/**${pkg.name}**@${pkg.version} *(${pkg.type})*\n`;

            for (const [k, v] of Object.entries(pkg.qualifiers)) {
                docs += `\n* ${k}: ${v}`;
            }

            docs += `\n\nRaw: \`${this.dependency.purl}\``;

            this.tooltip = new MarkdownString(docs, true);
        } catch (ex) {
            // fall back to showing the raw string
            this.label = this.dependency.purl;
            this.description = undefined;
        }
    }

}