use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

use crate::project::get_auto_detected_project;
use crate::similarity::is_similar_or_replacement;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub last_active: String,
    pub memory_count: i64,
    pub linked_project_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryRecord {
    pub id: String,
    pub project_id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub metadata: Value,
    pub is_permanent: bool,
    pub created_at: String,
    pub tokens_estimated: i64,
}

pub fn map_project_row(row: &rusqlite::Row) -> rusqlite::Result<ProjectRecord> {
    let raw_linked: String = row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "[]".to_string());
    let linked_project_ids: Vec<String> = serde_json::from_str(&raw_linked).unwrap_or_default();
    Ok(ProjectRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        created_at: row.get(3)?,
        last_active: row.get(4)?,
        memory_count: row.get(5)?,
        linked_project_ids,
    })
}

pub fn map_memory_row(row: &rusqlite::Row) -> rusqlite::Result<MemoryRecord> {
    let tags_str: String = row.get(3)?;
    let meta_str: String = row.get(4)?;
    let perm_int: i32 = row.get(5)?;

    Ok(MemoryRecord {
        id: row.get(0)?,
        project_id: row.get(1)?,
        content: row.get(2)?,
        tags: serde_json::from_str(&tags_str).unwrap_or_default(),
        metadata: serde_json::from_str(&meta_str).unwrap_or(Value::Null),
        is_permanent: perm_int == 1,
        created_at: row.get(6)?,
        tokens_estimated: row.get(7)?,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchMemoryItem {
    pub content: String,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<Value>,
    pub is_permanent: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportBackup {
    pub version: String,
    pub exported_at: String,
    pub projects: Vec<ProjectRecord>,
    pub memories: Vec<MemoryRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_projects: i64,
    pub total_memories: i64,
    pub permanent_memories: i64,
    pub short_term_memories: i64,
    pub db_size_bytes: u64,
}

pub fn get_db_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".gemini");
    path.push("config");
    path.push("memory");

    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }

    path.push("memory.db");
    path
}

pub fn get_db_connection() -> Result<Connection> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA busy_timeout = 5000;
        PRAGMA synchronous = NORMAL;
        PRAGMA cache_size = -64000;
        PRAGMA temp_store = MEMORY;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS projects (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          path TEXT NOT NULL,
          created_at TEXT NOT NULL,
          last_active TEXT NOT NULL,
          memory_count INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS memories (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          content TEXT NOT NULL,
          tags TEXT DEFAULT '[]',
          metadata TEXT DEFAULT '{}',
          is_permanent INTEGER DEFAULT 0,
          created_at TEXT NOT NULL,
          tokens_estimated INTEGER NOT NULL,
          FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_memories_project_id ON memories(project_id);
        CREATE INDEX IF NOT EXISTS idx_memories_created_at ON memories(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_memories_proj_perm_created ON memories(project_id, is_permanent DESC, created_at DESC);

        CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
            id UNINDEXED,
            project_id UNINDEXED,
            content,
            tags
        );

        CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
            INSERT INTO memories_fts(id, project_id, content, tags) VALUES (new.id, new.project_id, new.content, new.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
            DELETE FROM memories_fts WHERE id = old.id;
        END;

        CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
            DELETE FROM memories_fts WHERE id = old.id;
            INSERT INTO memories_fts(id, project_id, content, tags) VALUES (new.id, new.project_id, new.content, new.tags);
        END;
        ",
    )?;

    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN linked_project_ids TEXT DEFAULT '[]'",
        [],
    );

    // Ensure global special project exists
    let now = Utc::now().to_rfc3339();
    let _ = conn.execute(
        "INSERT INTO projects (id, name, path, created_at, last_active, memory_count, linked_project_ids)
         VALUES ('global', 'Global User Memories', 'GLOBAL', ?1, ?1, 0, '[]')
         ON CONFLICT(id) DO NOTHING",
        params![now],
    );

    Ok(conn)
}

pub fn get_or_create_project(
    name: Option<&str>,
    path: Option<&str>,
    create_if_absent: bool,
) -> Result<ProjectRecord> {
    let auto_proj = get_auto_detected_project(name, path)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))))?;
    let conn = get_db_connection()?;
    let now = Utc::now().to_rfc3339();

    let mut stmt = conn.prepare("SELECT id, name, path, created_at, last_active, memory_count, linked_project_ids FROM projects WHERE id = ?1")?;
    let existing = stmt.query_row(params![auto_proj.id], map_project_row);

    if let Ok(mut proj) = existing {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memories WHERE project_id = ?1",
            params![auto_proj.id],
            |r| r.get(0),
        )?;

        conn.execute(
            "UPDATE projects SET last_active = ?1, name = ?2, path = ?3, memory_count = ?4 WHERE id = ?5",
            params![now, auto_proj.name, auto_proj.path, count, auto_proj.id],
        )?;

        proj.name = auto_proj.name;
        proj.path = auto_proj.path;
        proj.last_active = now;
        proj.memory_count = count;
        return Ok(proj);
    }

    if !create_if_absent {
        return Ok(ProjectRecord {
            id: auto_proj.id,
            name: auto_proj.name,
            path: auto_proj.path,
            created_at: now.clone(),
            last_active: now,
            memory_count: 0,
            linked_project_ids: vec![],
        });
    }

    let new_proj = ProjectRecord {
        id: auto_proj.id,
        name: auto_proj.name,
        path: auto_proj.path,
        created_at: now.clone(),
        last_active: now,
        memory_count: 0,
        linked_project_ids: vec![],
    };

    conn.execute(
        "INSERT INTO projects (id, name, path, created_at, last_active, memory_count, linked_project_ids) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            new_proj.id,
            new_proj.name,
            new_proj.path,
            new_proj.created_at,
            new_proj.last_active,
            new_proj.memory_count,
            "[]"
        ],
    )?;

    Ok(new_proj)
}

