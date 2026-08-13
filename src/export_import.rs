use crate::db::{export_memories_to_json, import_memories_from_json, list_projects};

pub fn run_list_projects_mode() {
    match list_projects() {
        Ok(projs) => {
            println!("📋 Registered Projects in Memory Database:");
            println!("-----------------------------------------------------------------------------------------");
            println!("{:<14} {:<14} {:<22} {:<30}", "ID", "Memory Count", "Last Active", "Name & Path");
            println!("-----------------------------------------------------------------------------------------");
            for p in &projs {
                let short_active = if p.last_active.len() >= 16 { &p.last_active[..16] } else { &p.last_active };
                println!("{:<14} {:<14} {:<22} {} ({})", p.id, p.memory_count, short_active, p.name, p.path);
            }
            println!("-----------------------------------------------------------------------------------------");
            println!("Total Projects: {}\n", projs.len());
        }
        Err(e) => println!("❌ Failed to list projects: {}", e),
    }
}

pub fn run_export_mode(file_path: Option<&str>) {
    let target = file_path.unwrap_or("memory-backup.json");
    match export_memories_to_json(target) {
        Ok(path) => println!("📦 Successfully exported memories to: {}", path),
        Err(e) => println!("❌ Export failed: {}", e),
    }
}

pub fn run_import_mode(file_path: Option<&str>) {
    let target = match file_path {
        Some(f) => f,
        None => {
            println!("❌ Please specify input backup JSON file path to import. Example: memory-server import backup.json");
            return;
        }
    };

    match import_memories_from_json(target) {
        Ok((projs, mems)) => println!(
            "📥 Successfully imported {} projects and {} memories from: {}",
            projs, mems, target
        ),
        Err(e) => println!("❌ Import failed: {}", e),
    }
}
