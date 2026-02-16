import type { MergeSession } from "../lib/types";

interface StatusBarProps {
  session: MergeSession;
}

export default function StatusBar({ session }: StatusBarProps) {
  const conflicts = session.hunks.filter((h) => h.source === "Conflict");
  const resolved = conflicts.filter((h) => h.status !== "Unresolved");
  const allResolved = conflicts.length > 0 && resolved.length === conflicts.length;

  return (
    <div className={`status-bar ${allResolved ? "all-resolved" : ""}`}>
      <div className="status-left">
        <span>
          {conflicts.length === 0
            ? "No conflicts"
            : `${resolved.length}/${conflicts.length} conflicts resolved`}
        </span>
      </div>
      <div className="status-right">
        <span>{session.language || "plaintext"}</span>
      </div>
    </div>
  );
}
