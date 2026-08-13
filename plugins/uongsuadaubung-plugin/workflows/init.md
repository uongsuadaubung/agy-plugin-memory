---
description: Initialize project memory and display current session context
---

# /init Workflow

Initialize project memory and display current session context:

1. Call `get_or_create_project()` to detect or register current project.
2. Call `get_memories(project_id, limit=5)` to retrieve permanent rules & recent session memories.
3. Call `list_projects()` to inspect active registered projects.
4. Report status to user:
   - 📁 **Project Name & ID**
   - 📌 **Permanent Rules Count**
   - 🕒 **Recent Memories Count**
5. Ask user: "What task would you like to tackle next?"
