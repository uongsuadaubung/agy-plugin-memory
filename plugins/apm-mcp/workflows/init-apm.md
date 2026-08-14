---
description: Initialize project memory and display current session context
---

# /init-apm Workflow

Initialize project memory and display current session context:

1. Call `get_project()` to detect or register current project.
2. Call `get_memories(project_id, limit=30)` to retrieve permanent rules & recent session memories.
3. Inspect top-level workspace layout and save/update Architecture Tree memory via `add_memories(items=[{content: "...", is_permanent: true, tags: ["architecture"]}])`.
4. Report status to user:
   - **Project Name & ID**
   - **Permanent Rules Count**
   - **Recent Memories Count**
   - **Architecture Map Status**
5. Ask user: "What task would you like to tackle next?"
