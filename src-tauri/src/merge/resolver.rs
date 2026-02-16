use super::hunk::{HunkSource, HunkStatus, MergeSession};
use super::three_way::rebuild_result;

/// Resolve a single hunk by accepting local, remote, or both content.
pub fn resolve_hunk(session: &mut MergeSession, hunk_id: usize, status: HunkStatus) {
    let hunk = match session.hunks.iter().find(|h| h.id == hunk_id) {
        Some(h) => h.clone(),
        None => return,
    };

    let new_content = match status {
        HunkStatus::AcceptedLocal => hunk.local_content.clone(),
        HunkStatus::AcceptedRemote => hunk.remote_content.clone(),
        HunkStatus::AcceptedBoth => {
            if hunk.local_content.is_empty() {
                hunk.remote_content.clone()
            } else if hunk.remote_content.is_empty() {
                hunk.local_content.clone()
            } else {
                format!("{}\n{}", hunk.local_content, hunk.remote_content)
            }
        }
        _ => return,
    };

    // Rebuild result content
    let new_result = rebuild_result(&session.result_content, &hunk, &new_content);

    // Recompute result ranges after rebuilding
    let old_line_count = hunk.result_range.line_count();
    let new_line_count = if new_content.is_empty() {
        0
    } else {
        new_content.lines().count()
    };
    let line_delta = new_line_count as isize - old_line_count as isize;

    // Update the resolved hunk
    if let Some(h) = session.hunks.iter_mut().find(|h| h.id == hunk_id) {
        h.status = status;
        let old_start = h.result_range.start;
        if new_line_count > 0 {
            h.result_range.end = old_start + new_line_count - 1;
        } else {
            h.result_range.start = 0;
            h.result_range.end = 0;
        }
    }

    // Adjust result ranges for subsequent hunks
    let resolved_start = hunk.result_range.start;
    for h in session.hunks.iter_mut() {
        if h.id != hunk_id && h.result_range.start > resolved_start {
            if line_delta >= 0 {
                h.result_range.start = (h.result_range.start as isize + line_delta) as usize;
                h.result_range.end = (h.result_range.end as isize + line_delta) as usize;
            } else {
                let delta = (-line_delta) as usize;
                h.result_range.start = h.result_range.start.saturating_sub(delta);
                h.result_range.end = h.result_range.end.saturating_sub(delta);
            }
        }
    }

    session.result_content = new_result;
}

/// Auto-resolve all hunks that aren't true conflicts.
pub fn auto_resolve_non_conflicts(session: &mut MergeSession) {
    // For now, auto-resolve doesn't handle true conflicts.
    // Tree-sitter structural merge (Phase 5) will handle those.
    // Just mark any remaining non-conflict unresolved hunks.
    for hunk in session.hunks.iter_mut() {
        if hunk.status == HunkStatus::Unresolved && hunk.source != HunkSource::Conflict {
            hunk.status = HunkStatus::AutoResolved;
        }
    }
}
