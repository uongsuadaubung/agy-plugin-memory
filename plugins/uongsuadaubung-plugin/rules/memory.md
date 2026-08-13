---
trigger: always_on
---

# Rule: Intelligent Memory Management for uongsuadaubung-plugin

## Auto-Save Triggers (Batch Unified)
- Save / Upsert Memories: User mentions "Lưu quy tắc...", "Ghi nhớ vĩnh viễn...", "Quan trọng:", "Hoàn thành task..." -> batch_add_memories(items=[{content="...", is_permanent=...}]).
- Global Memories: User mentions "Ghi nhớ toàn cục..." -> batch_add_memories(project_id="global", items=[{content="...", is_permanent=true}]).

## Auto-Delete Triggers (Batch Unified)
- Delete Memories: User mentions "Xóa ghi nhớ X...", "Quên quy tắc X đi...", "Xóa các ký ức X, Y..." -> search_memories & batch_delete_memories(memory_ids=[...]).
- Delete Projects: User mentions "Xóa dự án X...", "Xóa các project X, Y..." -> batch_delete_projects(project_ids=[...]).
- Clear Project Memories: User mentions "Xóa tất cả bộ nhớ dự án", "Reset trí nhớ dự án..." -> clear_project_memories(project_id=...).

## List & Inspection Triggers
- List Projects: User asks "Xem danh sách dự án", "Danh sách project..." -> list_projects().
