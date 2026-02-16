import { useEffect, useState } from "react";
import type * as Monaco from "monaco-editor";
import type { MergeHunk, HunkStatus } from "../lib/types";

interface HunkActionsProps {
  hunks: MergeHunk[];
  editor: Monaco.editor.IStandaloneCodeEditor | null;
  onResolve: (hunkId: number, status: HunkStatus) => void;
}

interface ButtonPosition {
  hunkId: number;
  top: number;
}

export default function HunkActions({ hunks, editor, onResolve }: HunkActionsProps) {
  const [positions, setPositions] = useState<ButtonPosition[]>([]);

  useEffect(() => {
    if (!editor) return;

    const updatePositions = () => {
      const unresolvedConflicts = hunks.filter(
        (h) => h.source === "Conflict" && h.status === "Unresolved",
      );

      const newPositions = unresolvedConflicts.map((hunk) => {
        const top = editor.getTopForLineNumber(hunk.resultRange.start);
        const scrollTop = editor.getScrollTop();
        return {
          hunkId: hunk.id,
          top: top - scrollTop - 2,
        };
      });

      setPositions(newPositions);
    };

    updatePositions();
    const disposable = editor.onDidScrollChange(updatePositions);
    const layoutDisposable = editor.onDidLayoutChange(updatePositions);

    return () => {
      disposable.dispose();
      layoutDisposable.dispose();
    };
  }, [hunks, editor]);

  if (!editor) return null;

  return (
    <>
      {positions.map((pos) => (
        <div
          key={pos.hunkId}
          className="hunk-actions"
          style={{ top: pos.top, right: 20 }}
        >
          <button
            className="accept-local"
            onClick={() => onResolve(pos.hunkId, "AcceptedLocal")}
            title="Accept Local (Cmd+1)"
          >
            Local
          </button>
          <button
            className="accept-remote"
            onClick={() => onResolve(pos.hunkId, "AcceptedRemote")}
            title="Accept Remote (Cmd+2)"
          >
            Remote
          </button>
          <button
            className="accept-both"
            onClick={() => onResolve(pos.hunkId, "AcceptedBoth")}
            title="Accept Both"
          >
            Both
          </button>
        </div>
      ))}
    </>
  );
}
