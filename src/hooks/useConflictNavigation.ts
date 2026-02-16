import { useEffect, type RefObject } from "react";
import type * as Monaco from "monaco-editor";
import type { MergeHunk } from "../lib/types";

export function useConflictNavigation(
  editorRef: RefObject<Monaco.editor.IStandaloneCodeEditor | null>,
  hunks: MergeHunk[],
) {
  useEffect(() => {
    const editor = editorRef.current;
    if (!editor) return;

    const unresolvedConflicts = () =>
      hunks.filter((h) => h.source === "Conflict" && h.status === "Unresolved");

    // Cmd+N → next conflict
    editor.addCommand(
      // KeyMod.CtrlCmd | KeyCode.KeyN
      2048 | 44,
      () => {
        const conflicts = unresolvedConflicts();
        if (conflicts.length === 0) return;
        const currentLine = editor.getPosition()?.lineNumber ?? 0;
        const next =
          conflicts.find((h) => h.resultRange.start > currentLine) ?? conflicts[0];
        editor.revealLineInCenter(next.resultRange.start);
        editor.setPosition({ lineNumber: next.resultRange.start, column: 1 });
      },
    );

    // Cmd+P → previous conflict
    editor.addCommand(
      // KeyMod.CtrlCmd | KeyCode.KeyP
      2048 | 46,
      () => {
        const conflicts = unresolvedConflicts();
        if (conflicts.length === 0) return;
        const currentLine = editor.getPosition()?.lineNumber ?? Infinity;
        const prev =
          [...conflicts].reverse().find((h) => h.resultRange.start < currentLine) ??
          conflicts[conflicts.length - 1];
        editor.revealLineInCenter(prev.resultRange.start);
        editor.setPosition({ lineNumber: prev.resultRange.start, column: 1 });
      },
    );
  }, [editorRef, hunks]);
}
