# Weaver

A 3-way git merge tool built with Tauri 2.x, React 19, and Monaco Editor. Includes a "magic wand" feature that uses tree-sitter AST analysis to auto-resolve structurally non-conflicting changes.

## Architecture

```
src/                          # React + TypeScript frontend
  components/
    MergeView.tsx             # 3-pane container: Local | Result | Remote
    EditorPane.tsx            # Monaco wrapper (readOnly for L/R, editable for Result)
    HunkActions.tsx           # Per-hunk accept/reject overlay buttons
    Toolbar.tsx               # Save, Abort, Magic Wand
    StatusBar.tsx             # Resolution progress ("3/7 conflicts resolved")
  hooks/
    useMergeSession.ts        # Fetch session from Tauri backend
    useScrollSync.ts          # Synchronized scrolling with jitter guard
    useConflictNavigation.ts  # Cmd+N/P to jump between conflicts
  lib/
    tauri-commands.ts         # Typed invoke() wrappers
    decorations.ts            # Monaco decoration builders (colors per hunk type)
    types.ts                  # Shared TypeScript types

src-tauri/src/                # Rust backend
  main.rs                     # Entry point
  lib.rs                      # Tauri command registration, language detection, conflict marker parsing
  cli.rs                      # Parse $BASE/$LOCAL/$REMOTE/$MERGED from CLI args
  state.rs                    # Tauri managed state (AppState with Mutex<MergeSession>)
  safety.rs                   # .bak file creation before writing
  git/
    repo.rs                   # Register as git mergetool in ~/.gitconfig
    diff.rs                   # (placeholder for git2-based operations)
  merge/
    hunk.rs                   # Core data structures: MergeHunk, MergeSession, HunkStatus, HunkSource
    three_way.rs              # Line-based 3-way merge using `similar` crate
    resolver.rs               # Hunk resolution logic (accept local/remote/both)
    structural.rs             # Tree-sitter AST merge (magic wand)
```

## Build & Dev

Rust is at `~/.cargo/bin/` — prefix commands with `export PATH="$HOME/.cargo/bin:$PATH"` or ensure it's on PATH.

```bash
pnpm tauri dev                    # Dev mode with hot reload
pnpm tauri build --bundles app    # Release .app bundle (~11MB)
npx tsc --noEmit                  # TypeScript type check only
cargo check                       # Rust type check only (run from src-tauri/)
```

## Testing

```bash
# With release binary:
./src-tauri/target/release/weaver /tmp/weaver-test/base.rs /tmp/weaver-test/local.rs /tmp/weaver-test/remote.rs /tmp/weaver-test/merged.rs

# With dev mode (double -- to pass args through pnpm and tauri):
pnpm tauri dev -- -- /tmp/weaver-test/base.rs /tmp/weaver-test/local.rs /tmp/weaver-test/remote.rs /tmp/weaver-test/merged.rs
```

Test files are at `/tmp/weaver-test/{base,local,remote,merged}.rs`.

## Key Dependencies

**Rust:** `tauri 2`, `similar 2` (diff), `tree-sitter 0.24` + 8 grammar crates (behind `tree-sitter-grammars` feature), `serde`, `parking_lot`, `thiserror`

**Frontend:** `@monaco-editor/react 4.7` + `monaco-editor 0.55` (local bundle, NOT CDN), `@tauri-apps/api 2`, `react 19`, `react-resizable-panels 4.6`

## Dependency API Gotchas

- `react-resizable-panels` v4.6 uses `Group`/`Separator`/`orientation` — NOT the older `PanelGroup`/`PanelResizeHandle`/`direction` API
- Monaco must use local bundle loading (no CDN — offline guarantee for dev tools)
- Tree-sitter grammars are optional via cargo feature `tree-sitter-grammars` (default on)
- DMG bundling may fail; use `--bundles app` for `.app` bundle only

## Git Integration

```bash
# Register as mergetool:
git config --global mergetool.weaver.cmd '/path/to/weaver "$BASE" "$LOCAL" "$REMOTE" "$MERGED"'
git config --global mergetool.weaver.trustExitCode true

# Use during merge conflicts:
git mergetool --tool=weaver
```

Exit codes: 0 = resolved (saved), 1 = aborted.

## Keyboard Shortcuts

- **Cmd+N** — Next unresolved conflict
- **Cmd+P** — Previous unresolved conflict
- **Cmd+S** — Save result
- **Cmd+Q** — Abort merge

## How the 3-Way Merge Works

1. Compute `base→local` and `base→remote` diffs using `similar` crate
2. Walk both diffs to find overlapping base regions (conflicts) vs non-overlapping (auto-resolved)
3. Build result content: auto-apply non-conflicting changes, insert conflict markers for true conflicts
4. Frontend renders colored decorations (green=local, blue=remote, orange=conflict)
5. User resolves conflicts via overlay buttons or manual editing
6. Magic wand: tree-sitter parses all 3 versions, checks if conflict spans different top-level AST nodes (e.g. different functions), auto-resolves by taking both if structurally independent

## Known Issues & Missing Functionality

- **Save doesn't exit** — `git mergetool` expects the tool to exit after saving. Currently Save writes the file but doesn't close the window. Save should trigger `app.exit(0)`.
- **Hunk accept/reject buttons in side panes** — The Local and Remote panes need inline buttons (e.g. checkmark/X) on each hunk so users can click to accept or reject a change into the result. Currently `HunkActions.tsx` only renders buttons in the result pane for conflicts. The side panes need their own buttons for all change hunks (not just conflicts).
- **Conflict markers in result pane** — Unresolved conflicts show raw `<<<<<<< LOCAL` / `=======` / `>>>>>>> REMOTE` markers in the result editor. IntelliJ-style UX would hide these and show a clean "pick a side" view instead.

## Future Enhancements

- **Repo-aware mode** — `weaver` or `weaver .` in a repo directory detects the current git repo, finds all files with merge conflicts (via `git ls-files --unmerged` or similar), and presents a file picker to resolve them one by one. No need to manually pass file paths or configure as `git mergetool`.
- **Single-file mode** — `weaver path/to/conflicted_file.rs` opens a file containing `<<<<<<<` conflict markers, extracts local/remote from markers, uses the marker-stripped version as a synthetic base, and writes back to the same file. The conflict marker parser already exists in `lib.rs`; just needs a single-arg CLI path and synthetic base construction.
- **Cmd+1 / Cmd+2 shortcuts** — Accept local / accept remote for the conflict under the cursor
- **Line-level scroll mapping** — Current scroll sync is simple offset-based. Proper implementation would map lines through hunks (1:1 for unchanged regions, proportional interpolation for conflicts)
- **git2 integration** — Use libgit2 for richer repo context instead of shelling out to `git` CLI
