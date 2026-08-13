use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{self, Read};

use crate::db::{get_memories, get_or_create_project};
use crate::project::find_project_root;

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

    let root_path = find_project_root(if workspace_path.is_empty() {
        None
    } else {
        Some(&workspace_path)
    });
    let root_str = root_path.to_string_lossy();

    let project = match get_or_create_project(None, Some(&root_str), false) {
        Ok(p) => p,
        Err(_) => {
            println!("{}", json!({ "injectSteps": [] }));
            return;
        }
    };

    // Load ALL permanent rules + top 5 most recent short-term progress memories for token efficiency
    let global_mems = get_memories("global", 100, None, Some(true), None).unwrap_or_default();
    let perm_mems = get_memories(&project.id, 100, None, Some(true), Some(&root_str)).unwrap_or_default();
    let recent_mems = get_memories(&project.id, 5, None, Some(false), Some(&root_str)).unwrap_or_default();

    if global_mems.is_empty() && perm_mems.is_empty() && recent_mems.is_empty() {
        println!("{}", json!({ "injectSteps": [] }));
        return;
    }

    let mut ctx_text = format!("🧠 [Memory Context: {}]\n", project.name);

    if !global_mems.is_empty() {
        ctx_text.push_str("\n🌐 Global User Rules:\n");
        for m in &global_mems {
            ctx_text.push_str(&format!("• {}\n", m.content));
        }
    }

    if !perm_mems.is_empty() {
        ctx_text.push_str("\n📌 Project Permanent Rules:\n");
        for m in &perm_mems {
            ctx_text.push_str(&format!("• {}\n", m.content));
        }
    }

    if !recent_mems.is_empty() {
        ctx_text.push_str("\n🕒 Recent Progress:\n");
        for m in &recent_mems {
            ctx_text.push_str(&format!("• {}\n", m.content));
        }
    }

    let resp = HookResponse {
        inject_steps: vec![EphemeralStep {
            ephemeral_message: ctx_text,
        }],
    };

    println!("{}", serde_json::to_string(&resp).unwrap());
}
