use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Write as FmtWrite;
use std::io::{self, Read, Write as IoWrite};

use crate::db::{get_memories, get_project};
use crate::project::find_project_root;

pub const MAX_GLOBAL_MEMORIES: usize = 1000;
pub const MAX_PERMANENT_MEMORIES: usize = 1000;
pub const MAX_SHORT_TERM_MEMORIES: usize = 50;

#[derive(Debug, Deserialize)]
struct HookPayload {
    #[serde(rename = "workspacePaths")]
    workspace_paths: Option<Vec<String>>,
    #[serde(rename = "userPrompt")]
    user_prompt: Option<String>,
    #[serde(rename = "prompt")]
    prompt: Option<String>,
    #[serde(rename = "conversationId")]
    conversation_id: Option<String>,
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
    let mut input_prompt: Option<String> = None;
    let mut input_conversation_id: Option<String> = None;

    if !stdin_buffer.trim().is_empty() {
        if let Ok(payload) = serde_json::from_str::<HookPayload>(&stdin_buffer) {
            if let Some(paths) = payload.workspace_paths {
                workspace_paths = paths;
            }
            input_prompt = payload.user_prompt.or(payload.prompt);
            input_conversation_id = payload.conversation_id;
        }
    }

    let mut projects: Vec<crate::db::ProjectRecord> = Vec::new();
    let mut seen_project_ids = std::collections::HashSet::new();

    if workspace_paths.is_empty() {
        if let Ok(root_path) = find_project_root(None) {
            let root_str = root_path.to_string_lossy();
            if let Ok(proj) = get_project(None, Some(&root_str), true) {
                if !seen_project_ids.contains(&proj.id) {
                    seen_project_ids.insert(proj.id.clone());
                    projects.push(proj);
                }
            }
        }
    } else {
        for wp in &workspace_paths {
            if let Ok(root_path) = find_project_root(Some(wp)) {
                let root_str = root_path.to_string_lossy();
                if let Ok(proj) = get_project(None, Some(&root_str), true) {
                    if !seen_project_ids.contains(&proj.id) {
                        seen_project_ids.insert(proj.id.clone());
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

    let parent_pid = crate::process::get_parent_pid();
    if let Some(primary_proj) = projects.first() {
        let _ = crate::db::set_active_workspace(
            &primary_proj.path,
            &primary_proj.id,
            &primary_proj.name,
            parent_pid,
            input_conversation_id.as_deref(),
        );
    }

    let global_mems = get_memories(None, MAX_GLOBAL_MEMORIES, None, Some(true), true, None, None).unwrap_or_default();

    let mut ctx_text = String::with_capacity(1024);

    let proj_names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();
    let _ = writeln!(ctx_text, "[Memory Active: {} | Global: Active]", proj_names.join(", "));
    let _ = writeln!(
        ctx_text,
        "MEMORY DIRECTIVE: Follow all active project and global rules strictly. To query or search more memories, call get_memories(query=...). To save new rules, call add_memories(items=[...])."
    );

    if !global_mems.is_empty() {
        ctx_text.push_str("\nGlobal User Rules:\n");
        for m in &global_mems {
            let _ = writeln!(ctx_text, "- {}", m.content);
        }
    }

    for proj in &projects {
        let perm_mems = get_memories(None, MAX_PERMANENT_MEMORIES, None, Some(true), false, None, Some(&proj.path)).unwrap_or_default();
        let short_term_mems = get_memories(None, MAX_SHORT_TERM_MEMORIES, None, Some(false), false, None, Some(&proj.path)).unwrap_or_default();

        let mut linked_mems_by_proj: Vec<(String, Vec<crate::db::MemoryRecord>)> = Vec::new();
        for linked_id in &proj.linked_project_ids {
            if linked_id != &proj.id && linked_id != "global" && !seen_project_ids.contains(linked_id) {
                if let Ok(lmems) = get_memories(None, MAX_PERMANENT_MEMORIES, None, Some(true), false, Some(linked_id), None) {
                    if !lmems.is_empty() {
                        linked_mems_by_proj.push((linked_id.clone(), lmems));
                    }
                }
            }
        }

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

    if let Some(ref prompt_str) = input_prompt {
        let clean_prompt = prompt_str.trim();
        if clean_prompt.len() >= 3 {
            let mut matched_mems = Vec::new();
            for proj in &projects {
                if let Ok(mems) = get_memories(Some(clean_prompt), 5, None, None, false, None, Some(&proj.path)) {
                    matched_mems.extend(mems);
                }
            }

            if !matched_mems.is_empty() {
                let mut added_ids = std::collections::HashSet::new();
                let mut relevant_text = String::new();

                for m in matched_mems {
                    if added_ids.insert(m.id.clone()) {
                        let _ = writeln!(relevant_text, "- [{}] {}", m.id, m.content);
                    }
                }

                if !relevant_text.is_empty() {
                    ctx_text.push_str("\nRelevant Rules (Prompt Match):\n");
                    ctx_text.push_str(&relevant_text);
                }
            }
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
