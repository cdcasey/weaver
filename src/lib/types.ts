export type HunkStatus =
  | "Unresolved"
  | "AcceptedLocal"
  | "AcceptedRemote"
  | "AcceptedBoth"
  | "CustomEdit"
  | "AutoResolved";

export type HunkSource = "Local" | "Remote" | "Both" | "Conflict";

export interface LineRange {
  start: number;
  end: number;
}

export interface MergeHunk {
  id: number;
  baseRange: LineRange;
  localRange: LineRange;
  remoteRange: LineRange;
  resultRange: LineRange;
  source: HunkSource;
  status: HunkStatus;
  localContent: string;
  remoteContent: string;
  baseContent: string;
}

export interface MergeSession {
  basePath: string;
  localPath: string;
  remotePath: string;
  mergedPath: string;
  baseContent: string;
  localContent: string;
  remoteContent: string;
  resultContent: string;
  hunks: MergeHunk[];
  language: string;
}
