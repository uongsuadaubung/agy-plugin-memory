use std::path::Path;

use crate::project::normalize_path;

pub const IDE_APP_PATTERNS: &[&str] = &[
    "appdata/local/programs/antigravity ide",
    "appdata/local/programs/antigravity",
    "program files/antigravity ide",
    "program files (x86)/antigravity ide",
    "/usr/share/antigravity",
    "/opt/antigravity",
    ".local/share/antigravity",
    "antigravity.app/contents/macos",
    "antigravity ide.app/contents/macos",
];

pub fn is_ide_app_dir(path: &Path) -> bool {
    let norm = normalize_path(path);
    IDE_APP_PATTERNS.iter().any(|pattern| norm.contains(pattern))
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
    unsafe extern "C" {
        fn getppid() -> i32;
    }
    unsafe { getppid().max(0) as u32 }
}