pub fn resolve_target_project_id(
    project_id: &str,
    path: Option<&str>,
    create_if_absent: bool,
) -> Result<String> {
    if project_id == "global" {
        return Ok("global".to_string());
    }

    let conn = get_db_connection()?;

    if !project_id.trim().is_empty() && project_id != "368a02e91649" {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
                params![project_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if exists {
            return Ok(project_id.to_string());
        }
    }

    let proj = get_or_create_project(None, path, create_if_absent)?;
    Ok(proj.id)
}

pub fn batch_add_memories(
    project_id: &str,
    items: Vec<BatchMemoryItem>,
    path: Option<&str>,
) -> Result<Vec<MemoryRecord>> {
    let mut conn = get_db_connection()?;
    let target_id = resolve_target_project_id(project_id, path, true)?;
    let tx = conn.transaction()?;

    let mut results = Vec::new();
    let now = Utc::now().to_rfc3339();

    for (idx, item) in items.into_iter().enumerate() {
        let trimmed_content = item.content.trim();
        let is_permanent = item.is_permanent.unwrap_or(false);
        let tags = item.tags.unwrap_or_default();
        let metadata = item.metadata.unwrap_or(json!({}));

        let mut existing_id: Option<String> = None;
        let mut final_is_permanent = is_permanent;

        {
            let mut stmt = tx.prepare("SELECT id, content, is_permanent FROM memories WHERE project_id = ?1")?;
            let existing_rows: Vec<(String, String, i32)> = stmt
                .query_map(params![target_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                .flatten()
                .collect();

            for (eid, econtent, eperm) in existing_rows {
                if is_similar_or_replacement(trimmed_content, &econtent) {
                    existing_id = Some(eid);
                    if eperm == 1 {
                        final_is_permanent = true;
                    }
                    break;
                }
            }
        }

        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        let meta_json = serde_json::to_string(&metadata).unwrap_or_else(|_| "{}".to_string());
        let tokens_estimated = ((trimmed_content.len() as f64) / 4.0).ceil() as i64;

        let mem_id = if let Some(eid) = existing_id {
            tx.execute(
                "UPDATE memories SET content = ?1, tags = ?2, metadata = ?3, is_permanent = ?4, created_at = ?5, tokens_estimated = ?6 WHERE id = ?7",
                params![
                    trimmed_content,
                    tags_json,
                    meta_json,
                    if final_is_permanent { 1 } else { 0 },
                    now,
                    tokens_estimated,
                    eid
                ],
            )?;
            eid
        } else {
            let mid = format!("{}_{}_{}", target_id, Utc::now().timestamp_millis(), idx + 1);
            tx.execute(
                "INSERT INTO memories (id, project_id, content, tags, metadata, is_permanent, created_at, tokens_estimated)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    mid,
                    target_id,
                    trimmed_content,
                    tags_json,
                    meta_json,
                    if is_permanent { 1 } else { 0 },
                    now,
                    tokens_estimated
                ],
            )?;
            mid
        };

        results.push(MemoryRecord {
            id: mem_id,
            project_id: target_id.clone(),
            content: trimmed_content.to_string(),
            tags,
            metadata,
            is_permanent,
            created_at: now.clone(),
            tokens_estimated,
        });
    }

    let count: i64 = tx.query_row(
        "SELECT COUNT(*) FROM memories WHERE project_id = ?1",
        params![target_id],
        |r| r.get(0),
    )?;

    tx.execute(
        "UPDATE projects SET memory_count = ?1, last_active = ?2 WHERE id = ?3",
        params![count, now, target_id],
    )?;

    tx.commit()?;

    let _ = cleanup_expired(&target_id, 50, 30, path);

    Ok(results)
}

pub fn get_memories(
    project_id: &str,
    limit: usize,
    tags_filter: Option<Vec<String>>,
    is_permanent: Option<bool>,
    path: Option<&str>,
) -> Result<Vec<MemoryRecord>> {
    let target_id = resolve_target_project_id(project_id, path, false)?;

    let conn = get_db_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, project_id, content, tags, metadata, is_permanent, created_at, tokens_estimated
         FROM memories WHERE project_id = ?1 ORDER BY is_permanent DESC, created_at DESC",
    )?;

    let rows = stmt.query_map(params![target_id], map_memory_row)?;

    let mut result = Vec::new();

    for r in rows.flatten() {
        if let Some(perm) = is_permanent {
            if r.is_permanent != perm {
                continue;
            }
        }
        if let Some(ref filter_tags) = tags_filter {
            if !filter_tags.is_empty() {
                let has_match = filter_tags.iter().any(|t| r.tags.contains(t));
                if !has_match {
                    continue;
                }
            }
        }

        result.push(r);
    }

    result.truncate(limit);
    Ok(result)
}

pub fn search_memories(
    project_id: &str,
    query: &str,
    limit: usize,
    path: Option<&str>,
) -> Result<Vec<MemoryRecord>> {
    let target_id = resolve_target_project_id(project_id, path, false)?;

    let conn = get_db_connection()?;
    let fts_query = format!("\"{}\"*", query.trim().replace('"', ""));

    let fts_result = conn.prepare(
        "SELECT m.id, m.project_id, m.content, m.tags, m.metadata, m.is_permanent, m.created_at, m.tokens_estimated
         FROM memories_fts fts
         JOIN memories m ON fts.id = m.id
         WHERE m.project_id = ?1 AND memories_fts MATCH ?2
         ORDER BY bm25(memories_fts), m.is_permanent DESC, m.created_at DESC",
    );

    let mut result = Vec::new();

    if let Ok(mut stmt) = fts_result {
        let rows = stmt.query_map(params![target_id, fts_query], map_memory_row);

        if let Ok(rows) = rows {
            for r in rows.flatten() {
                result.push(r);
            }
        }
    }

    if result.is_empty() {
        let pattern = format!("%{}%", query.to_lowercase());
        let mut stmt = conn.prepare(
            "SELECT id, project_id, content, tags, metadata, is_permanent, created_at, tokens_estimated
             FROM memories WHERE project_id = ?1 AND (LOWER(content) LIKE ?2 OR LOWER(tags) LIKE ?2)
             ORDER BY is_permanent DESC, created_at DESC",
        )?;

        let rows = stmt.query_map(params![target_id, pattern], map_memory_row);

        if let Ok(rows) = rows {
            for r in rows.flatten() {
                result.push(r);
            }
        }
    }

    result.truncate(limit);
    Ok(result)
}

pub fn batch_delete_memories(memory_ids: Vec<String>) -> Result<usize> {
    if memory_ids.is_empty() {
        return Ok(0);
    }
    let mut conn = get_db_connection()?;
    let tx = conn.transaction()?;
    let count;
    {
        let placeholders: Vec<String> = (1..=memory_ids.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!("DELETE FROM memories WHERE id IN ({})", placeholders.join(","));
        let mut stmt = tx.prepare(&sql)?;
        count = stmt.execute(rusqlite::params_from_iter(memory_ids.iter()))?;
    }
    tx.commit()?;
    Ok(count)
}

pub fn batch_delete_projects(project_ids: Vec<String>) -> Result<usize> {
    let safe_ids: Vec<String> = project_ids.into_iter().filter(|id| id != "global").collect();
    if safe_ids.is_empty() {
        return Ok(0);
    }
    let mut conn = get_db_connection()?;
    let tx = conn.transaction()?;
    let count;
    {
        let placeholders: Vec<String> = (1..=safe_ids.len()).map(|i| format!("?{}", i)).collect();

        let sql_m = format!("DELETE FROM memories WHERE project_id IN ({})", placeholders.join(","));
        let mut stmt_m = tx.prepare(&sql_m)?;
        let _ = stmt_m.execute(rusqlite::params_from_iter(safe_ids.iter()));

        let sql_p = format!("DELETE FROM projects WHERE id IN ({}) AND id != 'global'", placeholders.join(","));
        let mut stmt_p = tx.prepare(&sql_p)?;
        count = stmt_p.execute(rusqlite::params_from_iter(safe_ids.iter()))?;
    }
    tx.commit()?;
    Ok(count)
}

pub fn batch_toggle_permanence(memory_ids: Vec<String>, is_permanent: bool) -> Result<usize> {
    if memory_ids.is_empty() {
        return Ok(0);
    }
    let mut conn = get_db_connection()?;
    let tx = conn.transaction()?;
    let count;
    {
        let perm_int = if is_permanent { 1 } else { 0 };

        let placeholders: Vec<String> = (2..=memory_ids.len() + 1).map(|i| format!("?{}", i)).collect();
        let sql = format!("UPDATE memories SET is_permanent = ?1 WHERE id IN ({})", placeholders.join(","));
        let mut stmt = tx.prepare(&sql)?;

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        params_vec.push(Box::new(perm_int));
        for id in &memory_ids {
            params_vec.push(Box::new(id.clone()));
        }

        count = stmt.execute(rusqlite::params_from_iter(params_vec.iter().map(|b| b.as_ref())))?;
    }
    tx.commit()?;
    Ok(count)
}

pub fn move_memories(
    memory_ids: Vec<String>,
    target_project_id: &str,
) -> Result<usize> {
    if memory_ids.is_empty() {
        return Ok(0);
    }

    let mut conn = get_db_connection()?;

    if target_project_id != "global" {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
                params![target_project_id],
                |r| r.get(0),
            )
            .unwrap_or(false);
        if !exists {
            return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Target project '{}' does not exist.", target_project_id),
                ),
            )));
        }
    }

    let mut affected_source_projects: std::collections::HashSet<String> = std::collections::HashSet::new();

    {
        let placeholders: Vec<String> = (1..=memory_ids.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!("SELECT DISTINCT project_id FROM memories WHERE id IN ({})", placeholders.join(","));
        if let Ok(mut stmt) = conn.prepare(&sql) {
            let rows = stmt.query_map(rusqlite::params_from_iter(memory_ids.iter()), |r| r.get::<_, String>(0));
            if let Ok(rows) = rows {
                for r in rows.flatten() {
                    if r != "global" && r != target_project_id {
                        affected_source_projects.insert(r);
                    }
                }
            }
        }
    }

    let moved_count: usize;
    let tx = conn.transaction()?;
    {
        let placeholders: Vec<String> = (2..=memory_ids.len() + 1).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            "UPDATE memories SET project_id = ?1 WHERE id IN ({})",
            placeholders.join(",")
        );
        let mut stmt = tx.prepare(&sql)?;

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        params_vec.push(Box::new(target_project_id.to_string()));
        for id in &memory_ids {
            params_vec.push(Box::new(id.clone()));
        }

        moved_count = stmt.execute(rusqlite::params_from_iter(params_vec.iter().map(|b| b.as_ref())))?;
    }
    tx.commit()?;

    if target_project_id != "global" {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memories WHERE project_id = ?1",
            params![target_project_id],
            |r| r.get(0),
        )?;
        conn.execute(
            "UPDATE projects SET memory_count = ?1 WHERE id = ?2",
            params![count, target_project_id],
        )?;
    }

    for src_id in affected_source_projects {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memories WHERE project_id = ?1",
            params![&src_id],
            |r| r.get(0),
        )?;
        conn.execute(
            "UPDATE projects SET memory_count = ?1 WHERE id = ?2",
            params![count, src_id],
        )?;
    }

    Ok(moved_count)
}

