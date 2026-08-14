use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Write as FmtWrite;
use std::io::{self, Read, Write as IoWrite};

use crate::db::{get_memories, get_or_create_project};
use crate::project::find_project_root;

pub const MAX_GLOBAL_MEMORIES: usize = 1000;
pub const MAX_PERMANENT_MEMORIES: usize = 1000;
pub const MAX_SHORT_TERM_MEMORIES: usize = 50;

#[derive(Debug, Deserialize)]
struct HookPayload {
    #[serde(rename = "workspacePaths")]
    workspace_paths: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct EphemeralStep {
    #[serde(rename = "ephemeralMessage")]
    ephemeral_message: String,
}

#[derive(Debug, Serialize)]
struct HookResponse {
    #[serde(rename = "injectSteps")]
    inject_steps: Vec<EphemeralStep>,
}

pub fn run_hook_mode() {
    let mut stdin_buffer = String::new();
    let _ = io::stdin().read_to_string(&mut stdin_buffer);

    let mut workspace_path = String::new();
    if !stdin_buffer.trim().is_empty() {
        if let Ok(payload) = serde_json::from_str::<HookPayload>(&stdin_buffer) {
            if let Some(paths) = payload.workspace_paths {
                if let Some(first) = paths.first() {
                    workspace_path = first.clone();
                }
            }
        }
    }

    let root_path = match find_project_root(if workspace_path.is_empty() {
        None
    } else {
        Some(&workspace_path)
    }) {
        Ok(p) => p,
        Err(_) => {
            println!("{}", json!({ "injectSteps": [] }));
            return;
        }
    };
    let root_str = root_path.to_string_lossy();

    let project = match get_or_create_project(None, Some(&root_str), false) {
        Ok(p) => p,
        Err(_) => {
            println!("{}", json!({ "injectSteps": [] }));
            return;
        }
    };

    let global_mems = get_memories("global", MAX_GLOBAL_MEMORIES, None, None, None).unwrap_or_default();
    let perm_mems = get_memories(&project.id, MAX_PERMANENT_MEMORIES, None, Some(true), Some(&root_str)).unwrap_or_default();
    let short_term_mems = get_memories(&project.id, MAX_SHORT_TERM_MEMORIES, None, Some(false), Some(&root_str)).unwrap_or_default();

    let mut linked_mems_by_proj: Vec<(String, Vec<crate::db::MemoryRecord>)> = Vec::new();
    for linked_id in &project.linked_project_ids {
        if linked_id != &project.id && linked_id != "global" {
            let lmems = get_memories(linked_id, MAX_PERMANENT_MEMORIES, None, Some(true), None).unwrap_or_default();
            if !lmems.is_empty() {
                linked_mems_by_proj.push((linked_id.clone(), lmems));
            }
        }
    }

    if global_mems.is_empty() && perm_mems.is_empty() && short_term_mems.is_empty() && linked_mems_by_proj.is_empty() {
        println!("{}", json!({ "injectSteps": [] }));
        return;
    }

    let mut ctx_text = String::with_capacity(1024);
    let _ = writeln!(ctx_text, "[Memory Context: {} | Project ID: {}]", project.name, project.id);

    if !global_mems.is_empty() {
        ctx_text.push_str("\nGlobal User Rules:\n");
        for m in &global_mems {
            let _ = writeln!(ctx_text, "- {}", m.content);
        }
    }

    if !perm_mems.is_empty() {
        ctx_text.push_str("\nProject Permanent Rules:\n");
        for m in &perm_mems {
            let _ = writeln!(ctx_text, "- {}", m.content);
        }
    }

    for (linked_id, lmems) in &linked_mems_by_proj {
        let _ = writeln!(ctx_text, "\nLinked Project Rules (from: {}):", linked_id);
        for m in lmems {
            let _ = writeln!(ctx_text, "- {}", m.content);
        }
    }

    if !short_term_mems.is_empty() {
        ctx_text.push_str("\nProject Short-Term Memories:\n");
        for m in &short_term_mems {
            let _ = writeln!(ctx_text, "- {}", m.content);
        }
    }

    let resp = HookResponse {
        inject_steps: vec![EphemeralStep {
            ephemeral_message: ctx_text,
        }],
    };

    if let Ok(s) = serde_json::to_string(&resp) {
        println!("{}", s);
        let _ = io::stdout().flush();
    }
}
