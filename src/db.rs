use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::project::get_auto_detected_project;
use crate::similarity::is_similar_or_replacement;

static DB_INIT: OnceLock<()> = OnceLock::new();

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
    pub created_at: String,
    pub tags: Vec<String>,
    pub metadata: Value,
    pub tokens_estimated: i64,
    pub is_permanent: bool,
}

pub fn map_project_row(row: &rusqlite::Row) -> rusqlite::Result<ProjectRecord> {
    let raw_linked = row.get_ref(6)?.as_str().unwrap_or("[]");
    let linked_project_ids: Vec<String> = serde_json::from_str(raw_linked).unwrap_or_default();
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
    let tags_ref = row.get_ref(3)?.as_str().unwrap_or("[]");
    let meta_ref = row.get_ref(4)?.as_str().unwrap_or("{}");
    let perm_int: i32 = row.get(5)?;

    Ok(MemoryRecord {
        id: row.get(0)?,
        project_id: row.get(1)?,
        content: row.get(2)?,
        created_at: row.get(6)?,
        tags: serde_json::from_str(tags_ref).unwrap_or_default(),
        metadata: serde_json::from_str(meta_ref).unwrap_or(Value::Null),
        tokens_estimated: row.get(7)?,
        is_permanent: perm_int == 1,
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
        ",
    )?;

    DB_INIT.get_or_init(|| {
        let _ = conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              path TEXT NOT NULL,
              created_at TEXT NOT NULL,
              last_active TEXT NOT NULL,
              memory_count INTEGER DEFAULT 0,
              linked_project_ids TEXT DEFAULT '[]'
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
            CREATE TABLE IF NOT EXISTS active_sessions (
                session_key TEXT PRIMARY KEY,
                workspace_path TEXT NOT NULL,
                project_id TEXT NOT NULL,
                project_name TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        );

        let now = Utc::now().to_rfc3339();
        let _ = conn.execute(
            "INSERT INTO projects (id, name, path, created_at, last_active, memory_count, linked_project_ids)
             VALUES ('global', 'Global User Memories', 'GLOBAL', ?1, ?1, 0, '[]')
             ON CONFLICT(id) DO NOTHING",
            params![now],
        );
    });

    Ok(conn)
}

pub fn set_active_workspace(
    path: &str,
    project_id: &str,
    project_name: &str,
    parent_pid: u32,
    conversation_id: Option<&str>,
) -> Result<()> {
    let conn = get_db_connection()?;
    let now = Utc::now().to_rfc3339();

    // 1. Lưu session_key = 'latest' cho fallback chung
    let _ = conn.execute(
        "INSERT INTO active_sessions (session_key, workspace_path, project_id, project_name, updated_at)
         VALUES ('latest', ?1, ?2, ?3, ?4)
         ON CONFLICT(session_key) DO UPDATE SET workspace_path = ?1, project_id = ?2, project_name = ?3, updated_at = ?4",
        params![path, project_id, project_name, now],
    );

    // 2. Lưu theo Parent Process ID để cô lập tuyệt đối giữa Window A và Window B
    if parent_pid > 0 {
        let ppid_key = format!("ppid_{parent_pid}");
        let _ = conn.execute(
            "INSERT INTO active_sessions (session_key, workspace_path, project_id, project_name, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(session_key) DO UPDATE SET workspace_path = ?2, project_id = ?3, project_name = ?4, updated_at = ?5",
            params![ppid_key, path, project_id, project_name, now],
        );
    }

    // 3. Lưu theo Conversation ID nếu có
    if let Some(cid) = conversation_id {
        let trimmed = cid.trim();
        if !trimmed.is_empty() {
            let conv_key = format!("conv_{trimmed}");
            let _ = conn.execute(
                "INSERT INTO active_sessions (session_key, workspace_path, project_id, project_name, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(session_key) DO UPDATE SET workspace_path = ?2, project_id = ?3, project_name = ?4, updated_at = ?5",
                params![conv_key, path, project_id, project_name, now],
            );
        }
    }

    Ok(())
}

pub fn get_active_workspace() -> Option<(String, String, String)> {
    let conn = get_db_connection().ok()?;
    let ppid = crate::process::get_parent_pid();

    // Ưu tiên 1: Tra cứu theo Parent Process ID của chính cửa sổ này
    if ppid > 0 {
        let ppid_key = format!("ppid_{ppid}");
        let res = conn.query_row(
            "SELECT workspace_path, project_id, project_name FROM active_sessions WHERE session_key = ?1",
            params![ppid_key],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );
        if let Ok(val) = res {
            return Some(val);
        }
    }

    // Ưu tiên 2: Fallback lấy session 'latest'
    conn.query_row(
        "SELECT workspace_path, project_id, project_name FROM active_sessions WHERE session_key = 'latest'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).ok()
}

pub fn extract_tags_from_content(content: &str) -> Vec<String> {
    let mut tags = HashSet::new();

    if let Some(start) = content.find("**") {
        if let Some(end) = content[start + 2..].find("**") {
            let header = &content[start + 2..start + 2 + end];
            let clean = header.trim_matches(|c: char| !c.is_alphanumeric() && c != ' ' && c != '_');
            let tag = clean.to_lowercase().replace(' ', "_");
            if !tag.is_empty() && tag.len() <= 40 {
                tags.insert(tag);
            }
        }
    }

    let lower = content.to_lowercase();
    let keywords = [
        "typescript", "javascript", "python", "rust", "react", "solidjs", "vue", "svelte",
        "tailwind", "scss", "css", "clean_code", "refactor", "error_handling", "git", "bun",
        "npm", "pnpm", "yarn", "database", "sqlite", "postgres", "auth", "security", "rule",
    ];

    for kw in keywords {
        if lower.contains(kw) {
            tags.insert((*kw).to_string());
        }
    }

    if tags.is_empty() {
        vec!["rule".to_string()]
    } else {
        tags.into_iter().collect()
    }
}

pub fn resolve_target_project_id(
    is_global: bool,
    project_override: Option<&str>,
    path: Option<&str>,
    create_if_absent: bool,
) -> Result<String> {
    if is_global {
        return Ok("global".to_string());
    }

    let conn = get_db_connection()?;

    if let Some(p) = project_override {
        let trimmed = p.trim();
        if trimmed.eq_ignore_ascii_case("global") {
            return Ok("global".to_string());
        }
        if !trimmed.is_empty() {
            let found_id: Option<String> = conn
                .query_row(
                    "SELECT id FROM projects WHERE id = ?1 OR name = ?1 COLLATE NOCASE LIMIT 1",
                    params![trimmed],
                    |row| row.get(0),
                )
                .ok();

            if let Some(id) = found_id {
                return Ok(id);
            }
        }
    }

    match get_project(None, path, create_if_absent) {
        Ok(proj) => Ok(proj.id),
        Err(_) => {
            let fallback_id: Option<String> = conn
                .query_row(
                    "SELECT id FROM projects WHERE id != 'global' ORDER BY last_active DESC LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .ok();

            if let Some(id) = fallback_id {
                Ok(id)
            } else {
                Ok("global".to_string())
            }
        }
    }
}

pub fn get_project(
    name: Option<&str>,
    path: Option<&str>,
    create_if_absent: bool,
) -> Result<ProjectRecord> {
    let conn = get_db_connection()?;
    let now = Utc::now().to_rfc3339();

    let auto_proj = match get_auto_detected_project(name, path) {
        Ok(p) => p,
        Err(e) => {
            let fallback: rusqlite::Result<ProjectRecord> = conn.query_row(
                "SELECT id, name, path, created_at, last_active, memory_count, linked_project_ids
                 FROM projects WHERE id != 'global' ORDER BY last_active DESC LIMIT 1",
                [],
                map_project_row,
            );

            if let Ok(proj) = fallback {
                return Ok(proj);
            } else {
                return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))));
            }
        }
    };

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

