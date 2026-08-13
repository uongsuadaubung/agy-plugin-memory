mod db;
mod export_import;
mod hook;
mod install;
mod mcp;
mod project;
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
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str());

    match mode {
        Some("install") | Some("--install") => {
            install::run_install_mode();
        }
        Some("uninstall") | Some("--uninstall") => {
            uninstall::run_uninstall_mode();
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
        Some("mcp") | Some("--mcp") | Some("stdio") | Some("--stdio") | Some("-m") => {
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
            if !io::stdin().is_terminal() {
                eprintln!("[ERROR] Unrecognized MCP command argument '{}', falling back to MCP mode.", other);
                mcp::run_mcp_mode();
            } else {
                println!("[ERROR] Unknown command: {}\n", other);
                print_cli_help();
            }
        }
    }
}
