use similar::{ChangeTag, TextDiff};
use super::hunk::{HunkSource, HunkStatus, LineRange, MergeHunk};

/// A change region from a 2-way diff (base vs side).
#[derive(Debug, Clone)]
struct DiffRegion {
    base_start: usize,
    base_end: usize,
    side_start: usize,
    side_end: usize,
}

/// Extract diff regions between base and a side (local or remote).
fn extract_regions(base: &str, side: &str) -> Vec<DiffRegion> {
    let diff = TextDiff::from_lines(base, side);
    let mut regions = Vec::new();
    let mut base_line = 1usize;
    let mut side_line = 1usize;
    let mut in_change = false;
    let mut region = DiffRegion {
        base_start: 0,
        base_end: 0,
        side_start: 0,
        side_end: 0,
    };

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => {
                if in_change {
                    region.base_end = base_line - 1;
                    region.side_end = side_line - 1;
                    regions.push(region.clone());
                    in_change = false;
                }
                base_line += 1;
                side_line += 1;
            }
            ChangeTag::Delete => {
                if !in_change {
                    region = DiffRegion {
                        base_start: base_line,
                        base_end: base_line,
                        side_start: side_line,
                        side_end: side_line.saturating_sub(1),
                    };
                    in_change = true;
                }
                base_line += 1;
            }
            ChangeTag::Insert => {
                if !in_change {
                    region = DiffRegion {
                        base_start: base_line,
                        base_end: base_line.saturating_sub(1),
                        side_start: side_line,
                        side_end: side_line,
                    };
                    in_change = true;
                }
                side_line += 1;
            }
        }
    }

    if in_change {
        region.base_end = base_line - 1;
        region.side_end = side_line - 1;
        regions.push(region);
    }

    regions
}

/// Check if two base ranges overlap.
fn ranges_overlap(a: &DiffRegion, b: &DiffRegion) -> bool {
    // Handle pure insertions (base_end < base_start)
    let a_start = a.base_start;
    let a_end = a.base_end.max(a.base_start);
    let b_start = b.base_start;
    let b_end = b.base_end.max(b.base_start);

    a_start <= b_end && b_start <= a_end
}

/// Get lines from content (1-indexed range, inclusive).
fn get_lines(content: &str, start: usize, end: usize) -> String {
    if start == 0 || end < start {
        return String::new();
    }
    let lines: Vec<&str> = content.lines().collect();
    let s = (start - 1).min(lines.len());
    let e = end.min(lines.len());
    lines[s..e].join("\n")
}

/// Perform 3-way merge producing a list of merge hunks.
pub fn three_way_merge(base: &str, local: &str, remote: &str) -> (Vec<MergeHunk>, String) {
    let local_regions = extract_regions(base, local);
    let remote_regions = extract_regions(base, remote);

    let mut hunks: Vec<MergeHunk> = Vec::new();
    let mut hunk_id = 0;

    let mut li = 0;
    let mut ri = 0;

    while li < local_regions.len() || ri < remote_regions.len() {
        let lr = local_regions.get(li);
        let rr = remote_regions.get(ri);

        match (lr, rr) {
            (Some(l), Some(r)) if ranges_overlap(l, r) => {
                // Conflict: both sides changed overlapping base region
                hunks.push(MergeHunk {
                    id: hunk_id,
                    base_range: LineRange::new(
                        l.base_start.min(r.base_start),
                        l.base_end.max(r.base_end),
                    ),
                    local_range: LineRange::new(l.side_start, l.side_end),
                    remote_range: LineRange::new(r.side_start, r.side_end),
                    result_range: LineRange::empty(),
                    source: HunkSource::Conflict,
                    status: HunkStatus::Unresolved,
                    local_content: get_lines(local, l.side_start, l.side_end),
                    remote_content: get_lines(remote, r.side_start, r.side_end),
                    base_content: get_lines(
                        base,
                        l.base_start.min(r.base_start),
                        l.base_end.max(r.base_end),
                    ),
                });
                hunk_id += 1;
                li += 1;
                ri += 1;
            }
            (Some(l), Some(r)) => {
                // No overlap — take whichever comes first in the base
                if l.base_start <= r.base_start {
                    hunks.push(MergeHunk {
                        id: hunk_id,
                        base_range: LineRange::new(l.base_start, l.base_end),
                        local_range: LineRange::new(l.side_start, l.side_end),
                        remote_range: LineRange::empty(),
                        result_range: LineRange::empty(),
                        source: HunkSource::Local,
                        status: HunkStatus::AutoResolved,
                        local_content: get_lines(local, l.side_start, l.side_end),
                        remote_content: String::new(),
                        base_content: get_lines(base, l.base_start, l.base_end),
                    });
                    hunk_id += 1;
                    li += 1;
                } else {
                    hunks.push(MergeHunk {
                        id: hunk_id,
                        base_range: LineRange::new(r.base_start, r.base_end),
                        local_range: LineRange::empty(),
                        remote_range: LineRange::new(r.side_start, r.side_end),
                        result_range: LineRange::empty(),
                        source: HunkSource::Remote,
                        status: HunkStatus::AutoResolved,
                        local_content: String::new(),
                        remote_content: get_lines(remote, r.side_start, r.side_end),
                        base_content: get_lines(base, r.base_start, r.base_end),
                    });
                    hunk_id += 1;
                    ri += 1;
                }
            }
            (Some(l), None) => {
                hunks.push(MergeHunk {
                    id: hunk_id,
                    base_range: LineRange::new(l.base_start, l.base_end),
                    local_range: LineRange::new(l.side_start, l.side_end),
                    remote_range: LineRange::empty(),
                    result_range: LineRange::empty(),
                    source: HunkSource::Local,
                    status: HunkStatus::AutoResolved,
                    local_content: get_lines(local, l.side_start, l.side_end),
                    remote_content: String::new(),
                    base_content: get_lines(base, l.base_start, l.base_end),
                });
                hunk_id += 1;
                li += 1;
            }
            (None, Some(r)) => {
                hunks.push(MergeHunk {
                    id: hunk_id,
                    base_range: LineRange::new(r.base_start, r.base_end),
                    local_range: LineRange::empty(),
                    remote_range: LineRange::new(r.side_start, r.side_end),
                    result_range: LineRange::empty(),
                    source: HunkSource::Remote,
                    status: HunkStatus::AutoResolved,
                    local_content: String::new(),
                    remote_content: get_lines(remote, r.side_start, r.side_end),
                    base_content: get_lines(base, r.base_start, r.base_end),
                });
                hunk_id += 1;
                ri += 1;
            }
            (None, None) => break,
        }
    }

    // Build the result content by applying non-conflicting changes
    let result = build_result(base, local, remote, &hunks);

    // Compute result ranges
    compute_result_ranges(&mut hunks, &result);

    (hunks, result)
}

