use std::env;
use std::fs;

// Embed plugin files directly into binary at compile time via include_str!
const PLUGIN_JSON: &str = include_str!("../plugins/apm-mcp/plugin.json");
const MCP_CONFIG_JSON: &str = include_str!("../plugins/apm-mcp/mcp_config.json");
const HOOKS_JSON: &str = include_str!("../plugins/apm-mcp/hooks.json");
const RULES_MEMORY_MD: &str = include_str!("../plugins/apm-mcp/rules/memory.md");
const INSTRUCTIONS_MEMORY_MD: &str = include_str!("../plugins/apm-mcp/instructions/memory.md");
const WORKFLOWS_INIT_MD: &str = include_str!("../plugins/apm-mcp/workflows/init-apm.md");
const WORKFLOWS_MEMORY_MD: &str = include_str!("../plugins/apm-mcp/workflows/memory.md");

pub fn run_install_mode() {
    let mut plugin_dir = match dirs::home_dir() {
        Some(h) => h,
        None => {
            println!("[ERROR] Could not resolve home directory.");
            return;
        }
    };

    plugin_dir.push(".gemini");
    plugin_dir.push("config");
    plugin_dir.push("plugins");
    plugin_dir.push("apm-mcp");

    // 0. Clean existing plugin directory before installation to remove legacy/unregistered files
    if plugin_dir.exists() {
        let curr_exe = env::current_exe().ok();
        if let Ok(entries) = fs::read_dir(&plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ref exe) = curr_exe {
                    if path == *exe {
                        continue;
                    }
                }
                if path.is_dir() {
                    let _ = fs::remove_dir_all(&path);
                } else {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }

    let bin_dir = plugin_dir.join("bin");
    let rules_dir = plugin_dir.join("rules");
    let instructions_dir = plugin_dir.join("instructions");
    let workflows_dir = plugin_dir.join("workflows");

    let _ = fs::create_dir_all(&bin_dir);
    let _ = fs::create_dir_all(&rules_dir);
    let _ = fs::create_dir_all(&instructions_dir);
    let _ = fs::create_dir_all(&workflows_dir);

    #[cfg(target_os = "windows")]
    let target_exe = bin_dir.join("apm-mcp.exe");
    #[cfg(not(target_os = "windows"))]
    let target_exe = bin_dir.join("apm-mcp");

    // 1. Copy the current running binary into target plugin bin/ directory
    if let Ok(curr_exe) = env::current_exe() {
        if curr_exe != target_exe {
            let _ = fs::copy(&curr_exe, &target_exe);
        }
    }

    // 2. Extract embedded plugin files to target directory, replacing placeholder command with absolute binary path
    let target_exe_str = target_exe.to_string_lossy().replace('\\', "/");

    let mcp_config = MCP_CONFIG_JSON.replace(
        "\"command\": \"apm-mcp\"",
        &format!("\"command\": \"{}\"", target_exe_str),
    );

    let hooks_config = HOOKS_JSON.replace(
        "\"command\": \"apm-mcp hook\"",
        &format!("\"command\": \"\\\"{}\\\" hook\"", target_exe_str),
    );

    let _ = fs::write(plugin_dir.join("plugin.json"), PLUGIN_JSON);
    let _ = fs::write(plugin_dir.join("mcp_config.json"), mcp_config);
    let _ = fs::write(plugin_dir.join("hooks.json"), hooks_config);
    let _ = fs::write(rules_dir.join("memory.md"), RULES_MEMORY_MD);
    let _ = fs::write(instructions_dir.join("memory.md"), INSTRUCTIONS_MEMORY_MD);
    let _ = fs::write(workflows_dir.join("init-apm.md"), WORKFLOWS_INIT_MD);
    let _ = fs::write(workflows_dir.join("memory.md"), WORKFLOWS_MEMORY_MD);

    println!(
        "[SUCCESS] apm-mcp successfully installed to: {}",
        plugin_dir.display()
    );
    println!("Binary copied with absolute path configs. Ready to use across all projects!");
}
