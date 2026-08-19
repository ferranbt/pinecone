import * as assert from "assert";
import * as path from "path";
import * as vscode from "vscode";

function fixture(name: string): vscode.Uri {
  return vscode.Uri.file(path.resolve(__dirname, "../../../testFixture", name));
}

function labelOf(item: vscode.CompletionItem): string {
  return typeof item.label === "string" ? item.label : item.label.label;
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
    await vscode.extensions.getExtension("ferranborreguero.pinecone")?.activate();
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
    // The signature is followed by the declaration and every call site.
    assert.ok(content.value.includes("Defined at"), content.value);
    assert.ok(content.value.includes("1 call"), content.value);
    assert.ok(content.value.includes("4:5"), content.value); // the call on line 4
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

  test("finds references to a function", async () => {
    const uri = fixture("symbols.pine");
    await open(uri);

    const locations = await vscode.commands.executeCommand<vscode.Location[]>(
      "vscode.executeReferenceProvider",
      uri,
      new vscode.Position(3, 6) // the `double` call on line 4
    );
    assert.ok(locations && locations.length > 0, "expected references");
    const lines = locations.map((l) => l.range.start.line).sort();
    // The declaration on line 3 (0-based 2) and the call on line 4 (0-based 3).
    assert.deepStrictEqual(lines, [2, 3], JSON.stringify(lines));
  });

  test("completes an object's fields", async () => {
    const uri = fixture("completion.pine");
    await open(uri);

    // Just after `p.` on line 7 (`v = p.x`).
    const list = await vscode.commands.executeCommand<vscode.CompletionList>(
      "vscode.executeCompletionItemProvider",
      uri,
      new vscode.Position(6, 6)
    );
    const labels = list.items.map(labelOf);
    assert.ok(
      labels.includes("x") && labels.includes("y"),
      labels.join(", ")
    );
  });

  test("completes a builtin namespace", async () => {
    const uri = fixture("completion.pine");
    await open(uri);

    // Just after `ta.` on line 8 (`w = ta.sma(close, 5)`).
    const list = await vscode.commands.executeCommand<vscode.CompletionList>(
      "vscode.executeCompletionItemProvider",
      uri,
      new vscode.Position(7, 7)
    );
    const labels = list.items.map(labelOf);
    assert.ok(labels.includes("sma"), labels.slice(0, 20).join(", "));
  });

  test("builtin members carry kind and signature", async () => {
    const uri = fixture("completion.pine");
    await open(uri);

    // `ta.sma` is a function whose parameters are shown as the detail.
    const ta = await vscode.commands.executeCommand<vscode.CompletionList>(
      "vscode.executeCompletionItemProvider",
      uri,
      new vscode.Position(7, 7) // after `ta.`
    );
    const sma = ta.items.find((i) => labelOf(i) === "sma");
    assert.ok(sma, "expected ta.sma");
    assert.strictEqual(sma!.kind, vscode.CompletionItemKind.Function);
    assert.strictEqual(sma!.detail, "sma(source, length)");

    // `math.pi` is a constant, not a function.
    const math = await vscode.commands.executeCommand<vscode.CompletionList>(
      "vscode.executeCompletionItemProvider",
      uri,
      new vscode.Position(8, 9) // after `math.`
    );
    const pi = math.items.find((i) => labelOf(i) === "pi");
    assert.ok(pi, "expected math.pi");
    assert.strictEqual(pi!.kind, vscode.CompletionItemKind.Constant);
  });
});
