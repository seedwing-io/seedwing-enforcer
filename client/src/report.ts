import * as vscode from "vscode";
import { SeedwingReport } from "./data";

export class Report {

  private static readonly VIEW_ID = "seedwingEnforcer.reportView";

  private readonly panel: vscode.WebviewPanel;

  public constructor(extensionUri: vscode.Uri, reports: SeedwingReport[]) {

    this.panel = vscode.window.createWebviewPanel(
      Report.VIEW_ID,
      "Seedwing Report",
      vscode.ViewColumn.Beside,
      {
        enableScripts: true,
        localResourceRoots: [vscode.Uri.joinPath(extensionUri, "assets")]
      }
    );

    const stylesPath = vscode.Uri.joinPath(extensionUri, "assets", "styles.css");
    const stylesUri = this.panel.webview.asWebviewUri(stylesPath);

    let inner = "";
    for (const r of reports) {
      inner += `
<section>
  <h2>${r.title}</h2>
  <div class="sw-rationale">${r.html}</div>
</section>
`;
    }

    this.panel.webview.html = `
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Seedwing Enforcer Report</title>
    <link rel="stylesheet" href="${stylesUri}">
  </head>
  <body>
    <header>
      <h1>Seedwing Enforcer Report</h1>
    </header>
    <main>
    ${inner}
    </main>
  </body>
</html>
`;
  }
}