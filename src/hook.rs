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

    let mut workspace_paths: Vec<String> = Vec::new();
    if !stdin_buffer.trim().is_empty() {
        if let Ok(payload) = serde_json::from_str::<HookPayload>(&stdin_buffer) {
            if let Some(paths) = payload.workspace_paths {
                workspace_paths = paths;
            }
        }
    }

    let mut projects: Vec<crate::db::ProjectRecord> = Vec::new();
    let mut seen_project_ids = std::collections::HashSet::new();

    if workspace_paths.is_empty() {
        if let Ok(root_path) = find_project_root(None) {
            let root_str = root_path.to_string_lossy();
            if let Ok(proj) = get_or_create_project(None, Some(&root_str), false) {
                if seen_project_ids.insert(proj.id.clone()) {
                    projects.push(proj);
                }
            }
        }
    } else {
        for wp in &workspace_paths {
            if let Ok(root_path) = find_project_root(Some(wp)) {
                let root_str = root_path.to_string_lossy();
                if let Ok(proj) = get_or_create_project(None, Some(&root_str), false) {
                    if seen_project_ids.insert(proj.id.clone()) {
                        projects.push(proj);
                    }
                }
            }
        }
    }

    if projects.is_empty() {
        println!("{}", json!({ "injectSteps": [] }));
        return;
    }

    let global_mems = get_memories("global", MAX_GLOBAL_MEMORIES, None, None, None).unwrap_or_default();

    let mut ctx_text = String::with_capacity(1024);

    if projects.len() == 1 {
        let proj = &projects[0];
        let _ = writeln!(ctx_text, "[Memory Context: {} | Project ID: {}]", proj.name, proj.id);
    } else {
        let names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();
        let ids: Vec<&str> = projects.iter().map(|p| p.id.as_str()).collect();
        let _ = writeln!(
            ctx_text,
            "[Memory Context: {} | Active Project IDs: {}]",
            names.join(", "),
            ids.join(", ")
        );
    }

    if !global_mems.is_empty() {
        ctx_text.push_str("\nGlobal User Rules:\n");
        for m in &global_mems {
            let _ = writeln!(ctx_text, "- {}", m.content);
        }
    }

    let mut total_memories_found = global_mems.len();

    for proj in &projects {
        let perm_mems = get_memories(&proj.id, MAX_PERMANENT_MEMORIES, None, Some(true), Some(&proj.path)).unwrap_or_default();
        let short_term_mems = get_memories(&proj.id, MAX_SHORT_TERM_MEMORIES, None, Some(false), Some(&proj.path)).unwrap_or_default();

        let mut linked_mems_by_proj: Vec<(String, Vec<crate::db::MemoryRecord>)> = Vec::new();
        for linked_id in &proj.linked_project_ids {
            if linked_id != &proj.id && linked_id != "global" && !seen_project_ids.contains(linked_id) {
                let lmems = get_memories(linked_id, MAX_PERMANENT_MEMORIES, None, Some(true), None).unwrap_or_default();
                if !lmems.is_empty() {
                    linked_mems_by_proj.push((linked_id.clone(), lmems));
                }
            }
        }

        total_memories_found += perm_mems.len() + short_term_mems.len() + linked_mems_by_proj.len();

        if !perm_mems.is_empty() {
            if projects.len() > 1 {
                let _ = writeln!(ctx_text, "\nProject Permanent Rules ({}):", proj.name);
            } else {
                ctx_text.push_str("\nProject Permanent Rules:\n");
            }
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
            if projects.len() > 1 {
                let _ = writeln!(ctx_text, "\nProject Short-Term Memories ({}):", proj.name);
            } else {
                ctx_text.push_str("\nProject Short-Term Memories:\n");
            }
            for m in &short_term_mems {
                let _ = writeln!(ctx_text, "- {}", m.content);
            }
        }
    }

    if total_memories_found == 0 {
        println!("{}", json!({ "injectSteps": [] }));
        return;
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
