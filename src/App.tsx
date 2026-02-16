import Toolbar from "./components/Toolbar";
import MergeView from "./components/MergeView";
import StatusBar from "./components/StatusBar";
import { useMergeSession } from "./hooks/useMergeSession";

export default function App() {
  const { session, loading, error, resolveHunk, saveResult, setResultContent } =
    useMergeSession();

  if (loading) {
    return (
      <div className="app-container" style={{ justifyContent: "center", alignItems: "center" }}>
        <p>Loading merge session...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="app-container" style={{ justifyContent: "center", alignItems: "center" }}>
        <p style={{ color: "#f44" }}>Error: {error}</p>
      </div>
    );
  }

  if (!session) {
    return (
      <div className="app-container" style={{ justifyContent: "center", alignItems: "center" }}>
        <p>No merge session available</p>
      </div>
    );
  }

  return (
    <div className="app-container">
      <Toolbar
        filename={session.mergedPath}
        onSave={saveResult}
      />
      <MergeView
        session={session}
        resolveHunk={resolveHunk}
        onResultEdit={setResultContent}
      />
      <StatusBar session={session} />
    </div>
  );
}
