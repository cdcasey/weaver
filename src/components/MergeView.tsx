import { useMemo, useRef, useCallback, useState } from "react";
import { Panel, Group, Separator } from "react-resizable-panels";
import type * as Monaco from "monaco-editor";
import EditorPane from "./EditorPane";
import HunkActions from "./HunkActions";
import { useScrollSync } from "../hooks/useScrollSync";
import { useConflictNavigation } from "../hooks/useConflictNavigation";
import {
  buildLocalDecorations,
  buildRemoteDecorations,
  buildResultDecorations,
} from "../lib/decorations";
import type { MergeSession, HunkStatus } from "../lib/types";

interface MergeViewProps {
  session: MergeSession;
  resolveHunk: (hunkId: number, status: HunkStatus) => void;
  onResultEdit: (content: string) => void;
}

export default function MergeView({
  session,
  resolveHunk,
  onResultEdit,
}: MergeViewProps) {
  const localEditorRef = useRef<Monaco.editor.IStandaloneCodeEditor | null>(null);
  const resultEditorRef = useRef<Monaco.editor.IStandaloneCodeEditor | null>(null);
  const remoteEditorRef = useRef<Monaco.editor.IStandaloneCodeEditor | null>(null);
  const [monacoInstance, setMonacoInstance] = useState<typeof Monaco | null>(null);

  const { scrollTop, handleScroll } = useScrollSync();

  const localDecorations = useMemo(
    () => monacoInstance ? buildLocalDecorations(monacoInstance, session.hunks) : [],
    [monacoInstance, session.hunks],
  );

  const remoteDecorations = useMemo(
    () => monacoInstance ? buildRemoteDecorations(monacoInstance, session.hunks) : [],
    [monacoInstance, session.hunks],
  );

  const resultDecorations = useMemo(
    () => monacoInstance ? buildResultDecorations(monacoInstance, session.hunks) : [],
    [monacoInstance, session.hunks],
  );

  const handleLocalMount = useCallback((editor: Monaco.editor.IStandaloneCodeEditor) => {
    localEditorRef.current = editor;
    // Grab monaco instance from the editor's model
    const m = (window as unknown as { monaco?: typeof Monaco }).monaco;
    if (m && !monacoInstance) setMonacoInstance(m);
  }, [monacoInstance]);

  const handleResultMount = useCallback((editor: Monaco.editor.IStandaloneCodeEditor) => {
    resultEditorRef.current = editor;
  }, []);

  const handleRemoteMount = useCallback((editor: Monaco.editor.IStandaloneCodeEditor) => {
    remoteEditorRef.current = editor;
  }, []);

  useConflictNavigation(resultEditorRef, session.hunks);

  return (
    <div className="merge-view">
      <div className="pane-labels">
        <div className="pane-label local">Local (Ours)</div>
        <div className="pane-label result">Result</div>
        <div className="pane-label remote">Remote (Theirs)</div>
      </div>
      <div className="editors-container">
        <Group orientation="horizontal">
          <Panel defaultSize={33} minSize={15}>
            <EditorPane
              content={session.localContent}
              language={session.language}
              readOnly={true}
              decorations={localDecorations}
              onEditorMount={handleLocalMount}
              onScrollChange={handleScroll}
              scrollTop={scrollTop}
            />
          </Panel>
          <Separator />
          <Panel defaultSize={34} minSize={20}>
            <div style={{ position: "relative", height: "100%" }}>
              <EditorPane
                content={session.resultContent}
                language={session.language}
                readOnly={false}
                decorations={resultDecorations}
                onContentChange={onResultEdit}
                onEditorMount={handleResultMount}
                onScrollChange={handleScroll}
                scrollTop={scrollTop}
              />
              <HunkActions
                hunks={session.hunks}
                editor={resultEditorRef.current}
                onResolve={resolveHunk}
              />
            </div>
          </Panel>
          <Separator />
          <Panel defaultSize={33} minSize={15}>
            <EditorPane
              content={session.remoteContent}
              language={session.language}
              readOnly={true}
              decorations={remoteDecorations}
              onEditorMount={handleRemoteMount}
              onScrollChange={handleScroll}
              scrollTop={scrollTop}
            />
          </Panel>
        </Group>
      </div>
    </div>
  );
}
