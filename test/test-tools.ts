import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { registerTools } from "../src/tools";

const server = new McpServer({
  name: "test-server",
  version: "1.0.0"
});

registerTools(server);

// Inspect registered tools
console.log("Registered tools keys:", Object.keys((server as any)._registeredTools || {}));
console.log("Registered tools detail:", (server as any)._registeredTools);
