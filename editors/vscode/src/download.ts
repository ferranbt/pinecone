import * as vscode from "vscode";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import * as https from "https";
import { IncomingMessage } from "http";
import { execFile } from "child_process";
import { promisify } from "util";

const execFileAsync = promisify(execFile);
const REPO = "ferranbt/pinecone";

/// Where this extension keeps the language server it downloads.
export const INSTALL_DIR = path.join(os.homedir(), ".pinecone-lsp");

/// Download the latest release's `pinecone` binary into `INSTALL_DIR` and
/// return its path.
export async function downloadServer(
  output: vscode.OutputChannel
): Promise<string> {
  const target = platformTarget();
  const tag = await latestTag();
  const archive = `pinecone-${tag}-${target}.tar.gz`;
  const url = `https://github.com/${REPO}/releases/download/${tag}/${archive}`;

  await fs.promises.mkdir(INSTALL_DIR, { recursive: true });
  const tarball = path.join(INSTALL_DIR, archive);
  output.appendLine(`Downloading ${url}`);
  await fetchToFile(url, tarball);
  await execFileAsync("tar", ["-xzf", tarball, "-C", INSTALL_DIR]);
  await fs.promises.unlink(tarball);

  const bin = path.join(INSTALL_DIR, "pinecone");
  if (!fs.existsSync(bin)) {
    throw new Error("archive did not contain a pinecone binary");
  }
  await fs.promises.chmod(bin, 0o755);
  output.appendLine(`Installed ${bin}`);
  return bin;
}

function platformTarget(): string {
  const key = `${process.platform}/${process.arch}`;
  switch (key) {
    case "linux/x64":
      return "x86_64-unknown-linux-gnu";
    case "darwin/arm64":
      return "aarch64-apple-darwin";
    default:
      throw new Error(
        `no prebuilt server for ${key}; build pinecone from source and set pinecone.server.path`
      );
  }
}

async function latestTag(): Promise<string> {
  const res = await get(`https://api.github.com/repos/${REPO}/releases/latest`);
  res.setEncoding("utf8");
  let body = "";
  for await (const chunk of res) {
    body += chunk;
  }
  const tag = JSON.parse(body).tag_name;
  if (!tag) {
    throw new Error("could not determine the latest release");
  }
  return tag;
}

async function fetchToFile(url: string, dest: string): Promise<void> {
  const res = await get(url);
  await new Promise<void>((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    res.pipe(file);
    file.on("finish", () => file.close((err) => (err ? reject(err) : resolve())));
    file.on("error", reject);
  });
}

function get(url: string, redirects = 5): Promise<IncomingMessage> {
  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { "User-Agent": "pinecone-vscode" } }, (res) => {
        const { statusCode = 0, headers } = res;
        if (statusCode >= 300 && statusCode < 400 && headers.location) {
          res.resume();
          return redirects > 0
            ? resolve(get(headers.location, redirects - 1))
            : reject(new Error("too many redirects"));
        }
        if (statusCode !== 200) {
          res.resume();
          return reject(new Error(`HTTP ${statusCode} for ${url}`));
        }
        resolve(res);
      })
      .on("error", reject);
  });
}