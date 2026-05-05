use crate::models::ProcessSnapshot;

pub fn empty_snapshot() -> ProcessSnapshot {
    ProcessSnapshot {
        stdout: String::new(),
        stderr: String::new(),
        exit_code: None,
        progress: 0.0,
    }
}
