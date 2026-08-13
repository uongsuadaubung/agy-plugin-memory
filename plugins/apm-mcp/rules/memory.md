---
trigger: always_on
---

# Rule: Intelligent Memory Management for apm-mcp

## Mandatory Project ID Rule
- Agent MUST extract the active Project ID from the injected header `[Memory Context: ... | Project ID: <id>]` at the start of each prompt (e.g., `f3c8062bad7f`).
- Agent MUST explicitly pass `project_id="<active_project_id>"` when invoking memory tools (`batch_add_memories`, `get_memories`, `search_memories`, `clear_project_memories`, `cleanup_expired`), unless `project_id="global"` is explicitly specified by the user.

## Auto-Save & Smart Refactoring Triggers (Batch Unified)
- Save / Upsert Memories: User mentions "Lưu quy tắc...", "Ghi nhớ vĩnh viễn...", "Quan trọng:", "Hoàn thành task..." -> batch_add_memories(project_id="<active_project_id>", items=[{content="...", is_permanent=...}]).
- Global Memories: User mentions "Ghi nhớ toàn cục..." -> batch_add_memories(project_id="global", items=[{content="...", is_permanent=true}]).
- Memory Conflict / Replacement: User mentions "Bỏ X dùng Y", "Chuyển từ X sang Y", "Thay thế quy tắc X...", "Từ giờ..." -> search_memories(project_id="<active_project_id>", query="X") & batch_delete_memories(memory_ids=[...]) & batch_add_memories(project_id="<active_project_id>", items=[{content="Y", is_permanent=...}]).
- Move Memories: User mentions "Chuyển ký ức X sang project Y...", "Chuyển quy tắc Z sang global..." -> move_memories(memory_ids=["..."], target_project_id="...").
- Project Linking: User mentions "Liên kết dự án X với Y...", "Thừa hưởng quy tắc từ dự án Z..." -> link_projects(project_id="<active_project_id>", target_project_ids=["..."]).

## Proactive Agent Intelligence Rules
- Auto-Log Progress: Upon completing a major task/fix, Agent automatically calls batch_add_memories(project_id="<active_project_id>", items=[{content="...", is_permanent=false, tags=["progress"]}]) to log 1-sentence progress.
- Architecture Learning: On /init-apm or first new project interaction, Agent inspects top-level workspace files/folders and automatically calls batch_add_memories(project_id="<active_project_id>", items=[{content="Project Architecture Tree...", is_permanent=true, tags=["architecture"]}]) to save the module layout map.
- Architecture Refactoring Update: When creating new module directories or refactoring layout, Agent automatically saves updated tree via batch_add_memories(project_id="<active_project_id>", items=[{content="Updated Architecture Tree...", is_permanent=true, tags=["architecture"]}]).
- Proactive Search: When debugging complex errors or working on specific modules, Agent automatically calls search_memories(project_id="<active_project_id>", query="<module_or_error>") to retrieve past insights.
- Smart Tagging: Agent MUST attach 1-3 relevant standard tags (e.g. ["rust", "architecture", "bugfix", "config"]) to every new memory.

## Auto-Delete Triggers (Batch Unified)
- Delete Memories: User mentions "Xóa ghi nhớ X...", "Quên quy tắc X đi...", "Xóa các ký ức X, Y..." -> search_memories(project_id="<active_project_id>", query="X") & batch_delete_memories(memory_ids=[...]).
- Delete Projects: User mentions "Xóa dự án X...", "Xóa các project X, Y..." -> batch_delete_projects(project_ids=[...]).
- Clear Project Memories: User mentions "Xóa tất cả bộ nhớ dự án", "Reset trí nhớ dự án..." -> clear_project_memories(project_id="<active_project_id>").
