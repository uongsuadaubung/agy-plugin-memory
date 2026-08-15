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
                            "version": "1.1.0"
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
                    ToolBuilder::new("get_memories")
                        .description("Retrieve or search memories. If query is omitted, retrieves all memories for the current project and global rules. If query is provided, performs full-text FTS5 BM25 search.")
                        .add_string_param("query", "Optional search keyword or query for full-text BM25 search")
                        .add_number_param("limit", 100, "Maximum number of memories to return")
                        .add_array_param("tags", "string", "Filter by tags")
                        .add_bool_param("is_permanent", "Filter permanent rules (true) or short-term memories (false)")
                        .add_bool_param("is_global", "Query only global user memories if true")
                        .add_string_param("project", "Optional project name, ID, or directory path (auto-detects CWD if omitted)")
                        .add_string_param("path", "Optional directory path")
                        .build(),

                    ToolBuilder::new("add_memories")
                        .description("Add or smart-upsert 1 or multiple memory entries. Automatically targets current workspace project, or global if is_global=true.")
                        .add_bool_param("is_global", "Set to true to save as universal global memory/rule across all projects")
                        .add_string_param("project", "Optional project name, ID, or directory path (auto-detects CWD if omitted)")
                        .add_string_param("path", "Optional directory path")
                        .add_object_array_param(
                            "items",
                            ObjectBuilder::new()
                                .string("content", "Memory content text")
                                .string_array("tags", "Optional tags array (auto-extracted if omitted)")
                                .object("metadata", "Optional metadata object")
                                .bool_flag("is_permanent", "Whether memory is permanent (rule/architecture/convention)")
                                .required(&["content"])
                                .build(),
                            "Array of memory entries to add or update"
                        )
                        .required_params(&["items"])
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
                        .description("Delete 1 or multiple memories by memory IDs.")
                        .add_array_param("memory_ids", "string", "List of memory IDs to delete")
                        .required_params(&["memory_ids"])
                        .build(),

                    ToolBuilder::new("toggle_permanence")
                        .description("Update permanence flag for 1 or multiple memories by ID array.")
                        .add_array_param("memory_ids", "string", "List of memory IDs")
                        .add_bool_param("is_permanent", "New permanence state")
                        .required_params(&["memory_ids", "is_permanent"])
                        .build(),

                    ToolBuilder::new("clear_memories")
                        .description("Delete ALL memories for the current project (or global if is_global=true).")
                        .add_bool_param("is_global", "Set to true to clear global memories")
                        .add_string_param("project", "Optional project name, ID, or path")
                        .add_string_param("path", "Optional directory path")
                        .build(),

                    ToolBuilder::new("cleanup")
                        .description("Retention cleanup for short-term memories older than 30 days.")
                        .add_number_param("max_memories", 50, "Maximum short-term memories to retain")
                        .add_number_param("expire_days", 30, "Expiration age in days")
                        .add_bool_param("is_global", "Set to true to clean up global short-term memories")
                        .add_string_param("project", "Optional project name, ID, or path")
                        .add_string_param("path", "Optional directory path")
                        .build(),

                    ToolBuilder::new("link_projects")
                        .description("Link current project to a target project to inherit its permanent rules.")
                        .add_string_param("target_project", "Target project name or directory path to link and inherit rules from")
                        .add_string_param("source_project", "Optional source project name, ID, or path")
                        .add_string_param("path", "Optional directory path")
                        .required_params(&["target_project"])
                        .build(),

                    ToolBuilder::new("list_projects")
                        .description("List all registered projects in the memory database.")
                        .build(),

                    ToolBuilder::new("memory_stats")
                        .description("Get memory database usage statistics (projects, memories, db size).")
                        .build(),

                    ToolBuilder::new("move_memories")
                        .description("Move memories by ID to another project or global.")
                        .add_array_param("memory_ids", "string", "Array of memory IDs to move")
                        .add_bool_param("target_is_global", "Move to global if true")
                        .add_string_param("target_project", "Target project name or directory path")
                        .required_params(&["memory_ids"])
                        .build(),

                    ToolBuilder::new("delete_projects")
                        .description("Delete 1 or multiple projects by name or ID.")
                        .add_array_param("projects", "string", "List of project names or IDs to delete")
                        .required_params(&["projects"])
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
                    "get_memories" => {
                        let query = args.get("query").and_then(Value::as_str);
                        let limit = usize::try_from(args.get("limit").and_then(Value::as_u64).unwrap_or(100)).unwrap_or(100);
                        let is_perm = args.get("is_permanent").and_then(Value::as_bool);
                        let is_global = args.get("is_global").and_then(Value::as_bool).unwrap_or(false);
                        let project_override = args.get("project").and_then(Value::as_str);
                        let path = args.get("path").and_then(Value::as_str);
                        let tags: Option<Vec<String>> = args
                            .get("tags")
                            .and_then(|v| serde_json::from_value(v.clone()).ok());

                        match get_memories(query, limit, tags, is_perm, is_global, project_override, path) {
                            Ok(mems) => mcp_ok_val(&format_memories_for_agent(&mems)),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "add_memories" => {
                        let is_global = args.get("is_global").and_then(Value::as_bool).unwrap_or(false);
                        let project_override = args.get("project").and_then(Value::as_str);
                        let path = args.get("path").and_then(Value::as_str);
                        let items: Vec<BatchMemoryItem> = args
                            .get("items")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();

                        match add_memories(items, is_global, project_override, path) {
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

                    "clear_memories" => {
                        let is_global = args.get("is_global").and_then(Value::as_bool).unwrap_or(false);
                        let project_override = args.get("project").and_then(Value::as_str);
                        let path = args.get("path").and_then(Value::as_str);
                        match clear_memories(is_global, project_override, path) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "deletedCount": count })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "cleanup" => {
                        let is_global = args.get("is_global").and_then(Value::as_bool).unwrap_or(false);
                        let project_override = args.get("project").and_then(Value::as_str);
                        let path = args.get("path").and_then(Value::as_str);
                        let max_mems = usize::try_from(args.get("max_memories").and_then(Value::as_u64).unwrap_or(50)).unwrap_or(50);
                        let expire_days = args.get("expire_days").and_then(Value::as_i64).unwrap_or(30);

                        match cleanup(is_global, project_override, max_mems, expire_days, path) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "deletedCount": count })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "link_projects" => {
                        let target_project = args.get("target_project").and_then(Value::as_str).unwrap_or("");
                        let source_project = args.get("source_project").and_then(Value::as_str);
                        let path = args.get("path").and_then(Value::as_str);
                        match link_projects(target_project, source_project, path) {
                            Ok(p) => mcp_ok_val(&json!({ "id": p.id, "name": p.name, "linked_project_ids": p.linked_project_ids })),
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

                    "memory_stats" => match memory_stats() {
                        Ok(stats) => mcp_ok_val(&json!(stats)),
                        Err(e) => mcp_err_str(e),
                    },

                    "move_memories" => {
                        let memory_ids: Vec<String> = args
                            .get("memory_ids")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let target_is_global = args.get("target_is_global").and_then(Value::as_bool).unwrap_or(false);
                        let target_project = args.get("target_project").and_then(Value::as_str);

                        match move_memories(memory_ids, target_is_global, target_project) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "movedCount": count })),
                            Err(e) => mcp_err_str(e),
                        }
                    }

                    "delete_projects" => {
                        let projects: Vec<String> = args
                            .get("projects")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();

                        match delete_projects(projects) {
                            Ok(count) => mcp_ok_val(&json!({ "success": true, "deletedCount": count })),
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
