---
description: Inspect and manage project & global memories
---

# /memory Workflow

Inspect and manage project & global memories:

1. Call `get_project()` to detect project ID.
2. Call `get_memories("global", limit=100, is_permanent=true)` to list Global User Preferences.
3. Call `get_memories(project_id, limit=100)` to list Project Permanent Rules & Recent Memories.
4. Call `project_links(project_id)` to list Linked Project Rules (if linked).
5. Display a structured summary table to user:
   - **Global Rules**: User preferences across all projects
   - **Project Permanent Rules**: Architecture & project conventions
   - **Linked Project Rules**: Inherited rules from ecosystem projects
   - **Recent Task Memories**: Session progress updates
6. Prompt user if they want to edit, delete, link projects, or clean up any memory entries.
