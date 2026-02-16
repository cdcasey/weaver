import { invoke } from "@tauri-apps/api/core";
import type { MergeSession, HunkStatus } from "./types";

export async function getMergeSession(): Promise<MergeSession> {
  return invoke<MergeSession>("get_merge_session");
}

export async function resolveHunk(
  hunkId: number,
  status: HunkStatus,
): Promise<MergeSession> {
  return invoke<MergeSession>("resolve_hunk", { hunkId, status });
}

export async function saveResult(content: string): Promise<void> {
  return invoke("save_result", { content });
}

export async function updateResultContent(content: string): Promise<void> {
  return invoke("update_result_content", { content });
}

export async function abortMerge(): Promise<void> {
  return invoke("abort_merge");
}

export async function autoResolve(): Promise<MergeSession> {
  return invoke<MergeSession>("auto_resolve");
}

export async function registerGitMergetool(): Promise<string> {
  return invoke<string>("register_git_mergetool");
}
