import * as path from "path";
import { runTests } from "@vscode/test-electron";

async function main() {
  try {
    const extensionDevelopmentPath = path.resolve(__dirname, "../../");
    const extensionTestsPath = path.resolve(__dirname, "./suite/index");
    const repoRoot = path.resolve(extensionDevelopmentPath, "../..");
    const serverPath =
      process.env.SERVER_PATH ||
      path.join(repoRoot, "target", "debug", "pinecone");

    await runTests({
      extensionDevelopmentPath,
      extensionTestsPath,
      launchArgs: ["--disable-extensions"],
      extensionTestsEnv: { SERVER_PATH: serverPath },
    });
  } catch (err) {
    console.error("Failed to run tests", err);
    process.exit(1);
  }
}

main();
