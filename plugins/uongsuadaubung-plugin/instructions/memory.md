# Memory Plugin Instructions for Agent

## Types of Memories

1. **Global User Memories (`project_id: "global"`, `is_permanent: true`)**:
   - Stores user preferences across ALL projects.
2. **Project Permanent Memories (`is_permanent: true`)**:
   - Stores critical project rules & architecture decisions.
3. **Project Short-term Memories (`is_permanent: false`)**:
   - Stores recent session summaries & task progress (auto-expires in 30 days unless accessed).

## Unified Batch API Reference (Pass 1 item for single operations, or multiple items for batch)

### Project Management
- `get_or_create_project(name="...", path="...")`: Auto-detect active project.
- `list_projects()`: List all registered projects.
- `clear_project_memories(project_id="...")`: Delete ALL memories for a project.
- `batch_delete_projects(project_ids=["p1"])`: Delete 1 or multiple projects by ID array (protected against 'global').

### Memory Operations (Unified Batch)
- `batch_add_memories(project_id="...", items=[{content="...", is_permanent=true}])`: Add or smart-upsert 1 or more memories.
- `get_memories(project_id="...", limit=100, is_permanent=true)`: Retrieve valid memories.
- `search_memories(project_id="...", query="...")`: FTS5 Full-Text BM25 relevance search across memories.
- `get_memory_by_id(memory_id="...")`: Inspect a single memory by ID.
- `batch_delete_memories(memory_ids=["id1"])`: Delete 1 or more memories by ID array.
- `batch_toggle_permanence(memory_ids=["id1"], is_permanent=true)`: Toggle permanence for 1 or more memories by ID array.
- `get_memory_stats()`: Get memory database usage statistics.
