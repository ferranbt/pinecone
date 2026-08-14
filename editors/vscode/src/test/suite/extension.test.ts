import * as assert from "assert";
import * as path from "path";
import * as vscode from "vscode";

function fixture(name: string): vscode.Uri {
  return vscode.Uri.file(path.resolve(__dirname, "../../../testFixture", name));
}

// Resolve once the server has published diagnostics for `uri` — i.e. it has
// analyzed the document. Event-driven, so tests wait on the real signal rather
// than a fixed delay.
function analyzed(uri: vscode.Uri): Promise<void> {
  return new Promise((resolve) => {
    const subscription = vscode.languages.onDidChangeDiagnostics((event) => {
      if (event.uris.some((u) => u.toString() === uri.toString())) {
        subscription.dispose();
        resolve();
      }
    });
  });
}

async function open(uri: vscode.Uri): Promise<vscode.TextDocument> {
  // An already-open document was analyzed on its first open and won't re-publish
  // diagnostics, so only wait for the analysis when opening it afresh.
  const alreadyOpen = vscode.workspace.textDocuments.some(
    (d) => d.uri.toString() === uri.toString()
  );
  const ready = alreadyOpen ? Promise.resolve() : analyzed(uri);
  const doc = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(doc);
  await ready;
  return doc;
}

suite("pinecone language server", () => {
  suiteSetup(async () => {
    await vscode.extensions.getExtension("pinecone.pinecone")?.activate();
  });

  test("reports a repaint warning", async () => {
    const uri = fixture("repaint.pine");
    await open(uri);

    const repaint = vscode.languages
      .getDiagnostics(uri)
      .find((d) => d.code === "security-repaint");
    assert.ok(repaint, "expected a security-repaint diagnostic");
    assert.strictEqual(repaint!.severity, vscode.DiagnosticSeverity.Warning);
  });

  test("formats a document", async () => {
    const uri = fixture("unformatted.pine");
    const doc = await open(uri);

    const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>(
      "vscode.executeFormatDocumentProvider",
      uri,
      { tabSize: 4, insertSpaces: true }
    );
    assert.ok(edits && edits.length > 0, "expected format edits");

    const workspaceEdit = new vscode.WorkspaceEdit();
    workspaceEdit.set(uri, edits);
    await vscode.workspace.applyEdit(workspaceEdit);
    assert.strictEqual(doc.getText(), "x = 1 + 2\n");
  });

  test("hovers a user function", async () => {
    const uri = fixture("symbols.pine");
    await open(uri);

    // `double` in its call on line 4 (0-based line 3, char 6).
    const hovers = await vscode.commands.executeCommand<vscode.Hover[]>(
      "vscode.executeHoverProvider",
      uri,
      new vscode.Position(3, 6)
    );
    assert.ok(hovers && hovers.length > 0, "expected a hover");
    const content = hovers[0].contents[0] as vscode.MarkdownString;
    assert.ok(content.value.includes("double(x)"), content.value);
  });

  test("goes to a definition", async () => {
    const uri = fixture("symbols.pine");
    await open(uri);

    const locations = await vscode.commands.executeCommand<vscode.Location[]>(
      "vscode.executeDefinitionProvider",
      uri,
      new vscode.Position(3, 6)
    );
    assert.ok(locations && locations.length > 0, "expected a definition");
    assert.strictEqual(locations[0].range.start.line, 2); // `double(x) =>`
  });
});