pub fn get_memory_by_id(memory_id: &str) -> Result<Option<MemoryRecord>> {
    let conn = get_db_connection()?;
    let mut stmt = conn.prepare("SELECT id, project_id, content, tags, metadata, is_permanent, created_at, tokens_estimated FROM memories WHERE id = ?1")?;
    let res = stmt
        .query_row(params![memory_id], |row| {
            let tags_str: String = row.get(3)?;
            let meta_str: String = row.get(4)?;
            let perm: i32 = row.get(5)?;
            Ok(MemoryRecord {
                id: row.get(0)?,
                project_id: row.get(1)?,
                content: row.get(2)?,
                tags: serde_json::from_str(&tags_str).unwrap_or_default(),
                metadata: serde_json::from_str(&meta_str).unwrap_or(Value::Null),
                is_permanent: perm == 1,
                created_at: row.get(6)?,
                tokens_estimated: row.get(7)?,
            })
        })
        .ok();
    Ok(res)
}

pub fn get_memory_stats() -> Result<MemoryStats> {
    let conn = get_db_connection()?;
    let total_projects: i64 = conn.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0)).unwrap_or(0);
    let total_memories: i64 = conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0)).unwrap_or(0);
    let permanent_memories: i64 = conn.query_row("SELECT COUNT(*) FROM memories WHERE is_permanent = 1", [], |r| r.get(0)).unwrap_or(0);
    let short_term_memories: i64 = conn.query_row("SELECT COUNT(*) FROM memories WHERE is_permanent = 0", [], |r| r.get(0)).unwrap_or(0);

    let db_path = get_db_path();
    let db_size_bytes = fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

    Ok(MemoryStats {
        total_projects,
        total_memories,
        permanent_memories,
        short_term_memories,
        db_size_bytes,
    })
}

