mod db;
mod export_import;
mod hook;
mod install;
mod mcp;
mod process;
mod project;
mod similarity;
mod uninstall;

use std::env;
use std::io::{self, IsTerminal};

fn print_cli_help() {
    println!("apm-mcp - Memory MCP Server & Plugin (v1.0.0)");
    println!("--------------------------------------------------");
    println!("High-performance Rust memory server for Antigravity.\n");
    println!("Usage:");
    println!("  apm-mcp <COMMAND>\n");
    println!("Commands:");
    println!("  install          Install plugin to ~/.gemini/config/plugins/apm-mcp");
    println!("  uninstall        Uninstall plugin");
    println!("  export [file]    Export memory database to a JSON backup file (default: memory-backup.json)");
    println!("  import <file>    Import memory database from a JSON backup file");
    println!("  hook             Run PreInvocation Lifecycle Hook mode (used by Antigravity)");
    println!("  mcp              Run Stdio MCP JSON-RPC Server mode (used by Antigravity IDE)\n");
    println!("Example:");
    println!("  apm-mcp export");
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("[FATAL PANIC in apm-mcp] {info}");
    }));

    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(String::as_str);

    match mode {
        Some("install" | "--install") => {
            install::run_install_mode();
        }
        Some("uninstall" | "--uninstall") => {
            uninstall::run_uninstall_mode();
        }
        Some("export" | "--export") => {
            let path = args.get(2).map(String::as_str);
            export_import::run_export_mode(path);
        }
        Some("import" | "--import") => {
            let path = args.get(2).map(String::as_str);
            export_import::run_import_mode(path);
        }
        Some("hook" | "--hook") => {
            hook::run_hook_mode();
        }
        Some("help" | "--help" | "-h") => {
            print_cli_help();
        }
        Some("mcp" | "--mcp" | "stdio" | "--stdio" | "-m") => {
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
            if io::stdin().is_terminal() {
                println!("[ERROR] Unknown command: {other}\n");
                print_cli_help();
            } else {
                eprintln!("[ERROR] Unrecognized MCP command argument '{other}', falling back to MCP mode.");
                mcp::run_mcp_mode();
            }
        }
    }
}
