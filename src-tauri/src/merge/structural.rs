#[cfg(feature = "tree-sitter-grammars")]
mod inner {
    use tree_sitter::{Language, Parser, Node, Tree};
    use crate::merge::hunk::{HunkSource, HunkStatus, MergeHunk, MergeSession};
    use crate::merge::three_way::rebuild_result;

    fn get_language(lang: &str) -> Option<Language> {
        match lang {
            "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
            "typescript" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            "javascript" => Some(tree_sitter_javascript::LANGUAGE.into()),
            "python" => Some(tree_sitter_python::LANGUAGE.into()),
            "go" => Some(tree_sitter_go::LANGUAGE.into()),
            "java" => Some(tree_sitter_java::LANGUAGE.into()),
            "c" => Some(tree_sitter_c::LANGUAGE.into()),
            "cpp" => Some(tree_sitter_cpp::LANGUAGE.into()),
            _ => None,
        }
    }

    fn parse(source: &str, language: Language) -> Option<Tree> {
        let mut parser = Parser::new();
        parser.set_language(&language).ok()?;
        parser.parse(source, None)
    }

    /// Find top-level AST nodes that span the given byte range.
    fn find_spanning_nodes<'a>(
        root: Node<'a>,
        start_byte: usize,
        end_byte: usize,
    ) -> Vec<Node<'a>> {
        let mut nodes = Vec::new();
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.end_byte() <= start_byte {
                continue;
            }
            if child.start_byte() >= end_byte {
                break;
            }
            nodes.push(child);
        }
        nodes
    }

    /// Get byte offset for a line number (1-indexed) in the source.
    fn line_to_byte(source: &str, line: usize) -> usize {
        if line <= 1 {
            return 0;
        }
        let mut byte_offset = 0;
        let mut current_line = 1;
        for ch in source.chars() {
            if current_line >= line {
                break;
            }
            byte_offset += ch.len_utf8();
            if ch == '\n' {
                current_line += 1;
            }
        }
        byte_offset
    }

    fn end_of_line_byte(source: &str, line: usize) -> usize {
        let mut byte_offset = 0;
        let mut current_line = 1;
        for ch in source.chars() {
            byte_offset += ch.len_utf8();
            if ch == '\n' {
                if current_line == line {
                    return byte_offset;
                }
                current_line += 1;
            }
        }
        byte_offset
    }

    /// Check if changes in a conflict affect different top-level AST nodes.
    /// If local changes one function and remote changes a different function,
    /// we can auto-resolve by taking both.
    fn can_structural_resolve(
        base_tree: &Tree,
        local_tree: &Tree,
        remote_tree: &Tree,
        hunk: &MergeHunk,
        base_src: &str,
        local_src: &str,
        remote_src: &str,
    ) -> bool {
        if hunk.source != HunkSource::Conflict || hunk.status != HunkStatus::Unresolved {
            return false;
        }

        // Get byte ranges for the conflict in base
        let base_start = line_to_byte(base_src, hunk.base_range.start);
        let base_end = end_of_line_byte(base_src, hunk.base_range.end);

        // Find AST nodes spanning the conflict in base
        let base_nodes = find_spanning_nodes(base_tree.root_node(), base_start, base_end);

        if base_nodes.len() < 2 {
            // Single node or no nodes — can't split structurally
            return false;
        }

        // Get byte ranges for local/remote changes
        let local_start = line_to_byte(local_src, hunk.local_range.start);
        let local_end = end_of_line_byte(local_src, hunk.local_range.end);
        let remote_start = line_to_byte(remote_src, hunk.remote_range.start);
        let remote_end = end_of_line_byte(remote_src, hunk.remote_range.end);

        // Find what nodes local and remote touch
        let local_nodes = find_spanning_nodes(local_tree.root_node(), local_start, local_end);
        let remote_nodes = find_spanning_nodes(remote_tree.root_node(), remote_start, remote_end);

        // Check if local and remote affect different sets of node kinds at different positions
        // Simple heuristic: if the top-level node types don't overlap in their
        // approximate source positions, they're structurally independent
        let local_kinds: Vec<(&str, usize, usize)> = local_nodes
            .iter()
            .map(|n| (n.kind(), n.start_byte(), n.end_byte()))
            .collect();
        let remote_kinds: Vec<(&str, usize, usize)> = remote_nodes
            .iter()
            .map(|n| (n.kind(), n.start_byte(), n.end_byte()))
            .collect();

        // If any node ranges overlap between local and remote, it's not safe
        for (_, ls, le) in &local_kinds {
            for (_, rs, re) in &remote_kinds {
                if ls < re && rs < le {
                    return false;
                }
            }
        }

        true
    }

    pub fn structural_auto_resolve(session: &mut MergeSession) -> usize {
        let language = match get_language(&session.language) {
            Some(l) => l,
            None => return 0,
        };

        let base_tree = match parse(&session.base_content, language.clone()) {
            Some(t) => t,
            None => return 0,
        };
        let local_tree = match parse(&session.local_content, language.clone()) {
            Some(t) => t,
            None => return 0,
        };
        let remote_tree = match parse(&session.remote_content, language) {
            Some(t) => t,
            None => return 0,
        };

        let mut resolved_count = 0;
        let conflict_ids: Vec<usize> = session
            .hunks
            .iter()
            .filter(|h| h.source == HunkSource::Conflict && h.status == HunkStatus::Unresolved)
            .map(|h| h.id)
            .collect();

        for hunk_id in conflict_ids {
            let hunk: MergeHunk = match session.hunks.iter().find(|h| h.id == hunk_id) {
                Some(h) => h.clone(),
                None => continue,
            };

            if can_structural_resolve(
                &base_tree,
                &local_tree,
                &remote_tree,
                &hunk,
                &session.base_content,
                &session.local_content,
                &session.remote_content,
            ) {
                // Auto-resolve by accepting both (local first, then remote)
                let new_content = if hunk.local_content.is_empty() {
                    hunk.remote_content.clone()
                } else if hunk.remote_content.is_empty() {
                    hunk.local_content.clone()
                } else {
                    format!("{}\n{}", hunk.local_content, hunk.remote_content)
                };

                let new_result = rebuild_result(
                    &session.result_content,
                    &hunk,
                    &new_content,
                );

                // Update ranges
                let old_line_count = hunk.result_range.line_count();
                let new_line_count = new_content.lines().count();
                let line_delta = new_line_count as isize - old_line_count as isize;

                if let Some(h) = session.hunks.iter_mut().find(|h| h.id == hunk_id) {
                    h.status = HunkStatus::AutoResolved;
                    if new_line_count > 0 {
                        h.result_range.end = h.result_range.start + new_line_count - 1;
                    }
                }

                let resolved_start = hunk.result_range.start;
                for h in session.hunks.iter_mut() {
                    if h.id != hunk_id && h.result_range.start > resolved_start {
                        h.result_range.start =
                            (h.result_range.start as isize + line_delta) as usize;
                        h.result_range.end =
                            (h.result_range.end as isize + line_delta) as usize;
                    }
                }

                session.result_content = new_result;
                resolved_count += 1;
            }
        }

        resolved_count
    }
}

#[cfg(not(feature = "tree-sitter-grammars"))]
mod inner {
    use crate::merge::hunk::MergeSession;

    pub fn structural_auto_resolve(_session: &mut MergeSession) -> usize {
        0
    }
}

pub use inner::structural_auto_resolve;
