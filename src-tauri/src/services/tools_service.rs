use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::models::ToolStatus;

pub fn get_tool_status() -> Vec<ToolStatus> {
    vec![tool_status("yt-dlp"), tool_status("ffmpeg"), tool_status("deno")]
}

pub fn find_required_tool(name: &str) -> Option<PathBuf> {
    find_tool_path(name)
}

fn tool_status(name: &str) -> ToolStatus {
    let path = find_tool_path(name);
    let installed = path.is_some();
    let current_version = path
        .as_ref()
        .and_then(|path| get_version(path))
        .or_else(|| if installed { Some("unknown".to_string()) } else { None });

    ToolStatus {
        name: name.to_string(),
        installed,
        current_version,
        path: path.map(|path| path.to_string_lossy().to_string()),
    }
}

fn find_tool_path(name: &str) -> Option<PathBuf> {
    if let Ok(custom_dir) = env::var("YTSAGE_RUST_TOOL_DIR") {
        for candidate in tool_name_candidates(&PathBuf::from(custom_dir), name) {
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .flat_map(|dir| tool_name_candidates(&dir, name))
            .find(|candidate| candidate.exists())
    })
}

fn tool_name_candidates(dir: &PathBuf, name: &str) -> Vec<PathBuf> {
    let mut candidates = vec![dir.join(name)];

    #[cfg(windows)]
    {
        if PathBuf::from(name).extension().is_none() {
            candidates.push(dir.join(format!("{name}.exe")));
        }
    }

    candidates
}

fn get_version(path: &PathBuf) -> Option<String> {
    Command::new(path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().map(|text| text.trim().to_string())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use uuid::Uuid;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &PathBuf) -> Self {
            let previous = env::var_os(key);
            unsafe {
                env::set_var(key, value);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => unsafe {
                    env::set_var(self.key, value);
                },
                None => unsafe {
                    env::remove_var(self.key);
                },
            }
        }
    }

    #[test]
    fn returns_three_known_tools() {
        let tools = get_tool_status();
        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0].name, "yt-dlp");
        assert_eq!(tools[1].name, "ffmpeg");
        assert_eq!(tools[2].name, "deno");
    }

    #[cfg(windows)]
    #[test]
    fn find_required_tool_accepts_windows_executable_in_custom_tool_dir() {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let temp_dir = env::temp_dir().join(format!("ytsage-rust-tools-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("temp tool dir should exist");
        let exe_path = temp_dir.join("yt-dlp.exe");
        fs::write(&exe_path, b"stub").expect("stub exe should be created");
        let _env_guard = EnvVarGuard::set("YTSAGE_RUST_TOOL_DIR", &temp_dir);

        let found = find_required_tool("yt-dlp");

        assert_eq!(found.as_deref(), Some(exe_path.as_path()));
        let _ = fs::remove_dir_all(temp_dir);
    }
}
