---
trigger: always_on
---

# Rule: Intelligent Memory Management for apm-mcp

## Mandatory Project ID Rule
- Agent MUST extract the active Project ID from the injected header `[Memory Context: ... | Project ID: <id>]` at the start of each prompt (e.g., `f3c8062bad7f`).
- Agent MUST explicitly pass `project_id="<active_project_id>"` when invoking memory tools (`batch_add_memories`, `get_memories`, `search_memories`, `clear_project_memories`, `cleanup_expired`), unless `project_id="global"` is explicitly specified or inferred.

## Automatic Scope & Permanence Classification Matrix
When Agent implicitly extracts memories from natural user prompts, Agent MUST automatically route to the correct target scope and permanence level based on information type:

1. **Global Permanent Scope (`project_id="global"`, `is_permanent=true`)**:
   - Criteria: General user coding preferences, personal style guidelines, prompt language preferences, or cross-project workflow habits (e.g., "I prefer Rust over C++", "Always explain in Vietnamese", "Never use inline CSS").
   - Action: `batch_add_memories(project_id="global", items=[{content="...", is_permanent=true, tags=["preference", "global-rule"]}])`.

2. **Project Permanent Scope (`project_id="<active_project_id>"`, `is_permanent=true`)**:
   - Criteria: Architectural decisions, database choice, port configs, framework conventions, API guidelines specific to current repository.
   - Action: `batch_add_memories(project_id="<active_project_id>", items=[{content="...", is_permanent=true, tags=["architecture", "config"]}])`.

3. **Project Ephemeral / Short-Term Scope (`project_id="<active_project_id>"`, `is_permanent=false`)**:
   - Criteria: Task progress logs, temporary bugfix insights, current sprint goals, WIP state.
   - Action: `batch_add_memories(project_id="<active_project_id>", items=[{content="...", is_permanent=false, tags=["progress", "troubleshooting"]}])`.

## Automatic Smart Upsert & Conflict Resolution
Before adding any new implicit memory:
- Agent MUST perform a `search_memories` check for conflicting or outdated entries.
- If an old entry is contradicted by new user statements, Agent MUST delete or replace the outdated memory entry via `batch_delete_memories` & `batch_add_memories` to maintain memory consistency.

## Proactive Reflection & User Feedback Badge
- **Post-Turn Reflection Check**: Before concluding any response turn, Agent MUST implicitly ask itself: *"Did the user establish a new rule, preference, constraint, or bug fix strategy in this turn?"*
- **Silent Badge Notification**: Upon automatically saving an implicit memory, Agent appends a short unobtrusive note at the end of the response: `[Auto-Memory Saved: <short_summary>]`.

## Explicit Auto-Save & Manual Command Triggers
- Save / Upsert Memories: User mentions "Lưu quy tắc...", "Ghi nhớ vĩnh viễn...", "Quan trọng:", "Hoàn thành task..." -> batch_add_memories(project_id="<active_project_id>", items=[{content="...", is_permanent=...}]).
- Global Memories: User mentions "Ghi nhớ toàn cục..." -> batch_add_memories(project_id="global", items=[{content="...", is_permanent=true}]).
- Memory Conflict / Replacement: User mentions "Bỏ X dùng Y", "Chuyển từ X sang Y", "Thay thế quy tắc X...", "Từ giờ..." -> search_memories(project_id="<active_project_id>", query="X") & batch_delete_memories(memory_ids=[...]) & batch_add_memories(project_id="<active_project_id>", items=[{content="Y", is_permanent=...}]).
- Move Memories: User mentions "Chuyển ký ức X sang project Y...", "Chuyển quy tắc Z sang global..." -> move_memories(memory_ids=["..."], target_project_id="...").
- Project Linking: User mentions "Liên kết dự án X với Y...", "Thừa hưởng quy tắc từ dự án Z..." -> link_projects(project_id="<active_project_id>", target_project_ids=["..."]).

## Auto-Delete Triggers
- Delete Memories: User mentions "Xóa ghi nhớ X...", "Quên quy tắc X đi...", "Xóa các ký ức X, Y..." -> search_memories(project_id="<active_project_id>", query="X") & batch_delete_memories(memory_ids=[...]).
- Delete Projects: User mentions "Xóa dự án X...", "Xóa các project X, Y..." -> batch_delete_projects(project_ids=[...]).
- Clear Project Memories: User mentions "Xóa tất cả bộ nhớ dự án", "Reset trí nhớ dự án..." -> clear_project_memories(project_id="<active_project_id>").


