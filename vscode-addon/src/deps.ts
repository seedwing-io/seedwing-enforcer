import * as vscode from "vscode";
import * as purl from "packageurl-js";

import {
    Dependency, UpdatedDependencies
} from "./data";
import { MarkdownString, ThemeIcon } from "vscode";

export class EnforcerDependenciesProvider implements vscode.TreeDataProvider<TreeNode> {
    private dependencies: Map<string, Array<Dependency>>;

    private _onDidChangeTreeData: vscode.EventEmitter<TreeNode | undefined | null | void> = new vscode.EventEmitter<TreeNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<TreeNode | undefined | null | void> = this._onDidChangeTreeData.event;

    constructor() {
        this.dependencies = new Map();
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    update(update: UpdatedDependencies): void {
        update.dependencies.sort((a, b) => {
            return a.purl.localeCompare(b.purl);
        });
        this.dependencies.set(update.root.toString(), update.dependencies);
        this.refresh();
    }

    getChildren(element?: TreeNode): TreeNode[] {
        console.debug("Update:", this.dependencies);

        // eslint-disable-next-line no-constant-condition
        if (this.dependencies.size < 1 || true) {
            return this.getChildrenSingle(element);
        } else {
            return this.getChildrenMulti(element);
        }
    }

    getTreeItem(element: TreeNode): vscode.TreeItem {
        return element;
    }

    getChildrenSingle(element?: TreeNode): TreeNode[] {
        if (!element) {
            // root 
            const list = this.dependencies.values().next().value;
            if (list === undefined) {
                return [];
            }
            return [...list].map((entry) => {
                return new DependencyNode(entry, vscode.TreeItemCollapsibleState.None);
            });
        } else {
            return [];
        }
    }

    getChildrenMulti(element?: TreeNode): TreeNode[] {
        if (!element) {
            // root
            return [...this.dependencies.keys()].map((key) => {
                return new ProjectNode(key, vscode.TreeItemCollapsibleState.Expanded);
            });
        } else if (element instanceof ProjectNode) {
            return EnforcerDependenciesProvider.mapDependencies(this.dependencies.get(element.root));
        } else {
            return [];
        }
    }

    static mapDependencies(dependencies: Array<Dependency>): TreeNode[] {
        if (!dependencies) {
            return [];
        }
        return dependencies.map((entry) => {
            return new DependencyNode(entry, vscode.TreeItemCollapsibleState.None);
        });
    }
}

const ICON_LIBRARY = new ThemeIcon("library");
const ICON_PROJECT = new ThemeIcon("project");

type TreeNode = DependencyNode | ProjectNode | undefined;

class ProjectNode extends vscode.TreeItem {
    constructor(
        public readonly root: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
    ) {
        super(root, collapsibleState);
        this.iconPath = ICON_PROJECT;
    }
}

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