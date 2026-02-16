import { abortMerge, autoResolve } from "../lib/tauri-commands";

interface ToolbarProps {
  filename: string;
  onSave: () => void;
}

export default function Toolbar({ filename, onSave }: ToolbarProps) {
  const handleAbort = async () => {
    try {
      await abortMerge();
    } catch (e) {
      console.error("Abort failed:", e);
    }
  };

  const handleMagicWand = async () => {
    try {
      await autoResolve();
      // Session will refresh from backend
    } catch (e) {
      console.error("Auto-resolve failed:", e);
    }
  };

  const basename = filename.split("/").pop() || filename;

  return (
    <div className="toolbar">
      <button onClick={onSave} title="Save (Cmd+S)">
        Save
      </button>
      <button className="magic" onClick={handleMagicWand} title="Magic Wand - Auto-resolve">
        Magic Wand
      </button>
      <span className="filename" title={filename}>
        {basename}
      </span>
      <span className="spacer" />
      <button className="danger" onClick={handleAbort} title="Abort (Cmd+Q)">
        Abort
      </button>
    </div>
  );
}
