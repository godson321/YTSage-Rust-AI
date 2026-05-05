use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::HistoryEntry;

use super::storage;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryStore {
    entries: Vec<HistoryEntry>,
}

static HISTORY_LOCK: Mutex<()> = Mutex::new(());

fn load_store() -> HistoryStore {
    let path = storage::history_path();
    match storage::read_json_file(&path) {
        Ok(store) => store,
        Err(_) => HistoryStore::default(),
    }
}

fn save_store(store: &HistoryStore) {
    let _ = storage::write_json_file(&storage::history_path(), store);
}

pub fn list_history() -> Vec<HistoryEntry> {
    let _guard = HISTORY_LOCK.lock().expect("history lock poisoned");
    load_store().entries
}

pub fn search_history(query: &str) -> Vec<HistoryEntry> {
    if query.trim().is_empty() {
        return list_history();
    }

    list_history()
        .into_iter()
        .filter(|entry| {
            let q = query.to_lowercase();
            entry.title.to_lowercase().contains(&q)
                || entry.url.to_lowercase().contains(&q)
                || entry.channel.to_lowercase().contains(&q)
        })
        .collect()
}

pub fn add_history_entry(entry: HistoryEntry) -> HistoryEntry {
    let _guard = HISTORY_LOCK.lock().expect("history lock poisoned");
    let mut store = load_store();
    store.entries.retain(|item| item.id != entry.id);
    store.entries.insert(0, entry.clone());
    save_store(&store);
    entry
}

pub fn remove_history_entry(entry_id: &str) -> bool {
    let _guard = HISTORY_LOCK.lock().expect("history lock poisoned");
    let mut store = load_store();
    let original_len = store.entries.len();
    store.entries.retain(|entry| entry.id != entry_id);
    let removed = store.entries.len() != original_len;
    if removed {
        save_store(&store);
    }
    removed
}

pub fn clear_history() -> usize {
    let _guard = HISTORY_LOCK.lock().expect("history lock poisoned");
    let mut store = load_store();
    let count = store.entries.len();
    store.entries.clear();
    save_store(&store);
    count
}

pub fn load_history_store() -> HistoryStore {
    let _guard = HISTORY_LOCK.lock().expect("history lock poisoned");
    load_store()
}

pub fn save_history_store(store: &HistoryStore) {
    let _guard = HISTORY_LOCK.lock().expect("history lock poisoned");
    save_store(store);
}

pub fn to_history_entry(value: Value) -> Option<HistoryEntry> {
    serde_json::from_value(value).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn set_temp_data_dir() -> PathBuf {
        let dir = env::temp_dir().join(format!("ytsage-rust-history-{}", Uuid::new_v4()));
        storage::set_data_dir_override(dir.clone());
        dir
    }

    fn sample_entry(id: &str, title: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            title: title.to_string(),
            url: format!("https://example.com/{id}"),
            channel: "Channel".to_string(),
            file_path: format!("C:/tmp/{id}.mp4"),
            download_date: Some("2026-01-01T00:00:00".to_string()),
            file_size: Some(123),
            thumbnail_url: Some("https://img.example/thumb.jpg".to_string()),
            format_id: Some("18".to_string()),
            resolution: Some("720p".to_string()),
            is_audio_only: false,
            duration: Some("1:00".to_string()),
            download_options: Some(Value::Null),
        }
    }

    #[test]
    fn add_and_list_history_entries_round_trip() {
        let _dir = set_temp_data_dir();
        let entry = sample_entry("1", "First");

        add_history_entry(entry.clone());
        let entries = list_history();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "First");
    }

    #[test]
    fn remove_history_entry_deletes_matching_item() {
        let _dir = set_temp_data_dir();
        add_history_entry(sample_entry("1", "First"));
        add_history_entry(sample_entry("2", "Second"));

        assert!(remove_history_entry("1"));
        assert_eq!(list_history().len(), 1);
    }

    #[test]
    fn clear_history_removes_all_items() {
        let _dir = set_temp_data_dir();
        add_history_entry(sample_entry("1", "First"));
        add_history_entry(sample_entry("2", "Second"));

        assert_eq!(clear_history(), 2);
        assert!(list_history().is_empty());
    }
}
