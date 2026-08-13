use crate::db::{export_memories_to_json, import_memories_from_json};

pub fn run_export_mode(file_path: Option<&str>) {
    let target = file_path.unwrap_or("memory-backup.json");
    match export_memories_to_json(target) {
        Ok(path) => println!("[SUCCESS] Successfully exported memories to: {}", path),
        Err(e) => println!("[ERROR] Export failed: {}", e),
    }
}

pub fn run_import_mode(file_path: Option<&str>) {
    let target = match file_path {
        Some(f) => f,
        None => {
            println!("[ERROR] Please specify input backup JSON file path to import. Example: apm-mcp import backup.json");
            return;
        }
    };

    match import_memories_from_json(target) {
        Ok((projs, mems)) => println!(
            "[SUCCESS] Successfully imported {} projects and {} memories from: {}",
            projs, mems, target
        ),
        Err(e) => println!("[ERROR] Import failed: {}", e),
    }
}
