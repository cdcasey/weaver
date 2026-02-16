import type { MergeHunk } from "./types";
import type * as Monaco from "monaco-editor";

export type DecorationEntry = Monaco.editor.IModelDeltaDecoration;

const COLORS = {
  localBg: "rgba(78, 201, 176, 0.15)",
  localBorder: "rgba(78, 201, 176, 0.5)",
  remoteBg: "rgba(86, 156, 214, 0.15)",
  remoteBorder: "rgba(86, 156, 214, 0.5)",
  conflictBg: "rgba(255, 140, 0, 0.15)",
  conflictBorder: "rgba(255, 140, 0, 0.5)",
  resolvedBg: "rgba(100, 100, 100, 0.1)",
};

function rangeForLines(
  monaco: typeof Monaco,
  startLine: number,
  endLine: number,
): Monaco.IRange {
  return new monaco.Range(startLine, 1, endLine, 1);
}

export function buildLocalDecorations(
  monaco: typeof Monaco,
  hunks: MergeHunk[],
): DecorationEntry[] {
  return hunks
    .filter((h) => h.localRange.start > 0)
    .map((hunk) => {
      const isConflict = hunk.source === "Conflict";
      const isResolved = hunk.status !== "Unresolved";
      return {
        range: rangeForLines(monaco, hunk.localRange.start, hunk.localRange.end),
        options: {
          isWholeLine: true,
          className: isResolved
            ? "hunk-resolved"
            : isConflict
              ? "hunk-conflict"
              : "hunk-local",
          glyphMarginClassName: isConflict && !isResolved ? "glyph-conflict" : undefined,
        },
      };
    });
}

export function buildRemoteDecorations(
  monaco: typeof Monaco,
  hunks: MergeHunk[],
): DecorationEntry[] {
  return hunks
    .filter((h) => h.remoteRange.start > 0)
    .map((hunk) => {
      const isConflict = hunk.source === "Conflict";
      const isResolved = hunk.status !== "Unresolved";
      return {
        range: rangeForLines(monaco, hunk.remoteRange.start, hunk.remoteRange.end),
        options: {
          isWholeLine: true,
          className: isResolved
            ? "hunk-resolved"
            : isConflict
              ? "hunk-conflict"
              : "hunk-remote",
        },
      };
    });
}

export function buildResultDecorations(
  monaco: typeof Monaco,
  hunks: MergeHunk[],
): DecorationEntry[] {
  return hunks
    .filter((h) => h.resultRange.start > 0)
    .map((hunk) => {
      const isConflict = hunk.source === "Conflict";
      const isResolved = hunk.status !== "Unresolved";
      return {
        range: rangeForLines(monaco, hunk.resultRange.start, hunk.resultRange.end),
        options: {
          isWholeLine: true,
          className: isResolved
            ? "hunk-resolved"
            : isConflict
              ? "hunk-conflict-result"
              : "hunk-clean",
        },
      };
    });
}

export const decorationStyles = `
  .hunk-local { background: ${COLORS.localBg}; border-left: 3px solid ${COLORS.localBorder}; }
  .hunk-remote { background: ${COLORS.remoteBg}; border-left: 3px solid ${COLORS.remoteBorder}; }
  .hunk-conflict { background: ${COLORS.conflictBg}; border-left: 3px solid ${COLORS.conflictBorder}; }
  .hunk-conflict-result { background: ${COLORS.conflictBg}; border-left: 3px solid ${COLORS.conflictBorder}; }
  .hunk-resolved { background: ${COLORS.resolvedBg}; }
  .hunk-clean { background: ${COLORS.localBg}; }
  .glyph-conflict { background: orange; width: 6px !important; margin-left: 3px; border-radius: 2px; }
`;
