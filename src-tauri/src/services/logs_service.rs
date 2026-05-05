use std::fs;
use std::sync::Mutex;

use super::storage;

static LOG_LOCK: Mutex<()> = Mutex::new(());

pub fn append_log_line(line: &str) {
    let _guard = LOG_LOCK.lock().expect("log lock poisoned");
    let path = storage::log_path();
    let _ = storage::ensure_parent_dir(&path);
    let mut content = fs::read_to_string(&path).unwrap_or_default();
    content.push_str(line);
    content.push('\n');
    let _ = fs::write(path, content);
}

pub fn tail_logs(lines: usize) -> Vec<String> {
    let _guard = LOG_LOCK.lock().expect("log lock poisoned");
    let content = fs::read_to_string(storage::log_path()).unwrap_or_default();
    let mut collected: Vec<String> = content
        .lines()
        .map(ToString::to_string)
        .collect();

    if lines == 0 || collected.len() <= lines {
        return collected;
    }

    collected.drain(0..collected.len() - lines);
    collected
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use uuid::Uuid;

    fn set_temp_data_dir() {
        let dir = env::temp_dir().join(format!("ytsage-rust-logs-{}", Uuid::new_v4()));
        storage::set_data_dir_override(dir);
    }

    #[test]
    fn tail_logs_returns_latest_lines() {
        set_temp_data_dir();
        append_log_line("one");
        append_log_line("two");
        append_log_line("three");

        assert_eq!(tail_logs(2), vec!["two".to_string(), "three".to_string()]);
    }
}