pub fn clear_project_memories(project_id: &str, path: Option<&str>) -> Result<usize> {
    let target_id = resolve_target_project_id(project_id, path, false)?;

    if target_id == "global" {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    let conn = get_db_connection()?;
    let deleted_count = conn.execute("DELETE FROM memories WHERE project_id = ?1", params![target_id])?;
    conn.execute("UPDATE projects SET memory_count = 0 WHERE id = ?1", params![target_id])?;

    Ok(deleted_count)
}

pub fn link_projects(
    project_id: &str,
    target_project_ids: Vec<String>,
    path: Option<&str>,
) -> Result<ProjectRecord> {
    let target_id = resolve_target_project_id(project_id, path, true)?;

    let filtered_targets: Vec<String> = target_project_ids
        .into_iter()
        .filter(|id| id != &target_id)
        .collect();

    let json_targets = serde_json::to_string(&filtered_targets).unwrap_or_else(|_| "[]".to_string());

    let conn = get_db_connection()?;
    conn.execute(
        "UPDATE projects SET linked_project_ids = ?1 WHERE id = ?2",
        params![json_targets, target_id],
    )?;

    get_or_create_project(None, path, false)
}

pub fn get_linked_project_ids(project_id: &str) -> Vec<String> {
    let conn = match get_db_connection() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let raw: String = conn
        .query_row(
            "SELECT linked_project_ids FROM projects WHERE id = ?1",
            params![project_id],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "[]".to_string());

    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn list_projects() -> Result<Vec<ProjectRecord>> {
    let conn = get_db_connection()?;
    let mut stmt = conn.prepare("SELECT id, name, path, created_at, last_active, memory_count, linked_project_ids FROM projects ORDER BY last_active DESC")?;
    let rows = stmt.query_map([], map_project_row)?;

    let mut list = Vec::new();
    for r in rows.flatten() {
        list.push(r);
    }
    Ok(list)
}

pub fn cleanup_expired(
    project_id: &str,
    max_memories: usize,
    expire_days: i64,
    path: Option<&str>,
) -> Result<usize> {
    let target_id = resolve_target_project_id(project_id, path, false)?;

    let conn = get_db_connection()?;
    let cutoff = (Utc::now() - chrono::Duration::days(expire_days)).to_rfc3339();

    let mut deleted_count = conn.execute(
        "DELETE FROM memories WHERE project_id = ?1 AND is_permanent = 0 AND created_at < ?2",
        params![target_id, cutoff],
    )?;

    let mut stmt = conn.prepare(
        "SELECT id FROM memories WHERE project_id = ?1 AND is_permanent = 0 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![target_id], |row| row.get::<_, String>(0))?;

    let mut ids = Vec::new();
    for r in rows.flatten() {
        ids.push(r);
    }

    if ids.len() > max_memories {
        let to_delete = &ids[max_memories..];
        for id in to_delete {
            let res = conn.execute("DELETE FROM memories WHERE id = ?1", params![id])?;
            deleted_count += res;
        }
    }

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memories WHERE project_id = ?1",
        params![target_id],
        |r| r.get(0),
    )?;

    conn.execute(
        "UPDATE projects SET memory_count = ?1 WHERE id = ?2",
        params![count, target_id],
    )?;

    Ok(deleted_count)
}

pub fn export_memories_to_json(file_path: &str) -> Result<String, String> {
    let projects = list_projects().map_err(|e| e.to_string())?;
    let conn = get_db_connection().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, project_id, content, tags, metadata, is_permanent, created_at, tokens_estimated FROM memories")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let tags_str: String = row.get(3)?;
            let meta_str: String = row.get(4)?;
            let perm_int: i32 = row.get(5)?;

            Ok(MemoryRecord {
                id: row.get(0)?,
                project_id: row.get(1)?,
                content: row.get(2)?,
                tags: serde_json::from_str(&tags_str).unwrap_or_default(),
                metadata: serde_json::from_str(&meta_str).unwrap_or(Value::Null),
                is_permanent: perm_int == 1,
                created_at: row.get(6)?,
                tokens_estimated: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut memories = Vec::new();
    for r in rows.flatten() {
        memories.push(r);
    }

    let backup = ExportBackup {
        version: "1.0.0".to_string(),
        exported_at: Utc::now().to_rfc3339(),
        projects,
        memories,
    };

    let json_str = serde_json::to_string_pretty(&backup).map_err(|e| e.to_string())?;
    fs::write(file_path, &json_str).map_err(|e| e.to_string())?;

    Ok(file_path.to_string())
}

pub fn import_memories_from_json(file_path: &str) -> Result<(usize, usize), String> {
    let json_str = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let backup: ExportBackup = serde_json::from_str(&json_str).map_err(|e| e.to_string())?;

    let conn = get_db_connection().map_err(|e| e.to_string())?;
    let mut proj_count = 0;
    let mut mem_count = 0;

    for p in backup.projects {
        let res = conn.execute(
            "INSERT INTO projects (id, name, path, created_at, last_active, memory_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET name = ?2, path = ?3, last_active = ?5, memory_count = ?6",
            params![p.id, p.name, p.path, p.created_at, p.last_active, p.memory_count],
        );
        if res.is_ok() {
            proj_count += 1;
        }
    }

    for m in backup.memories {
        let tags_json = serde_json::to_string(&m.tags).unwrap_or_else(|_| "[]".to_string());
        let meta_json = serde_json::to_string(&m.metadata).unwrap_or_else(|_| "{}".to_string());

        let res = conn.execute(
            "INSERT INTO memories (id, project_id, content, tags, metadata, is_permanent, created_at, tokens_estimated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET content = ?3, tags = ?4, metadata = ?5, is_permanent = ?6, created_at = ?7, tokens_estimated = ?8",
            params![
                m.id,
                m.project_id,
                m.content,
                tags_json,
                meta_json,
                if m.is_permanent { 1 } else { 0 },
                m.created_at,
                m.tokens_estimated
            ],
        );
        if res.is_ok() {
            mem_count += 1;
        }
    }

    Ok((proj_count, mem_count))
}
