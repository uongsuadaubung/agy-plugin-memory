use md5::{Digest, Md5};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DetectedProject {
    pub id: String,
    pub name: String,
    pub path: String,
}

pub fn normalize_path(path: &Path) -> String {
    let s = path.to_string_lossy().to_string();
    let clean = if s.starts_with(r"\\?\") {
        &s[4..]
    } else {
        &s
    };
    clean.replace('\\', "/").to_lowercase()
}

pub fn hash_project_path(path: &Path) -> String {
    let norm = normalize_path(path);
    let mut hasher = Md5::new();
    hasher.update(norm.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..12].to_string()
}

pub fn find_project_root(starting_path: Option<&str>) -> PathBuf {
    let start = match starting_path {
        Some(p) => PathBuf::from(p),
        None => env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let canonical = start.canonicalize().unwrap_or(start);
    let mut curr = canonical.as_path();

    let root_markers = [
        ".git",
        "package.json",
        "Cargo.toml",
        "pyproject.toml",
        "go.mod",
        "pom.xml",
        "build.gradle",
        ".antigravity",
    ];

    while let Some(parent) = curr.parent() {
        for marker in &root_markers {
            if curr.join(marker).exists() {
                return curr.to_path_buf();
            }
        }
        curr = parent;
    }

    canonical
}

pub fn get_auto_detected_project(name_override: Option<&str>, path_override: Option<&str>) -> DetectedProject {
    let root = find_project_root(path_override);
    let id = hash_project_path(&root);

    let clean_path = if root.to_string_lossy().starts_with(r"\\?\") {
        root.to_string_lossy()[4..].to_string()
    } else {
        root.to_string_lossy().to_string()
    };

    let name = match name_override {
        Some(n) if !n.trim().is_empty() => n.trim().to_string(),
        _ => root
            .file_name()
            .map(|os| os.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown-project".to_string()),
    };

    DetectedProject {
        id,
        name,
        path: clean_path,
    }
}
