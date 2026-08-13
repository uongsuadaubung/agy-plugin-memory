# Memory Plugin Instructions for Agent

## Types of Memories

1. **Global User Memories (`project_id: "global"`, `is_permanent: true`)**:
   - Stores user preferences across ALL projects.
2. **Project Permanent Memories (`is_permanent: true`)**:
   - Stores critical project rules & architecture decisions.
3. **Project Short-term Memories (`is_permanent: false`)**:
   - Stores recent session summaries & task progress (auto-expires in 30 days unless accessed).

## Proactive Agent Intelligence Guidelines

1. **Auto-Log Task Progress**:
   - Upon completing a major coding task, refactoring, or bugfix, Agent MUST automatically call `batch_add_memories(items=[{content="...", is_permanent: false, tags: ["progress"]}])` to record a 1-sentence progress summary.
2. **Project Architecture Learning**:
   - On `/init-apm` or first chat interaction in a new project, Agent MUST inspect top-level workspace files and directories and automatically call `batch_add_memories(items=[{content="Project Architecture Tree...", is_permanent: true, tags: ["architecture"]}])` to map the module layout.
   - When new directories/modules are added, Agent MUST update this memory using `tags: ["architecture"]`.
3. **Proactive Module & Error Search**:
   - When debugging complex errors or working on specific modules, Agent MUST proactively call `search_memories(query="<module_or_error>")` to retrieve historical gotchas and past solutions.
4. **Smart Tagging Requirement**:
   - Every memory created MUST include 1-3 standard lowercase tags (e.g. `["rust", "architecture", "bugfix"]`, `["database", "fts5"]`, `["workflow", "config"]`) for indexed categorization.

## Project Linking & Memory Inheritance

- **`link_projects(project_id="...", target_project_ids=["p1", "p2"])`**:
  - Links active project to target projects in the same ecosystem.
  - Linked projects inherit and load all **Permanent Rules** (`is_permanent: true`) of target projects automatically via Hook.
- **`get_project_links(project_id="...")`**:
  - Inspect current linked project IDs.

## Smart Deduplication & Conflict Refactoring

1. **Automatic Smart Upsert (Rust Engine)**:
   - `batch_add_memories` automatically computes Token Jaccard Similarity (≥60%) and Overlap Ratio against existing memories.
   - If a new memory item replaces or updates an existing concept, `apm-mcp` automatically updates/overwrites the existing record instead of creating duplicate entries.
2. **Agent Intent Refactoring**:
   - When the user expresses a rule change (e.g. *"Bỏ X dùng Y"*, *"Chuyển từ X sang Y"*, *"Thay quy tắc X bằng Y"*), the Agent MUST:
     a) Call `search_memories(query="X")` to locate obsolete memory entries.
     b) Call `batch_delete_memories(memory_ids=[...])` to remove obsolete `X` entries.
     c) Call `batch_add_memories(...)` to save the new `Y` rule.

## Unified Batch API Reference (Pass 1 item for single operations, or multiple items for batch)

### Project Management & Linking
- `get_or_create_project(name="...", path="...")`: Auto-detect active project workspace.
- `link_projects(project_id="...", target_project_ids=["p1"])`: Link current project to target projects.
- `get_project_links(project_id="...")`: Get linked project IDs for a project.
- `clear_project_memories(project_id="...")`: Delete ALL memories for a project.
- `batch_delete_projects(project_ids=["p1"])`: Delete 1 or multiple projects by ID array (protected against 'global').

### Memory Operations (Unified Batch)
- `batch_add_memories(project_id="...", items=[{content="...", is_permanent=true, tags=["tag1"]}])`: Add or smart-upsert 1 or more memories.
- `get_memories(project_id="...", limit=100, is_permanent=true)`: Retrieve valid memories.
- `search_memories(project_id="...", query="...")`: FTS5 Full-Text BM25 relevance search across memories.
- `get_memory_by_id(memory_id="...")`: Inspect a single memory by ID.
- `batch_delete_memories(memory_ids=["id1"])`: Delete 1 or more memories by ID array.
- `batch_toggle_permanence(memory_ids=["id1"], is_permanent=true)`: Toggle permanence for 1 or more memories by ID array.
- `get_memory_stats()`: Get memory database usage statistics.
