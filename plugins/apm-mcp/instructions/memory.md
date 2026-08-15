# Memory Plugin Instructions for Agent

## Core Architecture
- **Auto CWD Context**: The server automatically targets the current workspace directory and global context.
- **MCP Execution**: Use native MCP tool calls (`call_mcp_tool`) ONLY.
- **Memory-First Query Guard**: Whenever querying memories, rules, or state history, immediately invoke `get_memories` via `call_mcp_tool`.

## API Quick Reference
- `get_memories(query, limit, tags, is_permanent, is_global)`: All-in-one read & search.
- `add_memories(items=[{content, is_permanent, tags}], is_global)`: Add/smart-upsert memories.
- `update_memory(memory_id, content, tags, is_permanent)`: Update by memory ID.
- `delete_memories(memory_ids=[...])`: Delete by memory IDs.
- `clear_memories(is_global)`: Clear all memories.
- `link_projects(target_project)`: Inherit rules from target project.
- `list_projects()` | `memory_stats()`
