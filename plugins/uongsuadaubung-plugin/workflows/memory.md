---
description: Inspect and manage project & global memories
---

# /memory Workflow

Inspect and manage project & global memories:

1. Call `get_or_create_project()` to detect project ID.
2. Call `get_memories("global", limit=10, is_permanent=true)` to list Global User Preferences.
3. Call `get_memories(project_id, limit=10)` to list Project Permanent Rules & Recent Memories.
4. Display a structured summary table to user:
   - **Global Rules**: User preferences
   - **Project Permanent Rules**: Architecture & project conventions
   - **Recent Task Memories**: Session progress updates
5. Prompt user if they want to pin, edit, delete, or clean up any memory entries.
