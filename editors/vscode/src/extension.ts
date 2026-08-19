import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import { INSTALL_DIR, downloadServer } from "./download";

let client: LanguageClient | undefined;
let output: vscode.OutputChannel;

export async function activate(context: vscode.ExtensionContext) {
  output = vscode.window.createOutputChannel("Pinecone");
  context.subscriptions.push(output);

  const server = resolveServer();
  if (server) {
    start(server, context);
  } else {
    await promptDownload(context);
  }
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}

function resolveServer(): string | undefined {
  if (process.env.SERVER_PATH) {
    return process.env.SERVER_PATH;
  }
  const configured =
    vscode.workspace.getConfiguration("pinecone").get<string>("server.path") ||
    "pinecone";
  const onPath = which(configured);
  if (onPath) {
    return onPath;
  }
  const downloaded = path.join(INSTALL_DIR, "pinecone");
  return fs.existsSync(downloaded) ? downloaded : undefined;
}

function which(cmd: string): string | undefined {
  if (cmd.includes(path.sep)) {
    return fs.existsSync(cmd) ? cmd : undefined;
  }
  for (const dir of (process.env.PATH || "").split(path.delimiter)) {
    const candidate = path.join(dir, cmd);
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }
  return undefined;
}

// --- starting the client -------------------------------------------------

function start(command: string, context: vscode.ExtensionContext) {
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
  context.subscriptions.push({ dispose: () => void client?.stop() });
}

async function promptDownload(context: vscode.ExtensionContext) {
  const choice = await vscode.window.showInformationMessage(
    "The pinecone language server was not found. Download it?",
    "Download",
    "Open settings"
  );
  if (choice === "Open settings") {
    vscode.commands.executeCommand(
      "workbench.action.openSettings",
      "pinecone.server.path"
    );
    return;
  }
  if (choice !== "Download") {
    return;
  }
  try {
    const command = await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: "Downloading pinecone language server",
      },
      () => downloadServer(output)
    );
    start(command, context);
    vscode.window.showInformationMessage("pinecone language server installed.");
  } catch (err) {
    output.appendLine(`Download failed: ${err}`);
    vscode.window.showErrorMessage(
      `Failed to download the pinecone server: ${err}`
    );
  }
}