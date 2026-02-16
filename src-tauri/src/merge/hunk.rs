use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HunkStatus {
    Unresolved,
    AcceptedLocal,
    AcceptedRemote,
    AcceptedBoth,
    CustomEdit,
    AutoResolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HunkSource {
    Local,
    Remote,
    Both,
    Conflict,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

impl LineRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.start == 0 && self.end == 0
    }

    pub fn line_count(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.end - self.start + 1
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeHunk {
    pub id: usize,
    pub base_range: LineRange,
    pub local_range: LineRange,
    pub remote_range: LineRange,
    pub result_range: LineRange,
    pub source: HunkSource,
    pub status: HunkStatus,
    pub local_content: String,
    pub remote_content: String,
    pub base_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeSession {
    pub base_path: String,
    pub local_path: String,
    pub remote_path: String,
    pub merged_path: String,
    pub base_content: String,
    pub local_content: String,
    pub remote_content: String,
    pub result_content: String,
    pub hunks: Vec<MergeHunk>,
    pub language: String,
}
