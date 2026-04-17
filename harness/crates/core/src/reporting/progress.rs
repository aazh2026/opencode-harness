use crate::reporting::report::TaskResult;
use crate::types::TaskStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStats {
    pub done_count: u32,
    pub in_progress_count: u32,
    pub todo_count: u32,
    pub manual_check_count: u32,
    pub blocked_count: u32,
    pub skipped_count: u32,
    pub running_tasks: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

impl ProgressStats {
    pub fn from_task_results(results: &[TaskResult]) -> Self {
        let mut done_count = 0u32;
        let in_progress_count = 0u32;
        let todo_count = 0u32;
        let mut manual_check_count = 0u32;
        let mut blocked_count = 0u32;
        let skipped_count = 0u32;

        for result in results {
            match &result.verdict {
                crate::types::parity_verdict::ParityVerdict::Pass
                | crate::types::parity_verdict::ParityVerdict::PassWithAllowedVariance { .. } => {
                    done_count += 1;
                }
                crate::types::parity_verdict::ParityVerdict::Warn { .. } => {
                    done_count += 1;
                }
                crate::types::parity_verdict::ParityVerdict::Fail { .. } => {
                    done_count += 1;
                }
                crate::types::parity_verdict::ParityVerdict::ManualCheck { .. } => {
                    manual_check_count += 1;
                }
                crate::types::parity_verdict::ParityVerdict::Blocked { .. } => {
                    blocked_count += 1;
                }
                crate::types::parity_verdict::ParityVerdict::Error { .. } => {
                    done_count += 1;
                }
            }
        }

        Self {
            done_count,
            in_progress_count,
            todo_count,
            manual_check_count,
            blocked_count,
            skipped_count,
            running_tasks: Vec::new(),
            last_updated: Utc::now(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Default for ProgressStats {
    fn default() -> Self {
        Self {
            done_count: 0,
            in_progress_count: 0,
            todo_count: 0,
            manual_check_count: 0,
            blocked_count: 0,
            skipped_count: 0,
            running_tasks: Vec::new(),
            last_updated: Utc::now(),
        }
    }
}

pub struct TaskTracker {
    running_tasks: Arc<Mutex<HashMap<String, TaskStatus>>>,
}

impl TaskTracker {
    pub fn new() -> Self {
        Self {
            running_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start_task(&self, task_id: &str) {
        let mut tasks = self.running_tasks.lock().unwrap();
        tasks.insert(task_id.to_string(), TaskStatus::InProgress);
    }

    pub fn complete_task(&self, task_id: &str, status: TaskStatus) {
        let mut tasks = self.running_tasks.lock().unwrap();
        tasks.insert(task_id.to_string(), status);
    }

    pub fn get_running_tasks(&self) -> Vec<String> {
        let tasks = self.running_tasks.lock().unwrap();
        tasks.iter()
            .filter(|(_, status)| **status == TaskStatus::InProgress)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn get_stats(&self) -> ProgressStats {
        let tasks = self.running_tasks.lock().unwrap();
        let mut stats = ProgressStats::default();
        for (_, status) in tasks.iter() {
            match status {
                TaskStatus::Done => stats.done_count += 1,
                TaskStatus::InProgress => stats.in_progress_count += 1,
                TaskStatus::Todo => stats.todo_count += 1,
                TaskStatus::ManualCheck => stats.manual_check_count += 1,
                TaskStatus::Blocked => stats.blocked_count += 1,
                TaskStatus::Skipped => stats.skipped_count += 1,
            }
        }
        stats.running_tasks = tasks
            .iter()
            .filter(|(_, status)| **status == TaskStatus::InProgress)
            .map(|(id, _)| id.clone())
            .collect();
        stats.last_updated = Utc::now();
        stats
    }
}

impl Default for TaskTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LogTailReader;

impl LogTailReader {
    pub fn read_tail(&self, path: &Path, lines: usize) -> Result<Vec<String>, IoError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;
        let start = if all_lines.len() > lines {
            all_lines.len() - lines
        } else {
            0
        };
        Ok(all_lines[start..].to_vec())
    }
}

pub struct RecentArtifactsReader;

impl RecentArtifactsReader {
    pub fn find_recent(&self, base_path: &Path, count: usize) -> Vec<PathBuf> {
        let mut entries: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

        if let Ok(read_dir) = std::fs::read_dir(base_path) {
            for entry in read_dir.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            entries.push((entry.path(), modified));
                        }
                    }
                }
            }
        }

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.into_iter().take(count).map(|(p, _)| p).collect()
    }

    pub fn find_latest_report(&self, base_path: &Path) -> Option<PathBuf> {
        let mut latest_path: Option<PathBuf> = None;
        let mut latest_time: Option<std::time::SystemTime> = None;

        if let Ok(read_dir) = std::fs::read_dir(base_path) {
            for entry in read_dir.flatten() {
                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with("report") && name.ends_with(".json") {
                            if let Ok(metadata) = entry.metadata() {
                                if let Ok(modified) = metadata.modified() {
                                    if latest_time.map(|t| modified > t).unwrap_or(true) {
                                        latest_time = Some(modified);
                                        latest_path = Some(entry.path());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        latest_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::parity_verdict::{DiffCategory, ParityVerdict};

    #[test]
    fn test_progress_stats_from_task_results_passing() {
        let results = vec![
            TaskResult::new("T1".to_string(), ParityVerdict::Pass, 100),
            TaskResult::new("T2".to_string(), ParityVerdict::Pass, 100),
        ];
        let stats = ProgressStats::from_task_results(&results);
        assert_eq!(stats.done_count, 2);
        assert_eq!(stats.in_progress_count, 0);
        assert_eq!(stats.todo_count, 0);
    }

    #[test]
    fn test_progress_stats_from_task_results_with_fail() {
        let results = vec![
            TaskResult::new("T1".to_string(), ParityVerdict::Pass, 100),
            TaskResult::new(
                "T2".to_string(),
                ParityVerdict::Fail {
                    category: DiffCategory::OutputText,
                    details: "Mismatch".to_string(),
                },
                50,
            ),
        ];
        let stats = ProgressStats::from_task_results(&results);
        assert_eq!(stats.done_count, 2);
    }

    #[test]
    fn test_progress_stats_from_task_results_with_manual_check() {
        let results = vec![
            TaskResult::new("T1".to_string(), ParityVerdict::Pass, 100),
            TaskResult::new(
                "T2".to_string(),
                ParityVerdict::ManualCheck {
                    reason: "Needs review".to_string(),
                    candidates: vec![],
                },
                100,
            ),
        ];
        let stats = ProgressStats::from_task_results(&results);
        assert_eq!(stats.done_count, 1);
        assert_eq!(stats.manual_check_count, 1);
    }

    #[test]
    fn test_progress_stats_from_task_results_with_blocked() {
        let results = vec![
            TaskResult::new("T1".to_string(), ParityVerdict::Pass, 100),
            TaskResult::new(
                "T2".to_string(),
                ParityVerdict::Blocked {
                    reason: crate::types::parity_verdict::BlockedReason::BinaryNotFound {
                        binary: "opencode".to_string(),
                    },
                },
                0,
            ),
        ];
        let stats = ProgressStats::from_task_results(&results);
        assert_eq!(stats.done_count, 1);
        assert_eq!(stats.blocked_count, 1);
    }

    #[test]
    fn test_progress_stats_to_json() {
        let stats = ProgressStats::default();
        let json = stats.to_json();
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("done_count"));
        assert!(json_str.contains("in_progress_count"));
    }

    #[test]
    fn test_task_tracker_start_task() {
        let tracker = TaskTracker::new();
        tracker.start_task("task-1");
        let running = tracker.get_running_tasks();
        assert!(running.contains(&"task-1".to_string()));
    }

    #[test]
    fn test_task_tracker_complete_task_removes_from_running() {
        let tracker = TaskTracker::new();
        tracker.start_task("task-1");
        tracker.complete_task("task-1", TaskStatus::Done);
        let running = tracker.get_running_tasks();
        assert!(!running.contains(&"task-1".to_string()));
    }

    #[test]
    fn test_task_tracker_get_running_tasks() {
        let tracker = TaskTracker::new();
        tracker.start_task("task-1");
        tracker.start_task("task-2");
        tracker.start_task("task-3");
        let running = tracker.get_running_tasks();
        assert_eq!(running.len(), 3);
        assert!(running.contains(&"task-1".to_string()));
        assert!(running.contains(&"task-2".to_string()));
        assert!(running.contains(&"task-3".to_string()));
    }

    #[test]
    fn test_task_tracker_get_stats() {
        let tracker = TaskTracker::new();
        tracker.start_task("task-1");
        tracker.start_task("task-2");
        tracker.complete_task("task-1", TaskStatus::Done);
        tracker.complete_task("task-2", TaskStatus::Blocked);
        let stats = tracker.get_stats();
        assert_eq!(stats.done_count, 1);
        assert_eq!(stats.blocked_count, 1);
        assert_eq!(stats.in_progress_count, 0);
    }

    #[test]
    fn test_task_tracker_multiple_statuses() {
        let tracker = TaskTracker::new();
        tracker.start_task("task-1");
        tracker.complete_task("task-1", TaskStatus::Done);
        tracker.complete_task("task-2", TaskStatus::Todo);
        tracker.complete_task("task-3", TaskStatus::ManualCheck);
        tracker.complete_task("task-4", TaskStatus::Blocked);
        tracker.complete_task("task-5", TaskStatus::Skipped);

        let stats = tracker.get_stats();
        assert_eq!(stats.done_count, 1);
        assert_eq!(stats.todo_count, 1);
        assert_eq!(stats.manual_check_count, 1);
        assert_eq!(stats.blocked_count, 1);
        assert_eq!(stats.skipped_count, 1);
    }
}