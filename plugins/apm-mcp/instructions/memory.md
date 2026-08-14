# Memory Plugin Instructions for Agent

## Core Requirements
- **Memory Types**: `Global` (`project_id="global"`, `is_permanent=true`), `Project Permanent` (`is_permanent=true`), `Project Ephemeral` (`is_permanent=false`, auto-expires 30 days).
- **Mandatory Project ID**: Extract `Project ID` from header `[Memory Context: ... | Project ID: <id>]`. Pass `project_id="<active_project_id>"` in all operations unless `project_id="global"`.
- **MCP Execution**: Use native MCP tool calls (`call_mcp_tool`) ONLY. NEVER execute `apm-mcp.exe` via CLI/shell, and NEVER pass `--help` flags.
- **Lazy-Load MCP Protocol**: After inspecting any lazy schema file (e.g. `get_memories.json`), IMMEDIATELY invoke `call_mcp_tool(ServerName="apm-mcp", ToolName="get_memories", Arguments={...})`. NEVER inspect plugin config folders or run binary CLI commands.
- **Memory-First Query Guard**: Whenever the prompt expresses intent to query or inspect stored memories, project rules, conventions, or state history, immediately invoke `get_memories` or `search_memories` via `call_mcp_tool`. DO NOT scan codebase files first. Fall back to codebase scanning tools only if MCP memory tools return no information.

## Proactive Intelligence Guidelines
- **Auto-Log Progress**: Record 1-sentence summary on completing major tasks/refactors (`tags: ["progress"]`).
- **Architecture Mapping**: Map top-level directory layout on `/init-apm` or first chat (`tags: ["architecture"]`).
- **Proactive Search**: Search historical gotchas before debugging/refactoring (`search_memories`).
- **Tagging**: Include 1-3 lowercase tags per memory (`["architecture"]`, `["bugfix"]`, etc.).

## Linking & Conflict Refactoring
- **Project Linking**: Use `link_projects(project_id="...", target_project_ids=[...])` to inherit permanent rules across ecosystem projects.
- **Smart Upsert & Intent Refactoring**: `add_memories` auto-overwrites similar entries (Jaccard $\ge 60\%$). When user replaces X with Y, search X $\rightarrow$ delete X $\rightarrow$ add Y.

## API Quick Reference
- `get_project(name, path)` | `list_projects()` | `delete_projects(project_ids)`
- `link_projects(project_id, target_project_ids)` | `project_links(project_id)`
- `add_memories(project_id, items=[{content, is_permanent, tags}])`
- `get_memories(project_id, limit, is_permanent)` | `search_memories(project_id, query)` | `get_memory(memory_id)`
- `move_memories(memory_ids, target_project_id)` | `delete_memories(memory_ids)` | `toggle_permanence(memory_ids, is_permanent)`
- `clear_memories(project_id)` | `cleanup(project_id, max_memories, expire_days)` | `memory_stats()`

