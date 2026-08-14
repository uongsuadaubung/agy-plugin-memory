---
trigger: always_on
---

# Rule: Intelligent Memory Management for apm-mcp

## Core Execution Rules
- **Project ID**: Extract `Project ID` from prompt header `[Memory Context: ... | Project ID: <id>]`. Pass `project_id="<active_project_id>"` in all memory operations unless `project_id="global"`.
- **MCP Tool Call Exclusive**: Use native MCP tool calls (`call_mcp_tool`) ONLY. NEVER execute `apm-mcp.exe` via terminal/shell, and NEVER pass `--help` flags.
- **Lazy-Load MCP Protocol**: After inspecting any lazy schema (e.g., `get_memories.json`), IMMEDIATELY invoke `call_mcp_tool(ServerName="apm-mcp", ToolName="get_memories", Arguments={...})`. NEVER inspect plugin config folders or run binary CLI commands.
- **Forbidden Disk & DB Search**: NEVER execute PowerShell/terminal commands (e.g., `Get-ChildItem`, `dir`, `find`) to search disk for `*.db`, `memory.db`, `apm.db`, or plugin binary files. The database is managed EXCLUSIVELY by `apm-mcp`. Interact with memory data ONLY via `call_mcp_tool(ServerName="apm-mcp", ToolName="...", Arguments={...})`.
- **Memory & Rule Intent Priority (Memory-First Guard)**: Whenever the user's prompt expresses intent to query, inspect, or retrieve stored memories, project rules, saved preferences, state history, or established conventions:
  1. **Mandatory MCP Memory Call First**: The agent MUST first invoke `get_memories` and/or `search_memories` via `call_mcp_tool`.
  2. **Forbidden Initial Codebase Scan**: The agent MUST NOT use codebase scanning tools (`list_dir`, `grep_search`, `view_file`, shell commands) as its initial response to questions asking about stored memories or project rules.
  3. **Strict Output Source**: The response provided to the user MUST contain ONLY memory records returned by `call_mcp_tool(ServerName="apm-mcp", ToolName="get_memories", ...)`. The agent MUST NOT substitute or summarize repository files/documentation as stored memories.
  4. **Controlled Fallback**: Only if MCP memory retrieval yields insufficient context may the agent inspect project files.

## Scope Classification Matrix
Before saving to `global`, verify **Context Dependency**: if a rule references project-bound files, skills, configs, or local tools, route to **Project Scope**, NEVER `global`.

1. **Global Permanent (`project_id="global"`, `is_permanent=true`)**: Universal preferences & cross-project rules independent of context.
2. **Project Permanent (`project_id="<active_project_id>"`, `is_permanent=true`)**: Architecture, configs, conventions, & repo-bound workflows/skills.
3. **Project Ephemeral (`project_id="<active_project_id>"`, `is_permanent=false`)**: Sprint goals, progress logs, & temporary bugfix insights.

## Smart Upsert & Reflection
- **Conflict Resolution**: Search before adding; delete/replace obsolete entries via `delete_memories` when contradicted.
- **Silent Badge**: On auto-saving memories, append note: `[Auto-Memory Saved: <summary>]`.

## Action Triggers (Semantic Intent)
- **Save / Upsert**: Save rule or milestone $\rightarrow$ `add_memories(project_id="<active_project_id>", ...)`
- **Global Save**: Save universal rule $\rightarrow$ `add_memories(project_id="global", is_permanent=true, ...)`
- **Update / Direct Edit**: Edit existing rule directly by ID $\rightarrow$ `update_memory(memory_id="<id>", content="...", ...)`
- **Replace / Conflict**: Deprecate X for Y $\rightarrow$ use `update_memory` or `search_memories` $\rightarrow$ `delete_memories` $\rightarrow$ `add_memories`
- **Move / Link**: Transfer memories $\rightarrow$ `move_memories`; Link projects $\rightarrow$ `link_projects`
- **Delete / Reset**: Delete memory $\rightarrow$ `delete_memories`; Wipe project memories $\rightarrow$ `clear_memories`; Delete project $\rightarrow$ `delete_projects`



