use super::{HardwareSnapshot, SystemTimingsSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceSnapshot {
    pub timestamp: String,
    pub label: String,
    pub description: String,
    pub git_commit: String,
    pub git_branch: String,
    pub git_dirty: bool,
    pub build_type: String,
    pub rust_version: String,
    pub creature_count: u64,
    pub hardware_metrics: HardwareSnapshot,
    pub ecs_metrics: EcsMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcsMetrics {
    pub archetype_count: u64,
    pub entity_count: u64,
    pub system_tick_ms: f64,
}

impl PerformanceSnapshot {
    pub fn new(
        label: String,
        description: String,
        creature_count: u64,
        hardware_metrics: HardwareSnapshot,
        system_timings: &SystemTimingsSnapshot,
    ) -> Self {
        let git_info = get_git_info();

        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            label,
            description,
            git_commit: git_info.commit,
            git_branch: git_info.branch,
            git_dirty: git_info.dirty,
            build_type: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
            .to_string(),
            rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
            creature_count,
            hardware_metrics,
            ecs_metrics: EcsMetrics {
                archetype_count: system_timings.archetype_count,
                entity_count: system_timings.entity_count,
                system_tick_ms: system_timings.total_tick_us as f64 / 1000.0,
            },
        }
    }
}

#[derive(Debug)]
struct GitInfo {
    commit: String,
    branch: String,
    dirty: bool,
}

#[cfg(feature = "dev-tools")]
fn get_git_info() -> GitInfo {
    use git2::{Repository, StatusOptions};

    let repo = match Repository::open(".") {
        Ok(r) => r,
        Err(_) => {
            return GitInfo {
                commit: "unknown".to_string(),
                branch: "unknown".to_string(),
                dirty: false,
            }
        }
    };

    let head = repo.head().ok();
    let commit = head
        .as_ref()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|c| format!("{:.7}", c.id()))
        .unwrap_or_else(|| "unknown".to_string());

    let branch = head
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    let mut opts = StatusOptions::new();
    opts.include_untracked(false);
    let dirty = repo
        .statuses(Some(&mut opts))
        .ok()
        .map(|statuses| statuses.iter().any(|e| e.status() != git2::Status::CURRENT))
        .unwrap_or(false);

    GitInfo {
        commit,
        branch,
        dirty,
    }
}

#[cfg(not(feature = "dev-tools"))]
fn get_git_info() -> GitInfo {
    GitInfo {
        commit: "unknown".to_string(),
        branch: "unknown".to_string(),
        dirty: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let hw_metrics = HardwareSnapshot::default();
        let sys_timings = SystemTimingsSnapshot::default();

        let snapshot = PerformanceSnapshot::new(
            "test-snapshot".to_string(),
            "Test description".to_string(),
            1000,
            hw_metrics,
            &sys_timings,
        );

        assert_eq!(snapshot.label, "test-snapshot");
        assert_eq!(snapshot.creature_count, 1000);
        assert!(!snapshot.timestamp.is_empty());
        assert!(snapshot.build_type == "debug" || snapshot.build_type == "release");
    }

    #[test]
    fn test_snapshot_json_serialization() {
        let snapshot = PerformanceSnapshot::new(
            "test".to_string(),
            "desc".to_string(),
            500,
            HardwareSnapshot::default(),
            &SystemTimingsSnapshot::default(),
        );

        let json = serde_json::to_string_pretty(&snapshot).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("hardwareMetrics"));
        assert!(json.contains("ecsMetrics"));
        assert!(json.contains("gitCommit"));
        assert!(json.contains("gitBranch"));
        assert!(json.contains("gitDirty"));
    }

    #[test]
    #[cfg(feature = "dev-tools")]
    fn test_git_info_extraction() {
        let git_info = get_git_info();
        assert!(!git_info.commit.is_empty());
        assert!(!git_info.branch.is_empty());
    }
}
