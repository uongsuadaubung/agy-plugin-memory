use md5::{Digest, Md5};
use std::path::{Path, PathBuf};

const IDE_APP_PATTERNS: &[&str] = &[
    "appdata/local/programs/antigravity ide",
    "appdata/local/programs/antigravity",
    "program files/antigravity ide",
    "program files (x86)/antigravity ide",
];

const ROOT_MARKERS: &[&str] = &[
    ".git",
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "go.mod",
    "pom.xml",
    "build.gradle",
    ".antigravity",
];

#[derive(Debug, Clone)]
pub struct DetectedProject {
    pub id: String,
    pub name: String,
    pub path: String,
}

pub fn normalize_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    let clean = s.strip_prefix(r"\\?\").unwrap_or(&s);
    clean.replace('\\', "/").to_lowercase()
}

pub fn is_ide_app_dir(path: &Path) -> bool {
    let norm = normalize_path(path);
    IDE_APP_PATTERNS.iter().any(|pattern| norm.contains(pattern))
}

pub fn hash_project_path(path: &Path) -> String {
    let norm = normalize_path(path);
    let mut hasher = Md5::new();
    hasher.update(norm.as_bytes());
    let result = hasher.finalize();
    format!("{result:x}")[..12].to_string()
}

pub fn find_project_root(starting_path: Option<&str>) -> Result<PathBuf, String> {
    let start = match starting_path {
        Some(p) if !p.trim().is_empty() => PathBuf::from(p.trim()),
        _ => match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(_) => return Err("No project path provided and current working directory could not be determined.".to_string()),
        },
    };

    let canonical = start.canonicalize().unwrap_or(start);

    if is_ide_app_dir(&canonical) {
        let display = canonical.display();
        return Err(format!(
            "Path '{display}' is an IDE application installation directory, not a user project workspace."
        ));
    }

    let mut curr = canonical.as_path();

    while let Some(parent) = curr.parent() {
        for marker in ROOT_MARKERS {
            if curr.join(marker).exists() {
                return Ok(curr.to_path_buf());
            }
        }
        curr = parent;
    }

    Ok(canonical)
}

pub fn get_auto_detected_project(
    name_override: Option<&str>,
    path_override: Option<&str>,
) -> Result<DetectedProject, String> {
    let root = find_project_root(path_override)?;
    let id = hash_project_path(&root);

    let raw_str = root.to_string_lossy();
    let clean_path = raw_str
        .strip_prefix(r"\\?\")
        .unwrap_or(&raw_str)
        .to_string();

    let name = match name_override {
        Some(n) if !n.trim().is_empty() => n.trim().to_string(),
        _ => root
            .file_name()
            .map(|os| os.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown-project".to_string()),
    };

    Ok(DetectedProject {
        id,
        name,
        path: clean_path,
    })
}



