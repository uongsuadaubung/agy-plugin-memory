# Memory Plugin Instructions for Agent

## Core Requirements
- **Memory Types**: `Global` (`project_id="global"`, `is_permanent=true`), `Project Permanent` (`is_permanent=true`), `Project Ephemeral` (`is_permanent=false`, auto-expires 30 days).
- **Mandatory Project ID**: Extract `Project ID` from header `[Memory Context: ... | Project ID: <id>]`. Pass `project_id="<active_project_id>"` in all operations unless `project_id="global"`.
- **MCP Execution**: Use native MCP tool calls (`call_mcp_tool`) only. Never run MCP via CLI/shell or pass `--help` flags.

## Proactive Intelligence Guidelines
- **Auto-Log Progress**: Record 1-sentence summary on completing major tasks/refactors (`tags: ["progress"]`).
- **Architecture Mapping**: Map top-level directory layout on `/init-apm` or first chat (`tags: ["architecture"]`).
- **Proactive Search**: Search historical gotchas before debugging/refactoring (`search_memories`).
- **Tagging**: Include 1-3 lowercase tags per memory (`["architecture"]`, `["bugfix"]`, etc.).

## Linking & Conflict Refactoring
- **Project Linking**: Use `link_projects(project_id="...", target_project_ids=[...])` to inherit permanent rules across ecosystem projects.
- **Smart Upsert & Intent Refactoring**: `batch_add_memories` auto-overwrites similar entries (Jaccard $\ge 60\%$). When user replaces X with Y, search X $\rightarrow$ delete X $\rightarrow$ add Y.

## Unified Batch API Quick Reference
- `get_or_create_project(name, path)` | `list_projects()` | `batch_delete_projects(project_ids)`
- `link_projects(project_id, target_project_ids)` | `get_project_links(project_id)`
- `batch_add_memories(project_id, items=[{content, is_permanent, tags}])`
- `get_memories(project_id, limit, is_permanent)` | `search_memories(project_id, query)` | `get_memory_by_id(memory_id)`
- `move_memories(memory_ids, target_project_id)` | `batch_delete_memories(memory_ids)` | `batch_toggle_permanence(memory_ids, is_permanent)`
- `clear_project_memories(project_id)` | `cleanup_expired(project_id, max_memories, expire_days)` | `get_memory_stats()`

