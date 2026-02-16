import { useCallback, useEffect, useState } from "react";
import type { MergeSession, HunkStatus } from "../lib/types";
import * as commands from "../lib/tauri-commands";

export function useMergeSession() {
  const [session, setSession] = useState<MergeSession | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    commands
      .getMergeSession()
      .then((s) => {
        setSession(s);
        setLoading(false);
      })
      .catch((e) => {
        setError(String(e));
        setLoading(false);
      });
  }, []);

  const resolveHunk = useCallback(
    async (hunkId: number, status: HunkStatus) => {
      try {
        const updated = await commands.resolveHunk(hunkId, status);
        setSession(updated);
      } catch (e) {
        console.error("Failed to resolve hunk:", e);
      }
    },
    [],
  );

  const saveResult = useCallback(async () => {
    if (!session) return;
    try {
      await commands.saveResult(session.resultContent);
    } catch (e) {
      console.error("Failed to save:", e);
    }
  }, [session]);

  const setResultContent = useCallback(
    async (content: string) => {
      if (!session) return;
      setSession((prev) => (prev ? { ...prev, resultContent: content } : prev));
      try {
        await commands.updateResultContent(content);
      } catch (e) {
        console.error("Failed to update result content:", e);
      }
    },
    [session],
  );

  return { session, loading, error, resolveHunk, saveResult, setResultContent };
}