/// Build result content from the merge.
/// For non-conflicting hunks, apply the change. For conflicts, keep the base content
/// (user must resolve).
fn build_result(base: &str, _local: &str, _remote: &str, hunks: &[MergeHunk]) -> String {
    let base_lines: Vec<&str> = base.lines().collect();
    let mut result_lines: Vec<String> = Vec::new();
    let mut base_pos = 0; // 0-indexed

    // Sort hunks by base_range.start
    let mut sorted_hunks: Vec<&MergeHunk> = hunks.iter().collect();
    sorted_hunks.sort_by_key(|h| h.base_range.start);

    for hunk in &sorted_hunks {
        let hunk_base_start = if hunk.base_range.start > 0 {
            hunk.base_range.start - 1
        } else {
            base_pos
        };
        let hunk_base_end = hunk.base_range.end;

        // Copy unchanged lines before this hunk
        while base_pos < hunk_base_start && base_pos < base_lines.len() {
            result_lines.push(base_lines[base_pos].to_string());
            base_pos += 1;
        }

        match hunk.source {
            HunkSource::Local => {
                // Apply local change
                if !hunk.local_content.is_empty() {
                    for line in hunk.local_content.lines() {
                        result_lines.push(line.to_string());
                    }
                }
            }
            HunkSource::Remote => {
                // Apply remote change
                if !hunk.remote_content.is_empty() {
                    for line in hunk.remote_content.lines() {
                        result_lines.push(line.to_string());
                    }
                }
            }
            HunkSource::Conflict => {
                // For conflicts, insert conflict markers
                result_lines.push("<<<<<<< LOCAL".to_string());
                for line in hunk.local_content.lines() {
                    result_lines.push(line.to_string());
                }
                result_lines.push("=======".to_string());
                for line in hunk.remote_content.lines() {
                    result_lines.push(line.to_string());
                }
                result_lines.push(">>>>>>> REMOTE".to_string());
            }
            HunkSource::Both => {
                // Both sides made same change — take either
                if !hunk.local_content.is_empty() {
                    for line in hunk.local_content.lines() {
                        result_lines.push(line.to_string());
                    }
                }
            }
        }

        // Skip over the base lines this hunk replaces
        base_pos = hunk_base_end.min(base_lines.len());
    }

    // Copy remaining base lines
    while base_pos < base_lines.len() {
        result_lines.push(base_lines[base_pos].to_string());
        base_pos += 1;
    }

    result_lines.join("\n")
}

/// Compute result_range for each hunk based on the generated result content.
fn compute_result_ranges(hunks: &mut Vec<MergeHunk>, result: &str) {
    let result_lines: Vec<&str> = result.lines().collect();

    for hunk in hunks.iter_mut() {
        let search_content = match hunk.source {
            HunkSource::Conflict => {
                format!("<<<<<<< LOCAL\n{}\n=======\n{}\n>>>>>>> REMOTE",
                    hunk.local_content, hunk.remote_content)
            }
            HunkSource::Local => hunk.local_content.clone(),
            HunkSource::Remote => hunk.remote_content.clone(),
            HunkSource::Both => hunk.local_content.clone(),
        };

        if search_content.is_empty() {
            hunk.result_range = LineRange::empty();
            continue;
        }

        let search_lines: Vec<&str> = search_content.lines().collect();
        if search_lines.is_empty() {
            hunk.result_range = LineRange::empty();
            continue;
        }

        // Find the search content in result
        for i in 0..result_lines.len() {
            if i + search_lines.len() > result_lines.len() {
                break;
            }
            let matches = search_lines
                .iter()
                .enumerate()
                .all(|(j, sl)| result_lines[i + j] == *sl);
            if matches {
                hunk.result_range = LineRange::new(i + 1, i + search_lines.len());
                break;
            }
        }
    }
}

/// Rebuild result content after a hunk resolution.
pub fn rebuild_result(
    current_result: &str,
    hunk: &MergeHunk,
    new_content: &str,
) -> String {
    if hunk.result_range.is_empty() {
        return current_result.to_string();
    }

    let lines: Vec<&str> = current_result.lines().collect();
    let mut result = Vec::new();

    // Lines before the hunk
    for i in 0..(hunk.result_range.start - 1).min(lines.len()) {
        result.push(lines[i].to_string());
    }

    // Insert new content
    if !new_content.is_empty() {
        for line in new_content.lines() {
            result.push(line.to_string());
        }
    }

    // Lines after the hunk
    for i in hunk.result_range.end..lines.len() {
        result.push(lines[i].to_string());
    }

    result.join("\n")
}
