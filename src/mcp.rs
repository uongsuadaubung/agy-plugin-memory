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

struct ObjectBuilder {
    properties: Value,
    required: Vec<&'static str>,
}

impl ObjectBuilder {
    fn new() -> Self {
        Self {
            properties: json!({}),
            required: Vec::new(),
        }
    }

    fn string(mut self, name: &'static str, desc: &'static str) -> Self {
        self.properties[name] = json!({ "type": "string", "description": desc });
        self
    }

    fn bool_flag(mut self, name: &'static str, desc: &'static str) -> Self {
        self.properties[name] = json!({ "type": "boolean", "description": desc });
        self
    }

    fn string_array(mut self, name: &'static str, desc: &'static str) -> Self {
        self.properties[name] = json!({
            "type": "array",
            "items": { "type": "string" },
            "description": desc
        });
        self
    }

    fn object(mut self, name: &'static str, desc: &'static str) -> Self {
        self.properties[name] = json!({ "type": "object", "description": desc });
        self
    }

    fn required(mut self, req_fields: &[&'static str]) -> Self {
        self.required.extend_from_slice(req_fields);
        self
    }

    fn build(self) -> Value {
        let mut obj = json!({
            "type": "object",
            "properties": self.properties
        });
        if !self.required.is_empty() {
            obj["required"] = json!(self.required);
        }
        obj
    }
}

struct ToolBuilder {
    name: &'static str,
    description: &'static str,
    schema: Value,
}

impl ToolBuilder {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            description: "",
            schema: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    fn description(mut self, desc: &'static str) -> Self {
        self.description = desc;
        self
    }

    fn add_string_param(mut self, param_name: &'static str, desc: &'static str) -> Self {
        self.schema["properties"][param_name] = json!({ "type": "string", "description": desc });
        self
    }

    fn add_number_param(mut self, param_name: &'static str, default_val: u64, desc: &'static str) -> Self {
        self.schema["properties"][param_name] = json!({ "type": "number", "default": default_val, "description": desc });
        self
    }

    fn add_bool_param(mut self, param_name: &'static str, desc: &'static str) -> Self {
        self.schema["properties"][param_name] = json!({ "type": "boolean", "description": desc });
        self
    }

    fn add_array_param(mut self, param_name: &'static str, item_type: &'static str, desc: &'static str) -> Self {
        self.schema["properties"][param_name] = json!({
            "type": "array",
            "items": { "type": item_type },
            "description": desc
        });
        self
    }

    fn add_object_array_param(mut self, param_name: &'static str, item_schema: Value, desc: &'static str) -> Self {
        self.schema["properties"][param_name] = json!({
            "type": "array",
            "items": item_schema,
            "description": desc
        });
        self
    }

    fn required_params(mut self, req_fields: &[&'static str]) -> Self {
        self.schema["required"] = json!(req_fields);
        self
    }

    fn build(self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": self.schema,
        })
    }
}

