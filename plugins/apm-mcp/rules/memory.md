---
trigger: always_on
---

# Rule: Intelligent Memory Management for apm-mcp

## Core Execution Rules
- **Project ID**: Extract `Project ID` from prompt header `[Memory Context: ... | Project ID: <id>]`. Pass `project_id="<active_project_id>"` in all memory operations unless `project_id="global"`.
- **MCP Tool Call Exclusive**: Use native MCP tool calls (`call_mcp_tool`) only. Never run MCP via CLI/shell or pass `--help` flags.

## Scope Classification Matrix
Before saving to `global`, verify **Context Dependency**: if a rule references project-bound files, skills, configs, or local tools, route to **Project Scope**, NEVER `global`.

1. **Global Permanent (`project_id="global"`, `is_permanent=true`)**: Universal preferences & cross-project rules independent of context.
2. **Project Permanent (`project_id="<active_project_id>"`, `is_permanent=true`)**: Architecture, configs, conventions, & repo-bound workflows/skills.
3. **Project Ephemeral (`project_id="<active_project_id>"`, `is_permanent=false`)**: Sprint goals, progress logs, & temporary bugfix insights.

## Smart Upsert & Reflection
- **Conflict Resolution**: Search before adding; delete/replace obsolete entries via `batch_delete_memories` when contradicted.
- **Silent Badge**: On auto-saving memories, append note: `[Auto-Memory Saved: <summary>]`.

## Action Triggers (Semantic Intent)
- **Save / Upsert**: Save rule or milestone $\rightarrow$ `batch_add_memories(project_id="<active_project_id>", ...)`
- **Global Save**: Save universal rule $\rightarrow$ `batch_add_memories(project_id="global", is_permanent=true, ...)`
- **Replace / Conflict**: Deprecate X for Y $\rightarrow$ `search_memories` $\rightarrow$ `batch_delete_memories` $\rightarrow$ `batch_add_memories`
- **Move / Link**: Transfer memories $\rightarrow$ `move_memories`; Link projects $\rightarrow$ `link_projects`
- **Delete / Reset**: Delete memory $\rightarrow$ `batch_delete_memories`; Wipe project memories $\rightarrow$ `clear_project_memories`; Delete project $\rightarrow$ `batch_delete_projects`



