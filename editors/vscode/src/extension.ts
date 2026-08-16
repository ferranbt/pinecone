import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
  const output = vscode.window.createOutputChannel("Pinecone");
  context.subscriptions.push(output);

  const configured =
    vscode.workspace.getConfiguration("pinecone").get<string>("server.path") ||
    "pinecone";
  // SERVER_PATH lets the integration tests point at the freshly built binary.
  const command = process.env.SERVER_PATH || configured;
  output.appendLine(`Starting language server: ${command} lsp`);

  const serverOptions: ServerOptions = {
    command,
    args: ["lsp"],
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "pine" }],
    // Server `window/logMessage`s (e.g. "pine-lsp ready") land here too.
    outputChannel: output,
  };

  client = new LanguageClient(
    "pinecone",
    "Pine Script",
    serverOptions,
    clientOptions
  );
  client
    .start()
    .catch((err) =>
      output.appendLine(`Language server failed to start: ${err}`)
    );
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
