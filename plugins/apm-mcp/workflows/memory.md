---
description: Inspect and manage project & global memories
---

# /memory Workflow

Inspect and manage project & global memories:

1. Call `get_memories({})` to retrieve all active Project Permanent Rules, Global Rules, and Recent Memories in 1 step.
2. Display a structured summary table to user:
   - **Global Rules**: User preferences across all projects
   - **Project Permanent Rules**: Architecture & project conventions
   - **Linked Project Rules**: Inherited rules from ecosystem projects
   - **Recent Task Memories**: Session progress updates
3. Prompt user if they want to edit, delete, link projects, or clean up any memory entries.
