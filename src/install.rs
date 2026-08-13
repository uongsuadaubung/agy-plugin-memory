use std::env;
use std::fs;

use crate::ensure_bin_in_user_path;

// Embed plugin files directly into binary at compile time via include_str!
const PLUGIN_JSON: &str = include_str!("../plugins/uongsuadaubung-plugin/plugin.json");
const MCP_CONFIG_JSON: &str = include_str!("../plugins/uongsuadaubung-plugin/mcp_config.json");
const HOOKS_JSON: &str = include_str!("../plugins/uongsuadaubung-plugin/hooks.json");
const RULES_MEMORY_MD: &str = include_str!("../plugins/uongsuadaubung-plugin/rules/memory.md");
const INSTRUCTIONS_MEMORY_MD: &str = include_str!("../plugins/uongsuadaubung-plugin/instructions/memory.md");
const WORKFLOWS_INIT_MD: &str = include_str!("../plugins/uongsuadaubung-plugin/workflows/init.md");
const WORKFLOWS_MEMORY_MD: &str = include_str!("../plugins/uongsuadaubung-plugin/workflows/memory.md");

pub fn run_install_mode() {
    let mut plugin_dir = match dirs::home_dir() {
        Some(h) => h,
        None => {
            println!("❌ Could not resolve home directory.");
            return;
        }
    };

    plugin_dir.push(".gemini");
    plugin_dir.push("config");
    plugin_dir.push("plugins");
    plugin_dir.push("uongsuadaubung-plugin");

    let bin_dir = plugin_dir.join("bin");
    let rules_dir = plugin_dir.join("rules");
    let instructions_dir = plugin_dir.join("instructions");
    let workflows_dir = plugin_dir.join("workflows");

    let _ = fs::create_dir_all(&bin_dir);
    let _ = fs::create_dir_all(&rules_dir);
    let _ = fs::create_dir_all(&instructions_dir);
    let _ = fs::create_dir_all(&workflows_dir);

    // 1. Copy the current running binary into target plugin bin/ directory
    if let Ok(curr_exe) = env::current_exe() {
        #[cfg(target_os = "windows")]
        let target_exe = bin_dir.join("uongsuadaubung-memory.exe");
        #[cfg(not(target_os = "windows"))]
        let target_exe = bin_dir.join("uongsuadaubung-memory");

        if curr_exe != target_exe {
            let _ = fs::copy(&curr_exe, &target_exe);
        }
    }

    // 2. Extract embedded plugin files to target directory
    let _ = fs::write(plugin_dir.join("plugin.json"), PLUGIN_JSON);
    let _ = fs::write(plugin_dir.join("mcp_config.json"), MCP_CONFIG_JSON);
    let _ = fs::write(plugin_dir.join("hooks.json"), HOOKS_JSON);
    let _ = fs::write(rules_dir.join("memory.md"), RULES_MEMORY_MD);
    let _ = fs::write(instructions_dir.join("memory.md"), INSTRUCTIONS_MEMORY_MD);
    let _ = fs::write(workflows_dir.join("init.md"), WORKFLOWS_INIT_MD);
    let _ = fs::write(workflows_dir.join("memory.md"), WORKFLOWS_MEMORY_MD);

    // 3. Register User PATH
    ensure_bin_in_user_path();

    println!(
        "✅ uongsuadaubung-plugin successfully installed to: {}",
        plugin_dir.display()
    );
    println!("🚀 Binary copied and User PATH updated. Ready to use across all projects!");
}