fn build_fts5_query(raw: &str) -> String {
    let terms: Vec<String> = raw
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| !w.is_empty() && w.len() > 1)
        .map(|w| format!("\"{w}\"*"))
        .collect();

    if terms.is_empty() {
        format!("\"{}\"*", raw.trim().replace('"', ""))
    } else {
        terms.join(" AND ")
    }
}

pub fn get_memories(
    query: Option<&str>,
    limit: usize,
    tags_filter: Option<Vec<String>>,
    is_permanent: Option<bool>,
    is_global: bool,
    project_override: Option<&str>,
    path: Option<&str>,
) -> Result<Vec<MemoryRecord>> {
    let conn = get_db_connection()?;

    let project_ids = if is_global {
        vec!["global".to_string()]
    } else {
        let target_id = resolve_target_project_id(false, project_override, path, false)?;
        let linked = get_linked_project_ids(&target_id);
        let mut ids = vec![target_id, "global".to_string()];
        for lid in linked {
            if !ids.contains(&lid) {
                ids.push(lid);
            }
        }
        ids
    };

    if project_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<String> = (1..=project_ids.len()).map(|i| format!("?{i}")).collect();
    let in_clause = placeholders.join(", ");

    let mut result = Vec::new();

    if let Some(q) = query {
        let clean_q = q.trim();
        if !clean_q.is_empty() {
            let fts_query = build_fts5_query(clean_q);
            let fts_sql = format!(
                "SELECT m.id, m.project_id, m.content, m.tags, m.metadata, m.is_permanent, m.created_at, m.tokens_estimated
                 FROM memories_fts fts
                 JOIN memories m ON fts.id = m.id
                 WHERE m.project_id IN ({in_clause}) AND memories_fts MATCH ?{}
                 ORDER BY bm25(memories_fts), m.is_permanent DESC, m.created_at DESC",
                project_ids.len() + 1
            );

            if let Ok(mut stmt) = conn.prepare(&fts_sql) {
                let mut params_vec: Vec<rusqlite::types::Value> = project_ids
                    .iter()
                    .map(|id| rusqlite::types::Value::Text(id.clone()))
                    .collect();
                params_vec.push(rusqlite::types::Value::Text(fts_query));

                let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), map_memory_row);
                if let Ok(rows) = rows {
                    for r in rows.flatten() {
                        result.push(r);
                    }
                }
            }

            if result.is_empty() {
                let pattern = format!("%{}%", clean_q.to_lowercase());
                let fallback_sql = format!(
                    "SELECT id, project_id, content, tags, metadata, is_permanent, created_at, tokens_estimated
                     FROM memories WHERE project_id IN ({in_clause}) AND (LOWER(content) LIKE ?{} OR LOWER(tags) LIKE ?{})
                     ORDER BY is_permanent DESC, created_at DESC",
                    project_ids.len() + 1,
                    project_ids.len() + 2
                );

                if let Ok(mut stmt) = conn.prepare(&fallback_sql) {
                    let mut params_vec: Vec<rusqlite::types::Value> = project_ids
                        .iter()
                        .map(|id| rusqlite::types::Value::Text(id.clone()))
                        .collect();
                    params_vec.push(rusqlite::types::Value::Text(pattern.clone()));
                    params_vec.push(rusqlite::types::Value::Text(pattern));

                    let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), map_memory_row);
                    if let Ok(rows) = rows {
                        for r in rows.flatten() {
                            result.push(r);
                        }
                    }
                }
            }
        }
    }

    if query.map_or(true, |q| q.trim().is_empty()) {
        let sql = format!(
            "SELECT id, project_id, content, tags, metadata, is_permanent, created_at, tokens_estimated
             FROM memories WHERE project_id IN ({in_clause})
             ORDER BY is_permanent DESC, created_at DESC"
        );

        let mut stmt = conn.prepare(&sql)?;
        let params_vec: Vec<rusqlite::types::Value> = project_ids
            .iter()
            .map(|id| rusqlite::types::Value::Text(id.clone()))
            .collect();

        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), map_memory_row)?;
        for r in rows.flatten() {
            result.push(r);
        }
    }

    let mut filtered = Vec::new();
    for r in result {
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
        filtered.push(r);
    }

    filtered.truncate(limit);
    Ok(filtered)
}

