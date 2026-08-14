import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(_context: vscode.ExtensionContext) {
  const configured =
    vscode.workspace.getConfiguration("pinecone").get<string>("server.path") ||
    "pinecone";
  // SERVER_PATH lets the integration tests point at the freshly built binary.
  const command = process.env.SERVER_PATH || configured;

  const serverOptions: ServerOptions = {
    command,
    args: ["lsp"],
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "pine" }],
  };

  client = new LanguageClient(
    "pinecone",
    "Pine Script",
    serverOptions,
    clientOptions
  );
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
