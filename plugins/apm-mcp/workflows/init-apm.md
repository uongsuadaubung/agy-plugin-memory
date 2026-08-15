---
description: Initialize project memory and display current session context
---

# /init-apm Workflow

Initialize project memory and display current session context:

1. Call `get_memories({})` to retrieve active rules and memories for the current project + global.
2. Inspect top-level workspace layout and save/update Architecture Tree memory via `add_memories(items=[{content: "...", is_permanent: true, tags: ["architecture"]}])`.
3. Report status to user:
   - **Active Project Context**
   - **Permanent Rules Count**
   - **Architecture Map Status**
4. Ask user: "What task would you like to tackle next?"
