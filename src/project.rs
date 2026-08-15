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

#[cfg(windows)]
pub fn get_parent_pid() -> u32 {
    #[repr(C)]
    struct PROCESSENTRY32W {
        dw_size: u32,
        cnt_usage: u32,
        th32_process_id: u32,
        th32_default_heap_id: usize,
        th32_module_id: u32,
        cnt_threads: u32,
        th32_parent_process_id: u32,
        pc_pri_class_base: i32,
        dw_flags: u32,
        sz_exe_file: [u16; 260],
    }

    unsafe extern "system" {
        fn CreateToolhelp32Snapshot(dw_flags: u32, th32_process_id: u32) -> isize;
        fn Process32FirstW(h_snapshot: isize, lppe: *mut PROCESSENTRY32W) -> i32;
        fn Process32NextW(h_snapshot: isize, lppe: *mut PROCESSENTRY32W) -> i32;
        fn CloseHandle(h_object: isize) -> i32;
        fn GetCurrentProcessId() -> u32;
    }

    unsafe {
        let current_pid = GetCurrentProcessId();
        let snapshot = CreateToolhelp32Snapshot(0x00000002, 0); // TH32CS_SNAPPROCESS
        if snapshot == -1 || snapshot == 0 {
            return 0;
        }

        let mut entry = PROCESSENTRY32W {
            dw_size: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            cnt_usage: 0,
            th32_process_id: 0,
            th32_default_heap_id: 0,
            th32_module_id: 0,
            cnt_threads: 0,
            th32_parent_process_id: 0,
            pc_pri_class_base: 0,
            dw_flags: 0,
            sz_exe_file: [0; 260],
        };

        let mut ppid = 0;
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                if entry.th32_process_id == current_pid {
                    ppid = entry.th32_parent_process_id;
                    break;
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        ppid
    }
}

#[cfg(not(windows))]
pub fn get_parent_pid() -> u32 {
    unsafe { libc::getppid() as u32 }
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
    let root = match find_project_root(path_override) {
        Ok(r) => r,
        Err(e) => {
            if path_override.is_none() {
                if let Some((wpath, pid, pname)) = crate::db::get_active_workspace() {
                    return Ok(DetectedProject {
                        id: pid,
                        name: name_override.map(String::from).unwrap_or(pname),
                        path: wpath,
                    });
                }
            }
            return Err(e);
        }
    };
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
