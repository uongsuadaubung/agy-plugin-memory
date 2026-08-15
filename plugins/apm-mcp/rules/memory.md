---
trigger: always_on
---

# Rule: Intelligent Memory Management for apm-mcp

## Core Execution Rules
- **Automatic CWD Context**: The server automatically targets the current workspace directory and global context. You DO NOT need to calculate or supply project IDs.
- **MCP Tool Call Exclusive**: Use native MCP tool calls (`call_mcp_tool`) ONLY. NEVER execute `apm-mcp.exe` via terminal/shell, and NEVER pass `--help` flags.
- **Forbidden Disk & DB Search**: NEVER execute PowerShell/terminal commands (e.g., `Get-ChildItem`, `dir`, `find`) to search disk for `*.db`, `memory.db`, `apm.db`, or plugin binary files. Interact with memory data ONLY via `call_mcp_tool(ServerName="apm-mcp", ToolName="...", Arguments={...})`.
- **Memory & Rule Intent Priority (Memory-First Guard)**: Whenever the user's prompt expresses intent to query, inspect, or retrieve stored memories, project rules, saved preferences, state history, or established conventions:
  1. **Mandatory MCP Memory Call First**: Invoke `get_memories` via `call_mcp_tool(ServerName="apm-mcp", ToolName="get_memories", Arguments={})`.
  2. **Forbidden Initial Codebase Scan**: The agent MUST NOT use codebase scanning tools (`list_dir`, `grep_search`, `view_file`, shell commands) as its initial response to questions asking about stored memories or project rules.
  3. **Strict Output Source**: The response provided to the user MUST contain ONLY memory records returned by `call_mcp_tool(ServerName="apm-mcp", ToolName="get_memories", ...)`. The agent MUST NOT substitute or summarize repository files/documentation as stored memories.
  4. **Controlled Fallback**: Only if MCP memory retrieval yields insufficient context may the agent inspect project files.

## Action Triggers & Tool Usage (1-Step Calls)
- **Read All / Rules**: Call `call_mcp_tool(ServerName="apm-mcp", ToolName="get_memories", Arguments={})`
- **Search by Keyword**: Call `call_mcp_tool(ServerName="apm-mcp", ToolName="get_memories", Arguments={"query": "<keyword>"})`
- **Save Project Rule**: Call `call_mcp_tool(ServerName="apm-mcp", ToolName="add_memories", Arguments={"items": [{"content": "...", "is_permanent": true}]})`
- **Save Global Rule**: Call `call_mcp_tool(ServerName="apm-mcp", ToolName="add_memories", Arguments={"items": [{"content": "...", "is_permanent": true}], "is_global": true})`
- **Save Ephemeral Progress**: Call `call_mcp_tool(ServerName="apm-mcp", ToolName="add_memories", Arguments={"items": [{"content": "...", "is_permanent": false}]})`
- **Update by ID**: Call `call_mcp_tool(ServerName="apm-mcp", ToolName="update_memory", Arguments={"memory_id": "<id>", "content": "..."})`
- **Delete by ID**: Call `call_mcp_tool(ServerName="apm-mcp", ToolName="delete_memories", Arguments={"memory_ids": ["<id>"]})`
- **Wipe Project Memories**: Call `call_mcp_tool(ServerName="apm-mcp", ToolName="clear_memories", Arguments={})`
- **Link Project Inheritance**: Call `call_mcp_tool(ServerName="apm-mcp", ToolName="link_projects", Arguments={"target_project": "<target_name_or_path>"})`