pub fn add_memories(
    items: Vec<BatchMemoryItem>,
    is_global: bool,
    project_override: Option<&str>,
    path: Option<&str>,
) -> Result<Vec<MemoryRecord>> {
    let mut conn = get_db_connection()?;
    let target_id = resolve_target_project_id(is_global, project_override, path, true)?;
    let tx = conn.transaction()?;

    let mut existing_mems = Vec::new();
    {
        let mut stmt = tx.prepare(
            "SELECT id, project_id, content, tags, metadata, is_permanent, created_at, tokens_estimated
             FROM memories WHERE project_id = ?1",
        )?;
        let rows = stmt.query_map(params![target_id], map_memory_row)?;
        for r in rows.flatten() {
            existing_mems.push(r);
        }
    }

    let mut results = Vec::new();
    let now = Utc::now().to_rfc3339();

    for (idx, item) in items.into_iter().enumerate() {
        let trimmed_content = item.content.trim();
        if trimmed_content.is_empty() {
            continue;
        }

        let tags = match item.tags {
            Some(t) if !t.is_empty() => t,
            _ => extract_tags_from_content(trimmed_content),
        };

        let metadata = item.metadata.unwrap_or_else(|| json!({}));
        let is_permanent = item.is_permanent.unwrap_or(false);
        let tokens_estimated = (trimmed_content.len() / 4).max(1) as i64;

        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        let meta_json = serde_json::to_string(&metadata).unwrap_or_else(|_| "{}".to_string());

        let mut matched_id = None;
        for em in &existing_mems {
            if is_similar_or_replacement(trimmed_content, &em.content) {
                matched_id = Some(em.id.clone());
                break;
            }
        }

        let mem_id = if let Some(eid) = matched_id {
            tx.execute(
                "UPDATE memories
                 SET content = ?1, tags = ?2, metadata = ?3, is_permanent = ?4, created_at = ?5, tokens_estimated = ?6
                 WHERE id = ?7",
                params![
                    trimmed_content,
                    tags_json,
                    meta_json,
                    if is_permanent { 1 } else { 0 },
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

    let _ = cleanup(is_global, project_override, 50, 30, path);

    if let Ok(c) = get_db_connection() {
        let _ = c.execute("PRAGMA optimize;", []);
    }

    Ok(results)
}

pub fn get_memory(memory_id: &str) -> Result<Option<MemoryRecord>> {
    let conn = get_db_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, project_id, content, tags, metadata, is_permanent, created_at, tokens_estimated
         FROM memories WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![memory_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_memory_row(row)?))
    } else {
        Ok(None)
    }
}

pub fn update_memory(
    memory_id: &str,
    content: Option<&str>,
    tags: Option<Vec<String>>,
    metadata: Option<Value>,
    is_permanent: Option<bool>,
) -> Result<Option<MemoryRecord>> {
    let conn = get_db_connection()?;

    let existing = match get_memory(memory_id)? {
        Some(m) => m,
        None => return Ok(None),
    };

    let new_content = content.unwrap_or(&existing.content).trim();
    let new_tags = tags.unwrap_or(existing.tags);
    let new_metadata = metadata.unwrap_or(existing.metadata);
    let new_permanent = is_permanent.unwrap_or(existing.is_permanent);
    let tokens_estimated = (new_content.len() / 4).max(1) as i64;
    let now = Utc::now().to_rfc3339();

    let tags_json = serde_json::to_string(&new_tags).unwrap_or_else(|_| "[]".to_string());
    let meta_json = serde_json::to_string(&new_metadata).unwrap_or_else(|_| "{}".to_string());

    conn.execute(
        "UPDATE memories
         SET content = ?1, tags = ?2, metadata = ?3, is_permanent = ?4, created_at = ?5, tokens_estimated = ?6
         WHERE id = ?7",
        params![
            new_content,
            tags_json,
            meta_json,
            if new_permanent { 1 } else { 0 },
            now,
            tokens_estimated,
            memory_id
        ],
    )?;

    get_memory(memory_id)
}

pub fn delete_memories(memory_ids: Vec<String>) -> Result<usize> {
    if memory_ids.is_empty() {
        return Ok(0);
    }

    let mut conn = get_db_connection()?;
    let tx = conn.transaction()?;

    let mut affected_projects = HashSet::new();
    for id in &memory_ids {
        if let Ok(Some(m)) = get_memory(id) {
            affected_projects.insert(m.project_id);
        }
    }

    let placeholders: Vec<String> = (1..=memory_ids.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "DELETE FROM memories WHERE id IN ({})",
        placeholders.join(", ")
    );

    let mut stmt = tx.prepare(&sql)?;
    let params_vec: Vec<rusqlite::types::Value> = memory_ids
        .into_iter()
        .map(rusqlite::types::Value::Text)
        .collect();

    let deleted_count = stmt.execute(rusqlite::params_from_iter(params_vec))?;
    drop(stmt);

    let now = Utc::now().to_rfc3339();
    for pid in affected_projects {
        let count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM memories WHERE project_id = ?1",
            params![pid],
            |r| r.get(0),
        )?;
        tx.execute(
            "UPDATE projects SET memory_count = ?1, last_active = ?2 WHERE id = ?3",
            params![count, now, pid],
        )?;
    }

    tx.commit()?;
    Ok(deleted_count)
}

pub fn delete_projects(project_identifiers: Vec<String>) -> Result<usize> {
    if project_identifiers.is_empty() {
        return Ok(0);
    }

    let conn = get_db_connection()?;
    let mut deleted_count = 0;

    for ident in project_identifiers {
        let trimmed = ident.trim();
        if trimmed == "global" {
            continue;
        }

        let res = conn.execute(
            "DELETE FROM projects WHERE id = ?1 OR name = ?1 COLLATE NOCASE",
            params![trimmed],
        )?;
        deleted_count += res;
    }

    Ok(deleted_count)
}

pub fn toggle_permanence(memory_ids: Vec<String>, is_permanent: bool) -> Result<usize> {
    if memory_ids.is_empty() {
        return Ok(0);
    }

    let conn = get_db_connection()?;
    let placeholders: Vec<String> = (1..=memory_ids.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "UPDATE memories SET is_permanent = ?{} WHERE id IN ({})",
        memory_ids.len() + 1,
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&sql)?;
    let mut params_vec: Vec<rusqlite::types::Value> = memory_ids
        .into_iter()
        .map(rusqlite::types::Value::Text)
        .collect();
    params_vec.push(rusqlite::types::Value::Integer(if is_permanent { 1 } else { 0 }));

    let count = stmt.execute(rusqlite::params_from_iter(params_vec))?;
    Ok(count)
}

pub fn memory_stats() -> Result<MemoryStats> {
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

pub fn clear_memories(
    is_global: bool,
    project_override: Option<&str>,
    path: Option<&str>,
) -> Result<usize> {
    let target_id = resolve_target_project_id(is_global, project_override, path, false)?;

    let conn = get_db_connection()?;
    let deleted_count = conn.execute("DELETE FROM memories WHERE project_id = ?1", params![target_id])?;
    conn.execute("UPDATE projects SET memory_count = 0 WHERE id = ?1", params![target_id])?;

    Ok(deleted_count)
}

pub fn link_projects(
    target_project: &str,
    source_project: Option<&str>,
    path: Option<&str>,
) -> Result<ProjectRecord> {
    let source_id = resolve_target_project_id(false, source_project, path, true)?;
    let target_id = resolve_target_project_id(false, Some(target_project), None, false)?;

    let mut current_links = get_linked_project_ids(&source_id);
    if target_id != source_id && target_id != "global" && !current_links.contains(&target_id) {
        current_links.push(target_id);
    }

    let json_targets = serde_json::to_string(&current_links).unwrap_or_else(|_| "[]".to_string());

    let conn = get_db_connection()?;
    conn.execute(
        "UPDATE projects SET linked_project_ids = ?1 WHERE id = ?2",
        params![json_targets, source_id],
    )?;

    get_project(None, path, false)
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

pub fn cleanup(
    is_global: bool,
    project_override: Option<&str>,
    max_memories: usize,
    expire_days: i64,
    path: Option<&str>,
) -> Result<usize> {
    let target_id = match resolve_target_project_id(is_global, project_override, path, false) {
        Ok(id) => id,
        Err(_) => return Ok(0),
    };

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

pub fn move_memories(
    memory_ids: Vec<String>,
    target_is_global: bool,
    target_project: Option<&str>,
) -> Result<usize> {
    if memory_ids.is_empty() {
        return Ok(0);
    }

    let target_id = resolve_target_project_id(target_is_global, target_project, None, true)?;
    let mut conn = get_db_connection()?;
    let tx = conn.transaction()?;

    let mut source_project_ids = HashSet::new();
    for id in &memory_ids {
        if let Ok(Some(m)) = get_memory(id) {
            source_project_ids.insert(m.project_id);
        }
    }

    let placeholders: Vec<String> = (1..=memory_ids.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "UPDATE memories SET project_id = ?{} WHERE id IN ({})",
        memory_ids.len() + 1,
        placeholders.join(", ")
    );

    let mut stmt = tx.prepare(&sql)?;
    let mut params_vec: Vec<rusqlite::types::Value> = memory_ids
        .into_iter()
        .map(rusqlite::types::Value::Text)
        .collect();
    params_vec.push(rusqlite::types::Value::Text(target_id.clone()));

    let moved_count = stmt.execute(rusqlite::params_from_iter(params_vec))?;
    drop(stmt);

    let now = Utc::now().to_rfc3339();
    source_project_ids.insert(target_id);

    for pid in source_project_ids {
        let count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM memories WHERE project_id = ?1",
            params![pid],
            |r| r.get(0),
        )?;
        tx.execute(
            "UPDATE projects SET memory_count = ?1, last_active = ?2 WHERE id = ?3",
            params![count, now, pid],
        )?;
    }

    tx.commit()?;
    Ok(moved_count)
}

pub fn export_memories_to_json(file_path: &str) -> Result<String, String> {
    let projects = list_projects().map_err(|e| e.to_string())?;
    let conn = get_db_connection().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, project_id, content, tags, metadata, is_permanent, created_at, tokens_estimated FROM memories")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], map_memory_row)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tags_from_content() {
        let tags1 = extract_tags_from_content("**Clean Code Principles:** Always keep functions small.");
        assert!(tags1.contains(&"clean_code_principles".to_string()) || tags1.contains(&"clean_code".to_string()));

        let tags2 = extract_tags_from_content("Use TypeScript and React for UI development");
        assert!(tags2.contains(&"typescript".to_string()));
        assert!(tags2.contains(&"react".to_string()));
    }

    #[test]
    fn test_resolve_target_project_id() {
        let global_id = resolve_target_project_id(true, None, None, false).unwrap();
        assert_eq!(global_id, "global");

        let global_id2 = resolve_target_project_id(false, Some("global"), None, false).unwrap();
        assert_eq!(global_id2, "global");

        let cwd_id = resolve_target_project_id(false, None, None, true).unwrap();
        assert!(!cwd_id.is_empty());
        assert_ne!(cwd_id, "global");
    }

    #[test]
    fn test_add_and_unified_get_memories() {
        let test_item = BatchMemoryItem {
            content: "**Unit Testing Rule:** Always run cargo test before git commit.".to_string(),
            tags: None,
            metadata: None,
            is_permanent: Some(true),
        };

        let added = add_memories(vec![test_item], false, None, None).unwrap();
        assert_eq!(added.len(), 1);
        let mem_id = added[0].id.clone();

        let all_mems = get_memories(None, 100, None, None, false, None, None).unwrap();
        assert!(all_mems.iter().any(|m| m.id == mem_id));

        let searched = get_memories(Some("cargo test"), 10, None, None, false, None, None).unwrap();
        assert!(searched.iter().any(|m| m.id == mem_id));

        let _ = delete_memories(vec![mem_id]);
    }
}
