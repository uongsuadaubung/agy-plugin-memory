import { spawn } from "child_process";

async function testRpc() {
  const child = spawn("bun", ["run", "src/index.ts"], {
    cwd: process.cwd(),
    stdio: ["pipe", "pipe", "pipe"]
  });

  child.stderr.on("data", (data) => {
    console.log("[STDERR]:", data.toString());
  });

  child.stdout.on("data", (data) => {
    console.log("[STDOUT]:", data.toString());
  });

  // Step 1: initialize
  const initMsg = JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "initialize",
    params: {
      protocolVersion: "2024-11-05",
      capabilities: {},
      clientInfo: { name: "test-client", version: "1.0.0" }
    }
  }) + "\n";

  child.stdin.write(initMsg);

  await new Promise((r) => setTimeout(r, 500));

  // Step 2: initialized notification
  const initializedMsg = JSON.stringify({
    jsonrpc: "2.0",
    method: "notifications/initialized"
  }) + "\n";

  child.stdin.write(initializedMsg);

  await new Promise((r) => setTimeout(r, 500));

  // Step 3: tools/list
  const toolsListMsg = JSON.stringify({
    jsonrpc: "2.0",
    id: 2,
    method: "tools/list",
    params: {}
  }) + "\n";

  child.stdin.write(toolsListMsg);

  await new Promise((r) => setTimeout(r, 1000));

  child.kill();
}

testRpc();
