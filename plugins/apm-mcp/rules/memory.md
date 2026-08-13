---
trigger: always_on
---

# Rule: Intelligent Memory Management for apm-mcp

## Auto-Save & Smart Refactoring Triggers (Batch Unified)
- Save / Upsert Memories: User mentions "Lưu quy tắc...", "Ghi nhớ vĩnh viễn...", "Quan trọng:", "Hoàn thành task..." -> batch_add_memories(items=[{content="...", is_permanent=...}]).
- Global Memories: User mentions "Ghi nhớ toàn cục..." -> batch_add_memories(project_id="global", items=[{content="...", is_permanent=true}]).
- Memory Conflict / Replacement: User mentions "Bỏ X dùng Y", "Chuyển từ X sang Y", "Thay thế quy tắc X...", "Từ giờ..." -> search_memories(query="X") & batch_delete_memories(memory_ids=[...]) & batch_add_memories(items=[{content="Y", is_permanent=...}]).
- Project Linking: User mentions "Liên kết dự án X với Y...", "Thừa hưởng quy tắc từ dự án Z..." -> link_projects(project_id="...", target_project_ids=["..."]).

## Proactive Agent Intelligence Rules
- Auto-Log Progress: Upon completing a major task/fix, Agent automatically calls batch_add_memories(items=[{content="...", is_permanent=false, tags=["progress"]}]) to log 1-sentence progress.
- Architecture Learning: On /init-apm or first new project interaction, Agent inspects top-level workspace files/folders and automatically calls batch_add_memories(items=[{content="Project Architecture Tree...", is_permanent=true, tags=["architecture"]}]) to save the module layout map.
- Architecture Refactoring Update: When creating new module directories or refactoring layout, Agent automatically saves updated tree via batch_add_memories(items=[{content="Updated Architecture Tree...", is_permanent=true, tags=["architecture"]}]).
- Proactive Search: When debugging complex errors or working on specific modules, Agent automatically calls search_memories(query="<module_or_error>") to retrieve past insights.
- Smart Tagging: Agent MUST attach 1-3 relevant standard tags (e.g. ["rust", "architecture", "bugfix", "config"]) to every new memory.

## Auto-Delete Triggers (Batch Unified)
- Delete Memories: User mentions "Xóa ghi nhớ X...", "Quên quy tắc X đi...", "Xóa các ký ức X, Y..." -> search_memories & batch_delete_memories(memory_ids=[...]).
- Delete Projects: User mentions "Xóa dự án X...", "Xóa các project X, Y..." -> batch_delete_projects(project_ids=[...]).
- Clear Project Memories: User mentions "Xóa tất cả bộ nhớ dự án", "Reset trí nhớ dự án..." -> clear_project_memories(project_id=...).
