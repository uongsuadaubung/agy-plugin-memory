use std::env;
use std::fs;



#[cfg(target_os = "windows")]
fn schedule_delayed_deletion(plugin_dir: &std::path::Path) {
    let dir_str = plugin_dir.to_string_lossy();
    let ps_cmd = format!(
        "$p = '{}'; for ($i=0; $i -lt 5; $i++) {{ Start-Sleep -Seconds 1; if (Test-Path $p) {{ Remove-Item -Path $p -Recurse -Force -ErrorAction SilentlyContinue }} else {{ break }} }}",
        dir_str
    );

    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd])
        .spawn();
}

#[cfg(not(target_os = "windows"))]
fn schedule_delayed_deletion(plugin_dir: &std::path::Path) {
    let dir_str = plugin_dir.to_string_lossy();
    let sh_cmd = format!("for i in 1 2 3 4 5; do sleep 1; if [ -d '{}' ]; then rm -rf '{}'; else break; fi; done", dir_str, dir_str);
    let _ = std::process::Command::new("sh")
        .args(["-c", &sh_cmd])
        .spawn();
}

pub fn run_uninstall_mode() {
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

    let curr_exe = env::current_exe().ok();

    if plugin_dir.exists() {
        if let Ok(entries) = fs::read_dir(&plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ref exe) = curr_exe {
                    if path == *exe || path.join("memory-server.exe") == *exe {
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

        if fs::remove_dir_all(&plugin_dir).is_err() {
            schedule_delayed_deletion(&plugin_dir);
            println!("[INFO] Scheduled background self-deletion on exit.");
        } else {
            println!("[CLEAN] Plugin directory removed: {}", plugin_dir.display());
        }
    } else {
        println!("[INFO] Plugin directory does not exist: {}", plugin_dir.display());
    }

    // Remove memory database directory (~/.gemini/config/memory)
    if let Some(mut mem_dir) = dirs::home_dir() {
        mem_dir.push(".gemini");
        mem_dir.push("config");
        mem_dir.push("memory");

        if mem_dir.exists() {
            if fs::remove_dir_all(&mem_dir).is_err() {
                schedule_delayed_deletion(&mem_dir);
                println!("[INFO] Scheduled background memory database directory self-deletion on exit.");
            } else {
                println!("[CLEAN] Memory database directory removed: {}", mem_dir.display());
            }
        }
    }

    println!("[SUCCESS] apm-mcp successfully uninstalled!");
}
