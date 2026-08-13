use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::db::*;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn send_response(resp: &JsonRpcResponse) {
    if let Ok(s) = serde_json::to_string(resp) {
        println!("{}", s);
        let _ = io::stdout().flush();
    }
}

fn format_memory_for_agent(m: &MemoryRecord) -> Value {
    let mut obj = json!({
        "id": m.id,
        "content": m.content,
        "is_permanent": m.is_permanent
    });
    if !m.tags.is_empty() {
        obj["tags"] = json!(m.tags);
    }
    if !m.metadata.is_null() && m.metadata != json!({}) {
        obj["metadata"] = m.metadata.clone();
    }
    obj
}

fn format_memories_for_agent(mems: &[MemoryRecord]) -> Value {
    let list: Vec<Value> = mems.iter().map(format_memory_for_agent).collect();
    json!({ "count": list.len(), "memories": list })
}

fn mcp_ok_val(val: &Value) -> Value {
    json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string(val).unwrap_or_else(|_| "{}".to_string())
        }]
    })
}

fn mcp_err_str(err: impl std::fmt::Display) -> Value {
    json!({
        "isError": true,
        "content": [{
            "type": "text",
            "text": format!("Error: {}", err)
        }]
    })
}

pub fn run_mcp_mode() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    while let Some(Ok(line)) = lines.next() {
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let req_id = req.id.unwrap_or(Value::Null);

        match req.method.as_str() {
            "initialize" => {
                let resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req_id,
                    result: Some(json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": { "listChanged": true }
                        },
                        "serverInfo": {
                            "name": "apm-mcp-rust",
                            "version": "1.0.0"
                        }
                    })),
                    error: None,
                };
                send_response(&resp);
            }

            "notifications/initialized" => {}

            "tools/list" => {
                let tools = json!({
                    "tools": [
                        {
                            "name": "get_or_create_project",
                            "description": "Auto-detect current working directory project or get/create a project by name or path.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string", "description": "Optional project display name" },
                                    "path": { "type": "string", "description": "Optional absolute path to project directory" }
                                }
                            }
                        },
                        {
                            "name": "list_projects",
                            "description": "List all registered projects in the memory database.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        },
                        {
                            "name": "clear_project_memories",
                            "description": "Delete ALL memories (both permanent and short-term) for a project.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_id": { "type": "string", "description": "Unique Project ID" },
                                    "path": { "type": "string", "description": "Optional project directory path" }
                                },
                                "required": ["project_id"]
                            }
                        },
                        {
                            "name": "batch_delete_projects",
                            "description": "Batch delete 1 or multiple projects by an array of project IDs.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_ids": { "type": "array", "items": { "type": "string" }, "description": "List of project IDs to delete" }
                                },
                                "required": ["project_ids"]
                            }
                        },
                        {
                            "name": "batch_add_memories",
                            "description": "Add or smart-upsert 1 or multiple memory entries at once.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_id": { "type": "string", "description": "Unique Project ID" },
                                    "items": {
                                        "type": "array",
                                        "items": {
                                            "type": "object",
                                            "properties": {
                                                "content": { "type": "string" },
                                                "tags": { "type": "array", "items": { "type": "string" } },
                                                "metadata": { "type": "object" },
                                                "is_permanent": { "type": "boolean" }
                                            },
                                            "required": ["content"]
                                        }
                                    },
                                    "path": { "type": "string", "description": "Optional project path" }
                                },
                                "required": ["project_id", "items"]
                            }
                        },
                        {
                            "name": "get_memories",
                            "description": "Retrieve valid stored memories for a project.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_id": { "type": "string", "description": "Unique Project ID" },
                                    "limit": { "type": "number", "default": 100, "description": "Maximum number of memories to return" },
                                    "tags": { "type": "array", "items": { "type": "string" }, "description": "Filter tags" },
                                    "is_permanent": { "type": "boolean", "description": "Filter permanent or short-term" },
                                    "path": { "type": "string", "description": "Optional project path" }
                                },
                                "required": ["project_id"]
                            }
                        },
                        {
                            "name": "search_memories",
                            "description": "FTS5 Full-Text BM25 relevance search across memories content and tags.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_id": { "type": "string", "description": "Unique Project ID" },
                                    "query": { "type": "string", "description": "Search query / keyword" },
                                    "limit": { "type": "number", "default": 100, "description": "Maximum results" },
                                    "path": { "type": "string", "description": "Optional project path" }
                                },
                                "required": ["project_id", "query"]
                            }
                        },
                        {
                            "name": "get_memory_by_id",
                            "description": "Retrieve a single memory record by its unique memory ID.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "memory_id": { "type": "string", "description": "Memory ID to inspect" }
                                },
                                "required": ["memory_id"]
                            }
                        },
                        {
                            "name": "batch_delete_memories",
                            "description": "Batch delete 1 or multiple memories by an array of memory IDs.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "memory_ids": { "type": "array", "items": { "type": "string" }, "description": "List of memory IDs to delete" }
                                },
                                "required": ["memory_ids"]
                            }
                        },
                        {
                            "name": "batch_toggle_permanence",
                            "description": "Batch update permanence flag for 1 or multiple memories by ID array.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "memory_ids": { "type": "array", "items": { "type": "string" } },
                                    "is_permanent": { "type": "boolean" }
                                },
                                "required": ["memory_ids", "is_permanent"]
                            }
                        },
                        {
                            "name": "get_memory_stats",
                            "description": "Get memory database usage statistics (projects, memories, db size).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        },
                        {
                            "name": "cleanup_expired",
                            "description": "Retention cleanup for short-term memories older than 30 days.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_id": { "type": "string", "description": "Unique Project ID" },
                                    "max_memories": { "type": "number", "default": 50 },
                                    "expire_days": { "type": "number", "default": 30 }
                                },
                                "required": ["project_id"]
                            }
                        },
                        {
                            "name": "link_projects",
                            "description": "Link current project to 1 or more target projects to inherit their permanent rules.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_id": { "type": "string", "description": "Current Project ID" },
                                    "target_project_ids": { "type": "array", "items": { "type": "string" }, "description": "Array of target project IDs to link" },
                                    "path": { "type": "string", "description": "Optional directory path" }
                                },
                                "required": ["project_id", "target_project_ids"]
                            }
                        },
                        {
                            "name": "get_project_links",
                            "description": "Get linked project IDs for a project.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_id": { "type": "string", "description": "Project ID" }
                                },
                                "required": ["project_id"]
                            }
                        },
                        {
                            "name": "move_memories",
                            "description": "Move 1 or multiple memories by ID array to another target project (e.g. workspace project ID or 'global'). Automatically resolves and updates source project counts.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "memory_ids": { "type": "array", "items": { "type": "string" }, "description": "Array of memory IDs to move" },
                                    "target_project_id": { "type": "string", "description": "Target project ID to move memories into (e.g. workspace project ID or 'global')" }
                                },
                                "required": ["memory_ids", "target_project_id"]
                            }
                        }
                    ]
                });

                let resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req_id,
                    result: Some(tools),
                    error: None,
                };
                send_response(&resp);
            }

            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                let tool_result = match tool_name {
                    "get_or_create_project" => {
                        let name = args.get("name").and_then(|v| v.as_str());
                        let path = args.get("path").and_then(|v| v.as_str());
                        match get_or_create_project(name, path, true) {
                            Ok(p) => mcp_ok_val(&json!({ "id": p.id, "name": p.name, "memory_count": p.memory_count, "linked_project_ids": p.linked_project_ids })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "list_projects" => match list_projects() {
                        Ok(projs) => {
                            let simple: Vec<Value> = projs
                                .iter()
                                .map(|p| json!({ "id": p.id, "name": p.name, "memory_count": p.memory_count }))
                                .collect();
                            mcp_ok_val(&json!(simple))
                        }
                        Err(e) => mcp_err_str(e),
                    },

                    "clear_project_memories" => {
                        let project_id = args.get("project_id").and_then(|v| v.as_str()).unwrap_or("");
                        let path = args.get("path").and_then(|v| v.as_str());
                        match clear_project_memories(project_id, path) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "deletedCount": count })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "batch_delete_projects" => {
                        let project_ids: Vec<String> = args
                            .get("project_ids")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();

                        match batch_delete_projects(project_ids) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "deletedCount": count })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "batch_add_memories" => {
                        let project_id = args.get("project_id").and_then(|v| v.as_str()).unwrap_or("");
                        let path = args.get("path").and_then(|v| v.as_str());
                        let items: Vec<BatchMemoryItem> = args
                            .get("items")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();

                        match batch_add_memories(project_id, items, path) {
                            Ok(mems) => mcp_ok_val(&format_memories_for_agent(&mems)),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "get_memories" => {
                        let project_id = args.get("project_id").and_then(|v| v.as_str()).unwrap_or("");
                        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
                        let path = args.get("path").and_then(|v| v.as_str());
                        let is_perm = args.get("is_permanent").and_then(|v| v.as_bool());

                        let tags: Option<Vec<String>> = args
                            .get("tags")
                            .and_then(|v| serde_json::from_value(v.clone()).ok());

                        match get_memories(project_id, limit, tags, is_perm, path) {
                            Ok(mems) => mcp_ok_val(&format_memories_for_agent(&mems)),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "search_memories" => {
                        let project_id = args.get("project_id").and_then(|v| v.as_str()).unwrap_or("");
                        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
                        let path = args.get("path").and_then(|v| v.as_str());

                        match search_memories(project_id, query, limit, path) {
                            Ok(mems) => mcp_ok_val(&format_memories_for_agent(&mems)),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "get_memory_by_id" => {
                        let memory_id = args.get("memory_id").and_then(|v| v.as_str()).unwrap_or("");
                        match get_memory_by_id(memory_id) {
                            Ok(Some(m)) => mcp_ok_val(&format_memory_for_agent(&m)),
                            Ok(None) => mcp_err_str("Memory ID not found"),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "batch_delete_memories" => {
                        let memory_ids: Vec<String> = args
                            .get("memory_ids")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();

                        match batch_delete_memories(memory_ids) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "deletedCount": count })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "batch_toggle_permanence" => {
                        let memory_ids: Vec<String> = args
                            .get("memory_ids")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let is_perm = args.get("is_permanent").and_then(|v| v.as_bool()).unwrap_or(false);

                        match batch_toggle_permanence(memory_ids, is_perm) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "updatedCount": count, "is_permanent": is_perm })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "get_memory_stats" => match get_memory_stats() {
                        Ok(stats) => mcp_ok_val(&json!(stats)),
                        Err(e) => mcp_err_str(e),
                    },

                    "cleanup_expired" => {
                        let project_id = args.get("project_id").and_then(|v| v.as_str()).unwrap_or("");
                        let max_mems = args.get("max_memories").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
                        let expire_days = args.get("expire_days").and_then(|v| v.as_i64()).unwrap_or(30);

                        match cleanup_expired(project_id, max_mems, expire_days, None) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "deletedCount": count })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "link_projects" => {
                        let project_id = args.get("project_id").and_then(|v| v.as_str()).unwrap_or("");
                        let target_project_ids: Vec<String> = args.get("target_project_ids")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let path = args.get("path").and_then(|v| v.as_str());

                        match link_projects(project_id, target_project_ids, path) {
                            Ok(p) => mcp_ok_val(&json!({ "id": p.id, "linked_project_ids": p.linked_project_ids })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "get_project_links" => {
                        let project_id = args.get("project_id").and_then(|v| v.as_str()).unwrap_or("");
                        let linked = get_linked_project_ids(project_id);
                        mcp_ok_val(&json!({ "project_id": project_id, "linked_project_ids": linked }))
                    }

                    "move_memories" => {
                        let memory_ids: Vec<String> = args.get("memory_ids")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let target_project_id = args.get("target_project_id").and_then(|v| v.as_str()).unwrap_or("");

                        match move_memories(memory_ids, target_project_id) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "movedCount": count, "target_project_id": target_project_id })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    _ => mcp_err_str(format!("Unknown tool: {}", tool_name)),
                };

                let resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req_id,
                    result: Some(tool_result),
                    error: None,
                };
                send_response(&resp);
            }

            _ => {
                let resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req_id,
                    result: None,
                    error: Some(json!({ "code": -32601, "message": "Method not found" })),
                };
                send_response(&resp);
            }
        }
    }
}
