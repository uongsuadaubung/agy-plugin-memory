mod db;
mod export_import;
mod hook;
mod install;
mod mcp;
mod project;
mod uninstall;

use std::env;
use std::io::{self, IsTerminal};

#[cfg(target_os = "windows")]
pub fn ensure_bin_in_user_path() {
    if let Some(mut bin_dir) = dirs::home_dir() {
        bin_dir.push(".gemini");
        bin_dir.push("config");
        bin_dir.push("plugins");
        bin_dir.push("uongsuadaubung-plugin");
        bin_dir.push("bin");

        if bin_dir.exists() {
            let bin_str = bin_dir.to_string_lossy().to_string();
            let cmd = format!(
                "$bin = '{}'; $old = [Environment]::GetEnvironmentVariable('PATH', 'User'); if ($old -notlike '*'+$bin+'*') {{ [Environment]::SetEnvironmentVariable('PATH', $old + ';' + $bin, 'User') }}",
                bin_str
            );

            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", &cmd])
                .output();
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_bin_in_user_path() {
    if let Some(mut bin_dir) = dirs::home_dir() {
        bin_dir.push(".gemini");
        bin_dir.push("config");
        bin_dir.push("plugins");
        bin_dir.push("uongsuadaubung-plugin");
        bin_dir.push("bin");

        if bin_dir.exists() {
            let bin_str = bin_dir.to_string_lossy().to_string();
            let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("~"));

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let target_exe = bin_dir.join("uongsuadaubung-memory");
                if target_exe.exists() {
                    let _ = std::fs::set_permissions(&target_exe, std::fs::Permissions::from_mode(0o755));
                }
            }

            for rcfile in &[".zshrc", ".bashrc"] {
                let rcpath = home.join(rcfile);
                if rcpath.exists() {
                    if let Ok(content) = std::fs::read_to_string(&rcpath) {
                        if !content.contains("uongsuadaubung-plugin/bin") {
                            let export_line = format!("\nexport PATH=\"$PATH:{}\"\n", bin_str);
                            let _ = std::fs::OpenOptions::new().append(true).open(&rcpath).and_then(|mut f| {
                                use std::io::Write;
                                f.write_all(export_line.as_bytes())
                            });
                        }
                    }
                }
            }
        }
    }
}

fn print_cli_help() {
    println!("🧠 uongsuadaubung-memory - Memory MCP Server & Plugin (v1.0.0)");
    println!("--------------------------------------------------");
    println!("High-performance Rust memory server for Antigravity.\n");
    println!("Usage:");
    println!("  uongsuadaubung-memory <COMMAND>\n");
    println!("Commands:");
    println!("  install          Install plugin to ~/.gemini/config/plugins/uongsuadaubung-plugin & register PATH");
    println!("  uninstall        Uninstall plugin and clean PATH");
    println!("  projects         List all registered projects in memory database");
    println!("  export [file]    Export memory database to a JSON backup file (default: memory-backup.json)");
    println!("  import <file>    Import memory database from a JSON backup file");
    println!("  hook             Run PreInvocation Lifecycle Hook mode (used by Antigravity)");
    println!("  mcp              Run Stdio MCP JSON-RPC Server mode (used by Antigravity IDE)");
    println!("  help             Display this help message\n");
    println!("Example:");
    println!("  uongsuadaubung-memory projects");
}

fn main() {
    #[cfg(target_os = "windows")]
    ensure_bin_in_user_path();

    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str());

    match mode {
        Some("install") | Some("--install") => {
            install::run_install_mode();
        }
        Some("uninstall") | Some("--uninstall") => {
            uninstall::run_uninstall_mode();
        }
        Some("projects") | Some("list-projects") | Some("--projects") => {
            export_import::run_list_projects_mode();
        }
        Some("export") | Some("--export") => {
            let path = args.get(2).map(|s| s.as_str());
            export_import::run_export_mode(path);
        }
        Some("import") | Some("--import") => {
            let path = args.get(2).map(|s| s.as_str());
            export_import::run_import_mode(path);
        }
        Some("hook") | Some("--hook") => {
            hook::run_hook_mode();
        }
        Some("help") | Some("--help") | Some("-h") => {
            print_cli_help();
        }
        Some("mcp") | Some("--mcp") => {
            mcp::run_mcp_mode();
        }
        None => {
            if io::stdin().is_terminal() {
                print_cli_help();
            } else {
                mcp::run_mcp_mode();
            }
        }
        Some(other) => {
            println!("❌ Unknown command: {}\n", other);
            print_cli_help();
        }
    }
}
