# Weaver

A 3-way git merge tool with a tree-sitter-powered "magic wand" for auto-resolving structurally independent changes.

Built with Tauri 2, React 19, and Monaco Editor.

## Building

Requires Rust and pnpm.

```bash
pnpm install
pnpm tauri build --bundles app
```

The binary is at `src-tauri/target/release/weaver`. The `.app` bundle is at `src-tauri/target/release/bundle/macos/weaver.app`.

For development with hot reload:

```bash
pnpm tauri dev
```

## Usage

Pass four file paths: base (common ancestor), local (yours), remote (theirs), and the output merged file.

```bash
weaver base.rs local.rs remote.rs merged.rs
```

### As a git mergetool

Register weaver in your global git config:

```bash
git config --global mergetool.weaver.cmd 'weaver "$BASE" "$LOCAL" "$REMOTE" "$MERGED"'
git config --global mergetool.weaver.trustExitCode true
```

Then when you have merge conflicts:

```bash
git mergetool --tool=weaver
```

## Quick Test

Create some test files with a conflict:

**base.rs:**
```rust
fn greet(name: &str) {
    println!("Hello, {}!", name);
}
```

**local.rs** (you changed the greeting):
```rust
fn greet(name: &str) {
    println!("Hi there, {}!", name);
}
```

**remote.rs** (they changed the greeting differently):
```rust
fn greet(name: &str) {
    println!("Hey, {}! Welcome!", name);
}
```

```bash
touch merged.rs
weaver base.rs local.rs remote.rs merged.rs
```

The app opens with three panes. The left (Local) and right (Remote) panes are read-only. The center (Result) pane is editable. Conflicting regions are highlighted in orange with buttons to accept Local, Remote, or Both.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd+N | Next unresolved conflict |
| Cmd+P | Previous unresolved conflict |
| Cmd+S | Save |
| Cmd+Q | Abort |

## Magic Wand

The Magic Wand button runs tree-sitter AST analysis on the conflict regions. If local and remote changes affect different top-level nodes (e.g. different functions), it auto-resolves by accepting both. Supports Rust, TypeScript, JavaScript, Python, Go, Java, C, and C++.

## Supported Languages

Syntax highlighting (via Monaco) works for any language. Tree-sitter structural merge is available for: Rust, TypeScript, JavaScript, Python, Go, Java, C, C++.
