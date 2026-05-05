use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::models::UpdateStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheck {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub source: String,
}

pub fn compare_versions(current: &str, latest: &str) -> Ordering {
    let current_parts: Vec<u64> = current.split('.').map(|part| part.parse().unwrap_or(0)).collect();
    let latest_parts: Vec<u64> = latest.split('.').map(|part| part.parse().unwrap_or(0)).collect();

    for index in 0..current_parts.len().max(latest_parts.len()) {
        let current_part = *current_parts.get(index).unwrap_or(&0);
        let latest_part = *latest_parts.get(index).unwrap_or(&0);
        match current_part.cmp(&latest_part) {
            Ordering::Equal => continue,
            other => return other,
        }
    }

    Ordering::Equal
}

pub fn build_update_status(check: UpdateCheck) -> UpdateStatus {
    let update_available = check
        .latest_version
        .as_deref()
        .map(|latest| compare_versions(&check.current_version, latest) == Ordering::Less)
        .unwrap_or(false);

    UpdateStatus {
        current_version: check.current_version,
        latest_version: check.latest_version,
        update_available,
        source: check.source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_versions_orders_semver_like_strings() {
        assert_eq!(compare_versions("1.2.0", "1.3.0"), Ordering::Less);
        assert_eq!(compare_versions("1.3.0", "1.2.0"), Ordering::Greater);
        assert_eq!(compare_versions("1.2.0", "1.2.0"), Ordering::Equal);
    }

    #[test]
    fn build_update_status_flags_available_updates() {
        let status = build_update_status(UpdateCheck {
            current_version: "1.0.0".to_string(),
            latest_version: Some("1.1.0".to_string()),
            source: "pypi".to_string(),
        });

        assert!(status.update_available);
    }
}
