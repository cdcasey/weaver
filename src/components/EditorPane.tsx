import { useRef, useEffect } from "react";
import Editor, { type OnMount, type BeforeMount } from "@monaco-editor/react";
import type * as Monaco from "monaco-editor";
import { decorationStyles, type DecorationEntry } from "../lib/decorations";

interface EditorPaneProps {
  content: string;
  language: string;
  readOnly: boolean;
  decorations?: DecorationEntry[];
  onContentChange?: (value: string) => void;
  onEditorMount?: (editor: Monaco.editor.IStandaloneCodeEditor) => void;
  onScrollChange?: (scrollTop: number) => void;
  scrollTop?: number;
}

let stylesInjected = false;

export default function EditorPane({
  content,
  language,
  readOnly,
  decorations,
  onContentChange,
  onEditorMount,
  onScrollChange,
  scrollTop,
}: EditorPaneProps) {
  const editorRef = useRef<Monaco.editor.IStandaloneCodeEditor | null>(null);
  const decorationIds = useRef<string[]>([]);
  const isSyncing = useRef(false);

  const handleBeforeMount: BeforeMount = (monaco) => {
    if (!stylesInjected) {
      const style = document.createElement("style");
      style.textContent = decorationStyles;
      document.head.appendChild(style);
      stylesInjected = true;
    }

    monaco.editor.defineTheme("weaver-dark", {
      base: "vs-dark",
      inherit: true,
      rules: [],
      colors: {
        "editor.background": "#1e1e1e",
      },
    });
  };

  const handleMount: OnMount = (editor) => {
    editorRef.current = editor;
    editor.updateOptions({
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      renderLineHighlight: "none",
      overviewRulerBorder: false,
      hideCursorInOverviewRuler: true,
      glyphMargin: true,
      folding: false,
      lineNumbersMinChars: 4,
      padding: { top: 4 },
    });

    if (onScrollChange) {
      editor.onDidScrollChange((e) => {
        if (!isSyncing.current) {
          onScrollChange(e.scrollTop);
        }
      });
    }

    onEditorMount?.(editor);
  };

  useEffect(() => {
    const editor = editorRef.current;
    if (!editor || !decorations) return;
    decorationIds.current = editor.deltaDecorations(
      decorationIds.current,
      decorations,
    );
  }, [decorations]);

  useEffect(() => {
    const editor = editorRef.current;
    if (!editor || scrollTop === undefined) return;
    isSyncing.current = true;
    editor.setScrollTop(scrollTop);
    requestAnimationFrame(() => {
      isSyncing.current = false;
    });
  }, [scrollTop]);

  return (
    <div className="editor-pane">
      <Editor
        height="100%"
        language={language}
        value={content}
        theme="weaver-dark"
        beforeMount={handleBeforeMount}
        onMount={handleMount}
        onChange={(value) => onContentChange?.(value ?? "")}
        options={{
          readOnly,
          domReadOnly: readOnly,
          wordWrap: "off",
          automaticLayout: true,
          scrollbar: {
            vertical: "visible",
            horizontal: "visible",
            verticalScrollbarSize: 10,
          },
        }}
      />
    </div>
  );
}
