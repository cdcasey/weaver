use parking_lot::Mutex;
use crate::merge::hunk::MergeSession;

pub struct AppState {
    pub session: Mutex<Option<MergeSession>>,
    /// Track whether the user explicitly aborted (exit code 1) vs saved (exit code 0)
    pub aborted: Mutex<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(None),
            aborted: Mutex::new(false),
        }
    }
}