pub fn run_mcp_mode() {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line_buf = String::with_capacity(4096);

    while handle.read_line(&mut line_buf).unwrap_or(0) > 0 {
        let trimmed = line_buf.trim();
        if trimmed.is_empty() {
            line_buf.clear();
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(_) => {
                line_buf.clear();
                continue;
            }
        };

        line_buf.clear();

        if req.method.starts_with("notifications/") {
            continue;
        }

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

            "ping" => {
                let resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req_id,
                    result: Some(json!({})),
                    error: None,
                };
                send_response(&resp);
            }

            "tools/list" => {
                let tools_list = vec![
                    ToolBuilder::new("get_project")
                        .description("Get an existing project by name/path or create a new one.")
                        .add_string_param("name", "Project name")
                        .add_string_param("path", "Project directory path")
                        .required_params(&["name"])
                        .build(),

                    ToolBuilder::new("list_projects")
                        .description("List all registered projects in the memory database.")
                        .build(),

                    ToolBuilder::new("clear_memories")
                        .description("Delete ALL memories (both permanent and short-term) for a project.")
                        .add_string_param("project_id", "Unique Project ID")
                        .add_string_param("path", "Optional project directory path")
                        .required_params(&["project_id"])
                        .build(),

                    ToolBuilder::new("delete_projects")
                        .description("Delete 1 or multiple projects by an array of project IDs.")
                        .add_array_param("project_ids", "string", "List of project IDs to delete")
                        .required_params(&["project_ids"])
                        .build(),

                    ToolBuilder::new("add_memories")
                        .description("Add or smart-upsert 1 or multiple memory entries at once.")
                        .add_string_param("project_id", "Unique Project ID")
                        .add_string_param("path", "Optional project path")
                        .add_object_array_param(
                            "items",
                            ObjectBuilder::new()
                                .string("content", "Memory content text")
                                .string_array("tags", "Optional tags array")
                                .object("metadata", "Optional metadata object")
                                .bool_flag("is_permanent", "Whether memory is permanent")
                                .required(&["content"])
                                .build(),
                            "Array of memory entries to add or update"
                        )
                        .required_params(&["project_id", "items"])
                        .build(),

                    ToolBuilder::new("get_memories")
                        .description("Retrieve valid stored memories for a project.")
                        .add_string_param("project_id", "Unique Project ID")
                        .add_number_param("limit", 100, "Maximum number of memories to return")
                        .add_array_param("tags", "string", "Filter tags")
                        .add_bool_param("is_permanent", "Filter permanent or short-term")
                        .add_string_param("path", "Optional project path")
                        .required_params(&["project_id"])
                        .build(),

                    ToolBuilder::new("search_memories")
                        .description("FTS5 Full-Text BM25 relevance search across memories content and tags.")
                        .add_string_param("project_id", "Unique Project ID")
                        .add_string_param("query", "Search query / keyword")
                        .add_number_param("limit", 100, "Maximum results")
                        .add_string_param("path", "Optional project path")
                        .required_params(&["project_id", "query"])
                        .build(),

                    ToolBuilder::new("get_memory")
                        .description("Retrieve a single memory record by its unique memory ID.")
                        .add_string_param("memory_id", "Memory ID to inspect")
                        .required_params(&["memory_id"])
                        .build(),

                    ToolBuilder::new("update_memory")
                        .description("Directly update an existing memory record's content, tags, metadata, or permanence by memory ID.")
                        .add_string_param("memory_id", "Unique memory ID to update")
                        .add_string_param("content", "Optional new memory content text")
                        .add_array_param("tags", "string", "Optional new tags array")
                        .add_bool_param("is_permanent", "Optional new permanence state")
                        .required_params(&["memory_id"])
                        .build(),

                    ToolBuilder::new("delete_memories")
                        .description("Delete 1 or multiple memories by an array of memory IDs.")
                        .add_array_param("memory_ids", "string", "List of memory IDs to delete")
                        .required_params(&["memory_ids"])
                        .build(),

                    ToolBuilder::new("toggle_permanence")
                        .description("Update permanence flag for 1 or multiple memories by ID array.")
                        .add_array_param("memory_ids", "string", "List of memory IDs")
                        .add_bool_param("is_permanent", "New permanence state")
                        .required_params(&["memory_ids", "is_permanent"])
                        .build(),

                    ToolBuilder::new("memory_stats")
                        .description("Get memory database usage statistics (projects, memories, db size).")
                        .build(),

                    ToolBuilder::new("cleanup")
                        .description("Retention cleanup for short-term memories older than 30 days.")
                        .add_string_param("project_id", "Unique Project ID")
                        .add_number_param("max_memories", 50, "Maximum short-term memories to retain")
                        .add_number_param("expire_days", 30, "Expiration age in days")
                        .required_params(&["project_id"])
                        .build(),

                    ToolBuilder::new("link_projects")
                        .description("Link current project to 1 or more target projects to inherit their permanent rules.")
                        .add_string_param("project_id", "Current Project ID")
                        .add_array_param("target_project_ids", "string", "Array of target project IDs to link")
                        .add_string_param("path", "Optional directory path")
                        .required_params(&["project_id", "target_project_ids"])
                        .build(),

                    ToolBuilder::new("project_links")
                        .description("Get linked project IDs for a project.")
                        .add_string_param("project_id", "Project ID")
                        .required_params(&["project_id"])
                        .build(),

                    ToolBuilder::new("move_memories")
                        .description("Move 1 or multiple memories by ID array to another target project (e.g. workspace project ID or 'global'). Automatically resolves and updates source project counts.")
                        .add_array_param("memory_ids", "string", "Array of memory IDs to move")
                        .add_string_param("target_project_id", "Target project ID to move memories into (e.g. workspace project ID or 'global')")
                        .required_params(&["memory_ids", "target_project_id"])
                        .build(),
                ];

                let resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req_id,
                    result: Some(json!({ "tools": tools_list })),
                    error: None,
                };
                send_response(&resp);
            }

            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                let tool_result = match tool_name {
                    "get_project" => {
                        let name = args.get("name").and_then(|v| v.as_str());
                        let path = args.get("path").and_then(|v| v.as_str());
                        match get_project(name, path, true) {
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

                    "clear_memories" => {
                        let project_id = args.get("project_id").and_then(|v| v.as_str()).unwrap_or("");
                        let path = args.get("path").and_then(|v| v.as_str());
                        match clear_memories(project_id, path) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "deletedCount": count })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "delete_projects" => {
                        let project_ids: Vec<String> = args
                            .get("project_ids")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();

                        match delete_projects(project_ids) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "deletedCount": count })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "add_memories" => {
                        let project_id = args.get("project_id").and_then(|v| v.as_str()).unwrap_or("");
                        let path = args.get("path").and_then(|v| v.as_str());
                        let items: Vec<BatchMemoryItem> = args
                            .get("items")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();

                        match add_memories(project_id, items, path) {
                            Ok(mems) => mcp_ok_val(&format_memories_for_agent(&mems)),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "get_memories" => {
                        let project_id = args.get("project_id").and_then(Value::as_str).unwrap_or("");
                        let limit = usize::try_from(args.get("limit").and_then(Value::as_u64).unwrap_or(100)).unwrap_or(100);
                        let path = args.get("path").and_then(Value::as_str);
                        let is_perm = args.get("is_permanent").and_then(Value::as_bool);

                        let tags: Option<Vec<String>> = args
                            .get("tags")
                            .and_then(|v| serde_json::from_value(v.clone()).ok());

                        match get_memories(project_id, limit, tags, is_perm, path) {
                            Ok(mems) => mcp_ok_val(&format_memories_for_agent(&mems)),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "search_memories" => {
                        let project_id = args.get("project_id").and_then(Value::as_str).unwrap_or("");
                        let query = args.get("query").and_then(Value::as_str).unwrap_or("");
                        let limit = usize::try_from(args.get("limit").and_then(Value::as_u64).unwrap_or(100)).unwrap_or(100);
                        let path = args.get("path").and_then(Value::as_str);

                        match search_memories(project_id, query, limit, path) {
                            Ok(mems) => mcp_ok_val(&format_memories_for_agent(&mems)),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "get_memory" => {
                        let memory_id = args.get("memory_id").and_then(Value::as_str).unwrap_or("");
                        match get_memory(memory_id) {
                            Ok(Some(m)) => mcp_ok_val(&format_memory_for_agent(&m)),
                            Ok(None) => mcp_err_str("Memory ID not found"),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "update_memory" => {
                        let memory_id = args.get("memory_id").and_then(Value::as_str).unwrap_or("");
                        let content = args.get("content").and_then(Value::as_str);
                        let tags: Option<Vec<String>> = args
                            .get("tags")
                            .and_then(|v| serde_json::from_value(v.clone()).ok());
                        let metadata: Option<Value> = args.get("metadata").cloned();
                        let is_perm = args.get("is_permanent").and_then(Value::as_bool);

                        match update_memory(memory_id, content, tags, metadata, is_perm) {
                            Ok(Some(m)) => mcp_ok_val(&format_memory_for_agent(&m)),
                            Ok(None) => mcp_err_str(format!("Memory ID '{memory_id}' not found")),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "delete_memories" => {
                        let memory_ids: Vec<String> = args
                            .get("memory_ids")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();

                        match delete_memories(memory_ids) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "deletedCount": count })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "toggle_permanence" => {
                        let memory_ids: Vec<String> = args
                            .get("memory_ids")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let is_perm = args.get("is_permanent").and_then(Value::as_bool).unwrap_or(false);

                        match toggle_permanence(memory_ids, is_perm) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "updatedCount": count, "is_permanent": is_perm })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "memory_stats" => match memory_stats() {
                        Ok(stats) => mcp_ok_val(&json!(stats)),
                        Err(e) => mcp_err_str(e),
                    },

                    "cleanup" => {
                        let project_id = args.get("project_id").and_then(Value::as_str).unwrap_or("");
                        let max_mems = usize::try_from(args.get("max_memories").and_then(Value::as_u64).unwrap_or(50)).unwrap_or(50);
                        let expire_days = args.get("expire_days").and_then(Value::as_i64).unwrap_or(30);

                        match cleanup(project_id, max_mems, expire_days, None) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "deletedCount": count })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "link_projects" => {
                        let project_id = args.get("project_id").and_then(Value::as_str).unwrap_or("");
                        let target_project_ids: Vec<String> = args.get("target_project_ids")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let path = args.get("path").and_then(Value::as_str);

                        match link_projects(project_id, target_project_ids, path) {
                            Ok(p) => mcp_ok_val(&json!({ "id": p.id, "linked_project_ids": p.linked_project_ids })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "project_links" => {
                        let project_id = args.get("project_id").and_then(Value::as_str).unwrap_or("");
                        let linked = get_linked_project_ids(project_id);
                        mcp_ok_val(&json!({ "project_id": project_id, "linked_project_ids": linked }))
                    }

                    "move_memories" => {
                        let memory_ids: Vec<String> = args.get("memory_ids")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let target_project_id = args.get("target_project_id").and_then(Value::as_str).unwrap_or("");

                        match move_memories(memory_ids, target_project_id) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "movedCount": count, "target_project_id": target_project_id })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    _ => mcp_err_str(format!("Unknown tool: {tool_name}")),
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
